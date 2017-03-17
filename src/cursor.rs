/*
Copyright ⓒ 2015-2017 Daniel Keep.

Licensed under the MIT license (see LICENSE or <http://opensource.org
/licenses/MIT>) or the Apache License, Version 2.0 (see LICENSE of
<http://www.apache.org/licenses/LICENSE-2.0>), at your option. All
files in the project carrying such notice may not be copied, modified,
or distributed except according to those terms.
*/
/*!
Cursor implementation.
*/
use std::cmp;
use std::fmt;
use std::hash;
use uniseg::UnicodeSegmentation as UniSeg;
use Gc;
use iter::{IterAfter, IterBefore, IterCpAfter, IterCpBefore};
use util::{byte_pos_to_ptr, seek_utf8_cp_start_left, seek_utf8_cp_start_right, str_eq_literal};

/**
This type represents a cursor into a string slice; that is, in addition to having a beginning and end, it also has a current position between those two.  This position can be seeked left and right within those bounds.

> **Note**: the cursor may validly be positioned *at* the end of the string.  That is, in a position where there are no code points or grapheme clusters to the right of the cursor, and the entire contents of the string is to the left of the cursor.

The main reason for this is that *sometimes*, you want the ability to do things like "advance a character", and the existing APIs for this can be somewhat verbose.

In addition, *unstable* support for grapheme clusters is exposed by the standard library, which conflicts with the *stable* support provided by the `unicode-segmentation` crate, which makes doing "the right thing" painful.  `StrCursor` exposes grapheme clusters by default, and makes them cleaner to work with.

The cursor guarantees the following at all times:

* The cursor position *cannot* be outside of the original string slice it was constructed with.
* The cursor position *cannot* lie between Unicode code points, meaning that you *cannot* generate an invalid string slice from a cursor.
* If the code point-specific methods are *not* used, the cursor will always lie between grapheme clusters.

This last point is somewhat important: the cursor is designed to favour operating on grapheme clusters, rather than code points.  If you misalign the cursor with respect to grapheme clusters, the behaviour of methods that deal with grapheme clusters is officially *undefined*, but is generally well-behaved.

The methods that operate on the cursor will either return a fresh `Option<StrCursor>` (depending on whether the seek operation is valid or not), or mutate the existing cursor (in which case, they will *panic* if the seek operation is not valid).

## Method Summary

Variants that deal with code points are not explicitly mentioned here.  In general, all methods that involve grapheme clusters have a corresponding code point variant.

- `new_at_…`: creates a cursor for a string at a given position.

- `before`/`after`: returns grapheme cluster before/after the cursor.
- `slice_before`/`slice_after`: returns contents of string before/after the cursor.
- `slice_all`: returns entire backing string.
- `slice_between`/`slice_until`: returns contents of string between (unordered/ordered) cursors.
- `byte_pos`: byte position of the cursor.

- `at_prev`/`at_next`: returns a derived, relatively positioned cursor.
- `prev`/`next`: efficiently combines `before`/`after` and `at_prev`/`at_next`
- `seek_prev`/`seek_next`: repositions a cursor in-place, panicking on out-of-bounds movement.

- `iter_before`/`iter_after`: iterate over grapheme clusters before/after the cursor.

There are also some unsafe methods for performance-critical cases.  Note that these methods *do not* check their arguments for validity, and if misused can violate Rust's safety guarantees.

- `unsafe_seek_…`: seeks a given number of bytes left/right.
- `unsafe_set_at`: sets the cursor position directly.
- `unsafe_slice_until`: slices between two cursors.
*/
pub struct StrCursor<'a> {
    s: &'a str,
    at: *const u8,
}

