/*!
This crate provides a simple "cursor" type for string slices.  It provides the ability to safely seek back and forth through a string without worrying about producing invalid UTF-8 sequences, or splitting grapheme clusters.

See the `StrCursor` type for details.
*/
extern crate unicode_segmentation as uniseg;

use uniseg::UnicodeSegmentation as UniSeg;

/**
This type represents a cursor into a string slice; that is, in addition to having a beginning and end, it also has a current position between those two.  This position can be seeked left and right within those bounds.

> **Note**: the cursor may validly be positioned *at* the end of the string.  That is, in a position where there are no code points or grapheme clusters to the right of the cursor, and the entire contents of the string is to the left of the cursor.

The main reason for this is that *sometimes*, you want the ability to do things like "advance a character", and the existing APIs for this can be somewhat verbose.

The cursor guarantees the following at all times:

* The cursor position *cannot* be outside of the original string slice it was constructed with.
* The cursor position *cannot* lie between unicode code points, meaning that you *cannot* generate an invalid string slice from a cursor.
* If the codepoint-specific methods are *not* used, the cursor will always lie between grapheme clusters.

This last point is somewhat important: the cursor is designed to favour operating on grapheme clusters, rather than codepoints.  If you mis-align the cursor with respect to grapheme clusters, the behaviour of methods that deal with grapheme clusters is *undefined*.

The methods that operate on the cursor will either return a fresh `Option<StrCursor>` (depending on whether the seek operation is valid or not), or mutate the existing cursor (in which case, they will *panic* if the seek operation is not valid).
*/
pub struct StrCursor<'a> {
    s: &'a str,
    at: *const u8,
}

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
    Seeks the cursor to the beginning of the previous grapheme cluster.

    # Panics

    If the cursor is currently at the start of the string, then this function will panic.
    */
    #[inline]
    pub fn seek_prev(&mut self) {
        if !self.try_seek_right_gr() {
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

    /**
    Returns the grapheme cluster immediately to the left of the cursor, or `None` is the cursor is at the start of the string.
    */
    #[inline]
    pub fn before(&self) -> Option<&'a str> {
        self.at_prev().and_then(|cur| cur.after())
    }

    /**
    Returns the grapheme cluster immediately to the right of the cursor, or `None` is the cursor is at the end of the string.
    */
    #[inline]
    pub fn after(&self) -> Option<&'a str> {
        UniSeg::graphemes(self.slice_after(), /*is_extended:*/true).next()
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
    Returns the contents of the string *between* this cursor and another cursor.

    Returns `None` if the cursors are from different strings (even different subsets of the same string).
    */
    #[inline]
    pub fn slice_between(&self, until: StrCursor<'a>) -> Option<&'a str> {
        if !str_eq_literal(self.s, until.s) {
            None
        } else {
            use std::cmp::{max, min};
            unsafe {
                let beg = min(self.at, until.at);
                let end = max(self.at, until.at);
                let len = end as usize - beg as usize;
                let bytes = ::std::slice::from_raw_parts(beg, len);
                Some(::std::str::from_utf8_unchecked(bytes))
            }
        }
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
    Returns the cursor's current position within the string as the number of UTF-8 code units from the beginning of the string.
    */
    #[inline]
    pub fn byte_pos(&self) -> usize {
        self.at as usize - self.s.as_ptr() as usize
    }

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

impl<'a> std::fmt::Debug for StrCursor<'a> {
	fn fmt(&self, fmt: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
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
    fn partial_cmp(&self, other: &StrCursor<'a>) -> Option<std::cmp::Ordering> {
        // If the cursors are from different strings, they are unordered.
        if (self.s.as_ptr() != other.s.as_ptr()) || (self.s.len() != other.s.len()) {
            None
        } else {
            self.at.partial_cmp(&other.at)
        }
    }
}

impl<'a> std::hash::Hash for StrCursor<'a> {
    fn hash<H>(&self, state: &mut H)
    where H: std::hash::Hasher {
        self.s.as_ptr().hash(state);
        self.s.len().hash(state);
        self.at.hash(state);
    }
}

#[cfg(test)]
#[test]
fn test_new_at_start() {
    let cur = StrCursor::new_at_start("abcdef");
    assert_eq!(cur.slice_before(), "");
    assert_eq!(cur.slice_after(), "abcdef");
}

#[cfg(test)]
#[test]
fn test_new_at_end() {
    let cur = StrCursor::new_at_end("abcdef");
    assert_eq!(cur.slice_before(), "abcdef");
    assert_eq!(cur.slice_after(), "");
}

#[cfg(test)]
#[test]
fn test_new_at_cp_left_of_byte_pos() {
    let s = "This is a 本当 test.";
    let cur = StrCursor::new_at_cp_left_of_byte_pos(s, 11);
    assert_eq!(cur.slice_before(), "This is a ");
    assert_eq!(cur.slice_after(), "本当 test.");
}

#[cfg(test)]
#[test]
fn test_new_at_cp_right_of_byte_pos() {
    let s = "This is a 本当 test.";
    let cur = StrCursor::new_at_cp_right_of_byte_pos(s, 11);
    assert_eq!(cur.slice_before(), "This is a 本");
    assert_eq!(cur.slice_after(), "当 test.");
}

#[cfg(test)]
#[test]
fn test_new_at_left_of_byte_pos() {
    let s = "Jäger,Jäger,大嫌い,💪❤!";
    let r = (0..s.len()+1).map(|i| (i, StrCursor::new_at_left_of_byte_pos(s, i)))
        .map(|(i, cur)| (i, cur.byte_pos(), cur.after()))
        .collect::<Vec<_>>();
    assert_eq!(r, vec![
        (0, 0, Some("J")),
        (1, 1, Some("ä")),
        (2, 1, Some("ä")),
        (3, 3, Some("g")),
        (4, 4, Some("e")),
        (5, 5, Some("r")),
        (6, 6, Some(",")),
        (7, 7, Some("J")),
        (8, 8, Some("ä")),
        (9, 8, Some("ä")),
        (10, 8, Some("ä")),
        (11, 11, Some("g")),
        (12, 12, Some("e")),
        (13, 13, Some("r")),
        (14, 14, Some(",")),
        (15, 15, Some("大")),
        (16, 15, Some("大")),
        (17, 15, Some("大")),
        (18, 18, Some("嫌")),
        (19, 18, Some("嫌")),
        (20, 18, Some("嫌")),
        (21, 21, Some("い")),
        (22, 21, Some("い")),
        (23, 21, Some("い")),
        (24, 24, Some(",")),
        (25, 25, Some("💪")),
        (26, 25, Some("💪")),
        (27, 25, Some("💪")),
        (28, 25, Some("💪")),
        (29, 29, Some("❤")),
        (30, 29, Some("❤")),
        (31, 29, Some("❤")),
        (32, 32, Some("!")),
        (33, 33, None),
    ]);
}

#[cfg(test)]
#[test]
fn test_new_at_right_of_byte_pos() {
    let s = "Jäger,Jäger,大嫌い,💪❤!";
    let r = (0..s.len()+1).map(|i| (i, StrCursor::new_at_right_of_byte_pos(s, i)))
        .map(|(i, cur)| (i, cur.byte_pos(), cur.after()))
        .collect::<Vec<_>>();
    assert_eq!(r, vec![
        (0, 0, Some("J")),
        (1, 1, Some("ä")),
        (2, 3, Some("g")),
        (3, 3, Some("g")),
        (4, 4, Some("e")),
        (5, 5, Some("r")),
        (6, 6, Some(",")),
        (7, 7, Some("J")),
        (8, 8, Some("ä")),
        (9, 11, Some("g")),
        (10, 11, Some("g")),
        (11, 11, Some("g")),
        (12, 12, Some("e")),
        (13, 13, Some("r")),
        (14, 14, Some(",")),
        (15, 15, Some("大")),
        (16, 18, Some("嫌")),
        (17, 18, Some("嫌")),
        (18, 18, Some("嫌")),
        (19, 21, Some("い")),
        (20, 21, Some("い")),
        (21, 21, Some("い")),
        (22, 24, Some(",")),
        (23, 24, Some(",")),
        (24, 24, Some(",")),
        (25, 25, Some("💪")),
        (26, 29, Some("❤")),
        (27, 29, Some("❤")),
        (28, 29, Some("❤")),
        (29, 29, Some("❤")),
        (30, 32, Some("!")),
        (31, 32, Some("!")),
        (32, 32, Some("!")),
        (33, 33, None),
    ]);
}

#[cfg(test)]
#[test]
fn test_at_prev_cp() {
    let s = "大嫌い,💪❤";
    let cur = StrCursor::new_at_end(s);
    let bps = test_util::finite_iterate(cur, StrCursor::at_prev_cp)
        .map(|cur| cur.byte_pos())
        .collect::<Vec<_>>();
    assert_eq!(bps, vec![14, 10, 9, 6, 3, 0]);
}

#[cfg(test)]
#[test]
fn test_at_next_cp() {
    let s = "大嫌い,💪❤";
    let cur = StrCursor::new_at_start(s);
    let bps = test_util::finite_iterate(cur, StrCursor::at_next_cp)
        .map(|cur| cur.byte_pos())
        .collect::<Vec<_>>();
    assert_eq!(bps, vec![3, 6, 9, 10, 14, 17]);
}

#[cfg(test)]
#[test]
fn test_at_prev_and_before() {
    let s = "noe\u{0308}l";
    let cur = StrCursor::new_at_end(s);
    let bps = test_util::finite_iterate_lead(cur, StrCursor::at_prev)
        .map(|cur| (cur.byte_pos(), cur.after()))
        .collect::<Vec<_>>();
    assert_eq!(bps, vec![
        (6, None),
        (5, Some("l")),
        (2, Some("e\u{0308}")),
        (1, Some("o")),
        (0, Some("n")),
    ]);
}

#[cfg(test)]
#[test]
fn test_at_next_and_after() {
    let s = "noe\u{0308}l";
    let cur = StrCursor::new_at_start(s);
    let bps = test_util::finite_iterate_lead(cur, StrCursor::at_next)
        .map(|cur| (cur.byte_pos(), cur.after()))
        .collect::<Vec<_>>();
    assert_eq!(bps, vec![
        (0, Some("n")),
        (1, Some("o")),
        (2, Some("e\u{0308}")),
        (5, Some("l")),
        (6, None),
    ]);
}

#[cfg(test)]
#[test]
fn test_char_before_and_after() {
    let s = "大嫌い,💪❤";
    let cur = StrCursor::new_at_start(s);
    let r = test_util::finite_iterate_lead(cur, StrCursor::at_next_cp)
        .map(|cur| (cur.byte_pos(), cur.cp_before(), cur.cp_after()))
        .collect::<Vec<_>>();
    assert_eq!(r, vec![
        (0, None, Some('大')),
        (3, Some('大'), Some('嫌')),
        (6, Some('嫌'), Some('い')),
        (9, Some('い'), Some(',')),
        (10, Some(','), Some('💪')),
        (14, Some('💪'), Some('❤')),
        (17, Some('❤'), None)
    ]);
}

#[cfg(test)]
#[test]
fn test_slice_between() {
    let s = "they hit, fight, kick, wreak havoc, and rejoice";
    let cur0 = StrCursor::new_at_start(s);
    let cur1 = StrCursor::new_at_end(s);
    let cur2 = StrCursor::new_at_end("nobody knows what they're lookin' for");
    let cur3 = StrCursor::new_at_end(&s[1..]);
    assert_eq!(cur0.slice_between(cur1), Some(s));
    assert_eq!(cur1.slice_between(cur0), Some(s));
    assert_eq!(cur0.slice_between(cur2), None);
    assert_eq!(cur0.slice_between(cur3), None);
}

#[inline]
fn byte_pos_to_ptr(s: &str, byte_pos: usize) -> *const u8 {
    if s.len() < byte_pos {
        panic!("byte position out of bounds: the len is {} but the position is {}",
            s.len(), byte_pos);
    }
    unsafe { s.as_ptr().offset(byte_pos as isize) }
}

#[inline]
unsafe fn seek_utf8_cp_start_left(s: &str, mut from: *const u8) -> *const u8 {
    let beg = s.as_ptr();
    while from > beg && (*from & 0b11_00_0000 == 0b10_00_0000) {
        from = from.offset(-1);
    }
    from
}

#[cfg(test)]
#[test]
fn test_seek_utf8_cp_start_left() {
    let s = "カブム！";
    let b = s.as_bytes();
    assert_eq!(unsafe { seek_utf8_cp_start_left(s, &b[0]) }, &b[0]);
    assert_eq!(unsafe { seek_utf8_cp_start_left(s, &b[1]) }, &b[0]);
    assert_eq!(unsafe { seek_utf8_cp_start_left(s, &b[2]) }, &b[0]);
    assert_eq!(unsafe { seek_utf8_cp_start_left(s, &b[3]) }, &b[3]);
    assert_eq!(unsafe { seek_utf8_cp_start_left(s, &b[4]) }, &b[3]);
    assert_eq!(unsafe { seek_utf8_cp_start_left(s, &b[5]) }, &b[3]);
}

#[inline]
unsafe fn seek_utf8_cp_start_right(s: &str, mut from: *const u8) -> *const u8 {
    let end = s.as_ptr().offset(s.len() as isize);
    while from < end && (*from & 0b11_00_0000 == 0b10_00_0000) {
        from = from.offset(1);
    }
    from
}

#[cfg(test)]
#[test]
fn test_seek_utf8_cp_start_right() {
    let s = "カブム！";
    let b = s.as_bytes();
    assert_eq!(unsafe { seek_utf8_cp_start_right(s, &b[0]) }, &b[0]);
    assert_eq!(unsafe { seek_utf8_cp_start_right(s, &b[1]) }, &b[3]);
    assert_eq!(unsafe { seek_utf8_cp_start_right(s, &b[2]) }, &b[3]);
    assert_eq!(unsafe { seek_utf8_cp_start_right(s, &b[3]) }, &b[3]);
    assert_eq!(unsafe { seek_utf8_cp_start_right(s, &b[4]) }, &b[6]);
    assert_eq!(unsafe { seek_utf8_cp_start_right(s, &b[5]) }, &b[6]);
}

#[inline]
fn str_eq_literal(a: &str, b: &str) -> bool {
    a.as_bytes().as_ptr() == b.as_bytes().as_ptr()
        && a.len() == b.len()
}

#[cfg(test)]
#[test]
fn test_str_eq_literal() {
    let s = "hare hare yukai";
    assert!(str_eq_literal(s, s));
    assert!(str_eq_literal(&s[0..4], &s[0..4]));
    assert!(!str_eq_literal(&s[0..4], &s[5..9]));
    assert!(!str_eq_literal(&s[0..4], &s[0..3]));
}

#[cfg(test)]
mod test_util {
    pub struct FiniteIter<T, F>(Option<T>, F);

    impl<T, F> Iterator for FiniteIter<T, F>
    where
        F: FnMut(T) -> Option<T>,
        T: Clone,
    {
        type Item = T;

        fn next(&mut self) -> Option<Self::Item> {
            self.0.take().and_then(|last| {
                match (self.1)(last) {
                    Some(e) => {
                        self.0 = Some(e);
                        self.0.clone()
                    },
                    None => None
                }
            })
        }
    }

    pub fn finite_iterate<T, F>(seed: T, f: F) -> FiniteIter<T, F>
    where
        F: FnMut(T) -> Option<T>,
        T: Clone,
    {
        FiniteIter(Some(seed), f)
    }
    pub struct FiniteIterLead<T, F>(Option<T>, F, bool);

    impl<T, F> Iterator for FiniteIterLead<T, F>
    where
        F: FnMut(T) -> Option<T>,
        T: Clone,
    {
        type Item = T;

        fn next(&mut self) -> Option<Self::Item> {
            if !self.2 {
                self.2 = true;
                return self.0.clone();
            }

            self.0.take().and_then(|last| {
                match (self.1)(last) {
                    Some(e) => {
                        self.0 = Some(e);
                        self.0.clone()
                    },
                    None => None
                }
            })
        }
    }

    pub fn finite_iterate_lead<T, F>(seed: T, f: F) -> FiniteIterLead<T, F>
    where
        F: FnMut(T) -> Option<T>,
        T: Clone,
    {
        FiniteIterLead(Some(seed), f, false)
    }
}