/**
Cursor creation.
*/
impl<'a> StrCursor<'a> {
    /**
    Create a new cursor at the start of `s`.
    */
    #[inline]
    pub fn new_at_start(s: &'a str) -> StrCursor<'a> {
        StrCursor {
            s: s,
            at: s.as_ptr(),
        }
    }

    /**
    Create a new cursor past at the end of `s`.
    */
    #[inline]
    pub fn new_at_end(s: &'a str) -> StrCursor<'a> {
        StrCursor {
            s: s,
            at: byte_pos_to_ptr(s, s.len()),
        }
    }

    /**
    Create a new cursor at the first grapheme cluster which begins at or to the left of the given byte position.
    */
    #[inline]
    pub fn new_at_left_of_byte_pos(s: &'a str, byte_pos: usize) -> StrCursor<'a> {
        // Start at a codepoint.
        let cur = StrCursor::new_at_cp_left_of_byte_pos(s, byte_pos);

        // Seek back to the previous grapheme.
        let prev = cur.at_prev();

        let prev = match prev {
            None => return cur, // We were already at the start.
            Some(c) => c
        };

        // unwrap should be OK here.
        if prev.byte_pos() + prev.after().unwrap().len() > byte_pos {
            prev
        } else {
            cur
        }
    }

    /**
    Create a new cursor at the first grapheme cluster which begins at or to the right of the given byte position.
    */
    #[inline]
    pub fn new_at_right_of_byte_pos(s: &'a str, byte_pos: usize) -> StrCursor<'a> {
        // I don't know how robust the grapheme iteration rules are when trying to step forward from a (potentially) invalid position.  As such, I'm *instead* going to start from a known-good position.
        let cur = StrCursor::new_at_left_of_byte_pos(s, byte_pos);
        if cur.byte_pos() == byte_pos {
            return cur;
        }

        // This unwrap shouldn't be able to fail.
        cur.at_next().unwrap()
    }

    /**
    Create a new cursor at the first code point which begins at or to the left of the given byte position.

    # Note

    Where possible, you should prefer `new_at_left_of_byte_pos`.
    */
    #[inline]
    pub fn new_at_cp_left_of_byte_pos(s: &'a str, byte_pos: usize) -> StrCursor<'a> {
        StrCursor {
            s: s,
            at: unsafe { seek_utf8_cp_start_left(s, byte_pos_to_ptr(s, byte_pos)) },
        }
    }

    /**
    Create a new cursor at the first code point which begins at or to the right of the given byte position.

    # Note

    Where possible, you should prefer `new_at_right_of_byte_pos`.
    */
    #[inline]
    pub fn new_at_cp_right_of_byte_pos(s: &'a str, byte_pos: usize) -> StrCursor<'a> {
        StrCursor {
            s: s,
            at: unsafe { seek_utf8_cp_start_right(s, byte_pos_to_ptr(s, byte_pos)) },
        }
    }
}

/**
Cursor inspection.
*/
impl<'a> StrCursor<'a> {
    /**
    Returns the grapheme cluster immediately to the left of the cursor, or `None` is the cursor is at the start of the string.
    */
    #[inline]
    pub fn before(&self) -> Option<&'a Gc> {
        self.at_prev().and_then(|cur| cur.after())
    }

    /**
    Returns the grapheme cluster immediately to the right of the cursor, or `None` is the cursor is at the end of the string.
    */
    #[inline]
    pub fn after(&self) -> Option<&'a Gc> {
        Gc::split_from(self.slice_after()).map(|(gc, _)| gc)
    }

    /**
    Returns the code point immediately to the left of the cursor, or `None` is the cursor is at the start of the string.
    */
    #[inline]
    pub fn cp_before(&self) -> Option<char> {
        self.at_prev_cp().and_then(|cur| cur.cp_after())
    }

    /**
    Returns the code point immediately to the right of the cursor, or `None` is the cursor is at the end of the string.
    */
    #[inline]
    pub fn cp_after(&self) -> Option<char> {
        self.slice_after().chars().next()
    }

    /**
    Returns the contents of the string to the left of the cursor.
    */
    #[inline]
    pub fn slice_before(&self) -> &'a str {
        unsafe {
            self.s.slice_unchecked(0, self.byte_pos())
        }
    }

    /**
    Returns the contents of the string to the right of the cursor.
    */
    #[inline]
    pub fn slice_after(&self) -> &'a str {
        unsafe {
            self.s.slice_unchecked(self.byte_pos(), self.s.len())
        }
    }

    /**
    Returns the entire string slice behind the cursor.
    */
    #[inline]
    pub fn slice_all(&self) -> &'a str {
        self.s
    }

    /**
    Returns the contents of the string *between* this cursor and another cursor.

    The order of the cursors does not matter; `a.slice_between(b)` and `b.slice_between(a)` produce the same results.

    Returns `None` if the cursors are from different strings (even different subsets of the same string).
    */
    #[inline]
    pub fn slice_between(&self, other: StrCursor<'a>) -> Option<&'a str> {
        if !str_eq_literal(self.s, other.s) {
            None
        } else {
            use std::cmp::{max, min};
            unsafe {
                let beg = min(self.at, other.at);
                let end = max(self.at, other.at);
                let len = end as usize - beg as usize;
                let bytes = ::std::slice::from_raw_parts(beg, len);
                Some(::std::str::from_utf8_unchecked(bytes))
            }
        }
    }

    /**
    Returns the contents of the string *starting* at this cursor, ending at another.

    The order of the cursors matters; if `b` points to a position *before* `a` within the same string, `a.slice_until(b)` will result in an empty string slice.

    Returns `None` if the cursors are from different strings (even different subsets of the same string).
    */
    #[inline]
    pub fn slice_until(&self, end: StrCursor<'a>) -> Option<&'a str> {
        if !str_eq_literal(self.s, end.s) {
            None
        } else {
            unsafe {
                Some(self.unsafe_slice_until(end))
            }
        }
    }

    /**
    Returns the cursor's current position within the string as the number of UTF-8 code units from the beginning of the string.
    */
    #[inline]
    pub fn byte_pos(&self) -> usize {
        self.at as usize - self.s.as_ptr() as usize
    }
}

/**
Cursor movement.
*/
impl<'a> StrCursor<'a> {
    /**
    Returns a new cursor at the beginning of the previous grapheme cluster, or `None` if the cursor is currently positioned at the beginning of the string.
    */
    #[inline]
    pub fn at_prev(mut self) -> Option<StrCursor<'a>> {
        match self.try_seek_left_gr() {
            true => Some(self),
            false => None
        }
    }

    /**
    Returns a new cursor at the beginning of the next grapheme cluster, or `None` if the cursor is currently positioned at the end of the string.
    */
    #[inline]
    pub fn at_next(mut self) -> Option<StrCursor<'a>> {
        match self.try_seek_right_gr() {
            true => Some(self),
            false => None
        }
    }

    /**
    Returns a new cursor at the beginning of the previous code point, or `None` if the cursor is currently positioned at the beginning of the string.

    # Note

    Where possible, you should prefer `at_prev`.
    */
    #[inline]
    pub fn at_prev_cp(mut self) -> Option<StrCursor<'a>> {
        match self.try_seek_left_cp() {
            true => Some(self),
            false => None
        }
    }

    /**
    Returns a new cursor at the beginning of the next code point, or `None` if the cursor is currently positioned at the end of the string.

    # Note

    Where possible, you should prefer `at_next`.
    */
    #[inline]
    pub fn at_next_cp(mut self) -> Option<StrCursor<'a>> {
        match self.try_seek_right_cp() {
            true => Some(self),
            false => None
        }
    }

    /**
    Returns both the previous grapheme cluster and the cursor having seeked before it.

    This may be more efficient than doing both operations individually.
    */
    #[inline]
    pub fn prev(mut self) -> Option<(&'a Gc, StrCursor<'a>)> {
        unsafe {
            let g = match self.before() {
                Some(g) => g,
                None => return None,
            };
            self.unsafe_set_at(g.as_str());
            Some((g, self))
        }
    }

    /**
    Returns both the previous code point and the cursor having seeked before it.

    This may be more efficient than doing both operations individually.

    # Note

    Where possible, you should prefer `prev`.
    */
    #[inline]
    pub fn prev_cp(mut self) -> Option<(char, StrCursor<'a>)> {
        unsafe {
            let cp = match self.cp_before() {
                Some(cp) => cp,
                None => return None,
            };
            self.unsafe_seek_left(cp.len_utf8());
            Some((cp, self))
        }
    }

    /**
    Returns both the next grapheme cluster and the cursor having seeked past it.

    This may be more efficient than doing both operations individually.
    */
    #[inline]
    pub fn next(mut self) -> Option<(&'a Gc, StrCursor<'a>)> {
        unsafe {
            let g = match self.after() {
                Some(g) => g,
                None => return None,
            };
            self.unsafe_seek_right(g.len());
            Some((g, self))
        }
    }

    /**
    Returns both the next code point and the cursor having seeked past it.

    This may be more efficient than doing both operations individually.

    # Note

    Where possible, you should prefer `next`.
    */
    #[inline]
    pub fn next_cp(mut self) -> Option<(char, StrCursor<'a>)> {
        unsafe {
            let cp = match self.cp_after() {
                Some(cp) => cp,
                None => return None,
            };
            self.unsafe_seek_right(cp.len_utf8());
            Some((cp, self))
        }
    }

    /**
    Seeks the cursor to the beginning of the previous grapheme cluster.

    # Panics

    If the cursor is currently at the start of the string, then this function will panic.
    */
    #[inline]
    pub fn seek_prev(&mut self) {
        if !self.try_seek_left_gr() {
            panic!("cannot seek past the beginning of a string");
        }
    }

    /**
    Seeks the cursor to the beginning of the next grapheme cluster.

    # Panics

    If the cursor is currently at the end of the string, then this function will panic.
    */
    #[inline]
    pub fn seek_next(&mut self) {
        if !self.try_seek_right_gr() {
            panic!("cannot seek past the end of a string");
        }
    }

    /**
    Seeks the cursor to the beginning of the previous code point.

    # Panics

    If the cursor is currently at the start of the string, then this function will panic.

    # Note

    Where possible, you should prefer `seek_prev`.
    */
    #[inline]
    pub fn seek_prev_cp(&mut self) {
        if !self.try_seek_left_cp() {
            panic!("cannot seek past the beginning of a string");
        }
    }

    /**
    Seeks the cursor to the beginning of the next code point.

    # Panics

    If the cursor is currently at the end of the string, then this function will panic.

    # Note

    Where possible, you should prefer `seek_next`.
    */
    #[inline]
    pub fn seek_next_cp(&mut self) {
        if !self.try_seek_right_cp() {
            panic!("cannot seek past the end of a string");
        }
    }
}

/**
Predicate methods.
*/
impl<'a> StrCursor<'a> {
    #[inline]
    pub fn before_while<P>(self, mut predicate: P) -> (&'a str, Self)
    where P: FnMut(&'a Gc) -> bool {
        let start = self;
        let mut at = self;
        loop {
            match at.prev() {
                None => break,
                Some((gc, at2)) => {
                    if predicate(gc) {
                        at = at2;
                    } else {
                        break;
                    }
                },
            }
        }
        unsafe {
            (at.unsafe_slice_until(start), at)
        }
    }

    #[inline]
    pub fn after_while<P>(self, mut predicate: P) -> (&'a str, Self)
    where P: FnMut(&'a Gc) -> bool {
        let start = self;
        let mut at = self;
        loop {
            match at.next() {
                None => break,
                Some((gc, at2)) => {
                    if predicate(gc) {
                        at = at2;
                    } else {
                        break;
                    }
                },
            }
        }
        unsafe {
            (start.unsafe_slice_until(at), at)
        }
    }

    #[inline]
    pub fn cp_before_while<P>(self, mut predicate: P) -> (&'a str, Self)
    where P: FnMut(char) -> bool {
        let start = self;
        let mut at = self;
        loop {
            match at.prev_cp() {
                None => break,
                Some((cp, at2)) => {
                    if predicate(cp) {
                        at = at2;
                    } else {
                        break;
                    }
                },
            }
        }
        unsafe {
            (at.unsafe_slice_until(start), at)
        }
    }

    #[inline]
    pub fn cp_after_while<P>(self, mut predicate: P) -> (&'a str, Self)
    where P: FnMut(char) -> bool {
        let start = self;
        let mut at = self;
        loop {
            match at.next_cp() {
                None => break,
                Some((cp, at2)) => {
                    if predicate(cp) {
                        at = at2;
                    } else {
                        break;
                    }
                },
            }
        }
        unsafe {
            (start.unsafe_slice_until(at), at)
        }
    }
}

/**
Iterator methods.
*/
impl<'a> StrCursor<'a> {
    /**
    Iterates over grapheme clusters right-to-left, starting at the cursor.

    You can call the `with_cursor` method on the result to get an iterator over `(&Gc, StrCursor)` pairs.
    */
    #[inline]
    pub fn iter_before(self) -> IterBefore<'a> {
        IterBefore(self)
    }

    /**
    Iterates over grapheme clusters left-to-right, starting at the cursor.

    You can call the `with_cursor` method on the result to get an iterator over `(&Gc, StrCursor)` pairs.
    */
    #[inline]
    pub fn iter_after(self) -> IterAfter<'a> {
        IterAfter(self)
    }

    /**
    Iterates over grapheme clusters right-to-left, starting at the cursor.

    You can call the `with_cursor` method on the result to get an iterator over `(&Gc, StrCursor)` pairs.

    # Note

    Where possible, you should prefer `iter_before`.
    */
    #[inline]
    pub fn iter_cp_before(self) -> IterCpBefore<'a> {
        IterCpBefore(self)
    }

    /**
    Iterates over grapheme clusters left-to-right, starting at the cursor.

    You can call the `with_cursor` method on the result to get an iterator over `(&Gc, StrCursor)` pairs.

    # Note

    Where possible, you should prefer `iter_after`.
    */
    #[inline]
    pub fn iter_cp_after(self) -> IterCpAfter<'a> {
        IterCpAfter(self)
    }
}

/**
Unsafe methods.

These methods do not perform any validity checking, and as such should be used with extreme caution.
*/
impl<'a> StrCursor<'a> {
    /**
    Creates a cursor at the specified byte offset, without performing any bounds or validity checks.
    */
    #[inline]
    pub unsafe fn unsafe_new_at_byte_pos(s: &'a str, byte_pos: usize) -> StrCursor<'a> {
        StrCursor {
            s: s,
            at: byte_pos_to_ptr(s, byte_pos),
        }
    }

    /**
    Seeks exactly `bytes` left, without performing any bounds or validity checks.
    */
    #[inline]
    pub unsafe fn unsafe_seek_left(&mut self, bytes: usize) {
        self.at = self.at.offset(-(bytes as isize));
    }

    /**
    Seeks exactly `bytes` right, without performing any bounds or validity checks.
    */
    #[inline]
    pub unsafe fn unsafe_seek_right(&mut self, bytes: usize) {
        self.at = self.at.offset(bytes as isize);
    }

    /**
    Seeks to the start of `s`, without performing any bounds or validity checks.
    */
    #[inline]
    pub unsafe fn unsafe_set_at(&mut self, s: &'a str) {
        self.at = s.as_bytes().as_ptr();
    }

    /**
    Returns the string slice between two cursors, without performing any bounds or validity checks.
    */
    #[inline]
    pub unsafe fn unsafe_slice_until(self, end: Self) -> &'a str {
        let beg = self.at;
        let end = end.at;
        let len = if end >= beg {
            end as usize - beg as usize
        } else {
            0
        };
        let bytes = ::std::slice::from_raw_parts(beg, len);
        ::std::str::from_utf8_unchecked(bytes)
    }
}

/**
Internal methods.
*/
impl<'a> StrCursor<'a> {
    #[inline]
    fn try_seek_left_cp(&mut self) -> bool {
        unsafe {
            // We just have to ensure that offsetting the `at` pointer *at all* is safe.
            if self.byte_pos() == 0 {
                return false;
            }
            self.at = seek_utf8_cp_start_left(self.s, self.at.offset(-1));
            true
        }
    }

    #[inline]
    fn try_seek_right_cp(&mut self) -> bool {
        unsafe {
            // We just have to ensure that offsetting the `at` pointer *at all* is safe.
            if self.byte_pos() == self.s.len() {
                return false;
            }
            self.at = seek_utf8_cp_start_right(self.s, self.at.offset(1));
            true
        }
    }

    #[inline]
    fn try_seek_left_gr(&mut self) -> bool {
        let len = {
            let gr = UniSeg::graphemes(self.slice_before(), /*is_extended:*/true).next_back();
            gr.map(|gr| gr.len())
        };
        match len {
            Some(len) => {
                unsafe {
                    self.at = self.at.offset(-(len as isize));
                }
                true
            },
            None => false
        }
    }

    #[inline]
    fn try_seek_right_gr(&mut self) -> bool {
        let len = {
            let gr = UniSeg::graphemes(self.slice_after(), /*is_extended:*/true).next();
            gr.map(|gr| gr.len())
        };
        match len {
            Some(len) => {
                unsafe {
                    self.at = self.at.offset(len as isize);
                }
                true
            },
            None => false
        }
    }
}

impl<'a> Copy for StrCursor<'a> {}

impl<'a> Clone for StrCursor<'a> {
    fn clone(&self) -> StrCursor<'a> {
        *self
    }
}

impl<'a> fmt::Debug for StrCursor<'a> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(fmt, "StrCursor({:?} | {:?})", self.slice_before(), self.slice_after())
    }
}

impl<'a> Eq for StrCursor<'a> {}

impl<'a> PartialEq for StrCursor<'a> {
    fn eq(&self, other: &StrCursor<'a>) -> bool {
        (self.at == other.at)
        && (self.s.as_ptr() == other.s.as_ptr())
        && (self.s.len() == other.s.len())
    }

    fn ne(&self, other: &StrCursor<'a>) -> bool {
        (self.at != other.at)
        || (self.s.as_ptr() != other.s.as_ptr())
        || (self.s.len() != other.s.len())
    }
}

impl<'a> PartialOrd for StrCursor<'a> {
    fn partial_cmp(&self, other: &StrCursor<'a>) -> Option<cmp::Ordering> {
        // If the cursors are from different strings, they are unordered.
        if (self.s.as_ptr() != other.s.as_ptr()) || (self.s.len() != other.s.len()) {
            None
        } else {
            self.at.partial_cmp(&other.at)
        }
    }
}

impl<'a> hash::Hash for StrCursor<'a> {
    fn hash<H>(&self, state: &mut H)
    where H: hash::Hasher {
        self.s.as_ptr().hash(state);
        self.s.len().hash(state);
        self.at.hash(state);
    }
}
