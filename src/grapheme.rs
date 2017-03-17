/*
Copyright ⓒ 2015-2017 Daniel Keep.

Licensed under the MIT license (see LICENSE or <http://opensource.org
/licenses/MIT>) or the Apache License, Version 2.0 (see LICENSE of
<http://www.apache.org/licenses/LICENSE-2.0>), at your option. All
files in the project carrying such notice may not be copied, modified,
or distributed except according to those terms.
*/
/*!
Defines types for representing single grapheme clusters.
*/
use std::borrow::{Borrow, Cow, ToOwned};
use std::convert::AsRef;
use std::cmp::Ordering;
use std::fmt::{self, Debug, Display};
use std::mem::transmute;
use std::ops::Deref;
use uniseg::UnicodeSegmentation as UniSeg;

/**
An iterator over the lower case mapping of a given grapheme cluster, returned from [`Gc::to_lowercase`](struct.Gc.html#method.to_lowercase).
*/
pub type ToLowercase<'a> = ::std::iter::FlatMap<::std::str::Chars<'a>, ::std::char::ToLowercase, fn(char) -> ::std::char::ToLowercase>;

/**
An iterator over the lower case mapping of a given grapheme cluster, returned from [`Gc::to_uppercase`](struct.Gc.html#method.to_uppercase).
*/
pub type ToUppercase<'a> = ::std::iter::FlatMap<::std::str::Chars<'a>, ::std::char::ToUppercase, fn(char) -> ::std::char::ToUppercase>;

/**
A slice of a single Unicode grapheme cluster (GC) (akin to `str`).

A grapheme cluster is a single visual "unit" in Unicode text, and is composed of *at least* one Unicode code point, possibly more.

This type is a wrapper around `str` that enforces the additional invariant that it will *always* contain *exactly* one grapheme cluster.  This allows some operations (such as extracting the base code point) simpler.

## Why Grapheme Clusters?

The simplest example is the distinction between "é" ("Latin Small Letter E with Acute") and "é" ("Latin Small Letter E", "Combining Acute Accent"): the first is *one* code point, the second is *two*.

In Rust, the `char` type is a single code point.  As a result, treating it as a "character" is incorrect for the same reason that using `u8` is: it excludes many legitimate characters.  It can also cause issues whereby naive algorithms may corrupt text by considering components of a grapheme cluster separately.  For example, truncating a string to "10 characters" using `char`s can lead to logical characters being broken apart, potentially changing their meaning.

One inconvenience when dealing with grapheme clusters in Rust is that they are not accurately represented by any type more-so than a regular `&str`.  However, operations that might make sense on an individual character (such as asking whether it is in the ASCII range, or is numeric) don't make sense on a full string.  In addition, a `&str` can be empty or contain more than one grapheme cluster.

Hence, this type guarantees that it always represents *exactly* one Unicode grapheme cluster.
*/
#[derive(Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Gc(str);

impl Gc {
    /**
    Create a new `Gc` from the given string slice.

    The slice must contain *exactly* one grapheme cluster.  In the event that the input is empty, or contains more than one grapheme cluster, this function will return `None`.

    See: [`split_from`](#method.split_from).
    */
    pub fn from_str(s: &str) -> Option<&Gc> {
        match Gc::split_from(s) {
            Some((gc, tail)) => if tail.len() == 0 { Some(gc) } else { None },
            None => None
        }
    }

    /**
    Create a new `Gc` from the given string slice.

    This function *does not* check to ensure the provided slice is a single, valid grapheme cluster.
    */
    pub unsafe fn from_str_unchecked(s: &str) -> &Gc {
        transmute(s)
    }

    /**
    Try to split a single grapheme cluster from the start of `s`.

    Returns `None` if the given string was empty.
    */
    pub fn split_from(s: &str) -> Option<(&Gc, &str)> {
        unsafe {
            let gr = match UniSeg::graphemes(s, /*is_extended:*/true).next() {
                Some(gr) => gr,
                None => return None,
            };
            Some((Gc::from_str_unchecked(gr), s.slice_unchecked(gr.len(), s.len())))
        }
    }

    /**
    Returns the length of this grapheme cluster in bytes.
    */
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /**
    Does this grapheme cluster have additional marks applied to it?

    This is `true` if the cluster is comprised of more than a single code point.
    */
    pub fn has_marks(&self) -> bool {
        self.base_char().len_utf8() != self.as_str().len()
    }

    /**
    Converts this to a byte slice.
    */
    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }

    /**
    Converts this to a string slice.
    */
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /**
    Returns the "base" code point.

    That is, this returns the first code point in the cluster.
    */
    pub fn base_char(&self) -> char {
        unsafe {
            match self.0.chars().next() {
                Some(cp) => cp,
                None => debug_unreachable!(),
            }
        }
    }

    /**
    Returns the "base" code point as a grapheme cluster.

    This is equivalent to converting this GC into a string slice, then slicing off the bytes that make up the first code point.
    */
    pub fn base(&self) -> &Gc {
        unsafe {
            let base_cp = self.base_char();
            let base_len = base_cp.len_utf8();
            Gc::from_str_unchecked(self.0.slice_unchecked(base_len, self.0.len()))
        }
    }

    /**
    Checks the given predicate against a non-composed cluster.

    This can be used in cases where you want to test some predicate on "simple" grapheme clusters, without having to separately check that the grapheme has no combining marks, and that the base code point satisfies the predicate.

    If the grapheme cluster contains combining marks, this method *always* returns `false`.

    # Example

    ```rust
    # use strcursor::Gc;
    // space + combining double rightwards arrow below
    let gc0 = Gc::from_str(" ͢").unwrap();
    // just space
    let gc1 = Gc::from_str(" ").unwrap();

    // gc0 probably shouldn't be interpreted as whitespace, but this passes:
    assert_eq!(gc0.base_char().is_whitespace(), true);
    assert_eq!(gc1.base_char().is_whitespace(), true);

    // Solution: only apply test to "simple" clusters:
    assert_eq!(gc0.is_base(char::is_whitespace), false);
    assert_eq!(gc1.is_base(char::is_whitespace), true);
    ```
    */
    pub fn is_base<P>(&self, predicate: P) -> bool
    where P: FnOnce(char) -> bool {
        if self.has_marks() {
            false
        } else {
            let cp = self.base_char();
            predicate(cp)
        }
    }

    /**
    Returns the combining marks as a string slice.

    The result of this method may be empty, or of arbitrary length.
    */
    pub fn mark_str(&self) -> &str {
        unsafe {
            let base_cp = self.base_char();
            let base_len = base_cp.len_utf8();
            self.0.slice_unchecked(base_len, self.0.len())
        }
    }

    /**
    An iterator over the code points of this grapheme cluster.
    */
    pub fn chars(&self) -> ::std::str::Chars {
        self.0.chars()
    }

    /**
    An iterator over the code points of this grapheme cluster, and their associated byte offsets.
    */
    pub fn char_indices(&self) -> ::std::str::CharIndices {
        self.0.char_indices()
    }

    /**
    An iterator over the bytes of this grapheme cluster.
    */
    pub fn bytes(&self) -> ::std::str::Bytes {
        self.0.bytes()
    }

    /**
    Returns an iterator over the code points in the lower case equivalent of this grapheme cluster.
    */
    pub fn to_lowercase(&self) -> ToLowercase {
        self.0.chars().flat_map(char::to_lowercase)
    }

    /**
    Returns an iterator over the code points in the upper case equivalent of this grapheme cluster.
    */
    pub fn to_uppercase(&self) -> ToUppercase {
        self.0.chars().flat_map(char::to_uppercase)
    }
}

impl AsRef<str> for Gc {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl AsRef<[u8]> for Gc {
    fn as_ref(&self) -> &[u8] {
        self.as_str().as_bytes()
    }
}

impl Debug for Gc {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        Debug::fmt(&self.0, fmt)
    }
}

impl Display for Gc {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(&self.0, fmt)
    }
}

impl<'a> PartialEq<&'a Gc> for Gc {
    fn eq(&self, other: &&'a Gc) -> bool {
        self.eq(*other)
    }
}

impl<'a> PartialEq<Gc> for &'a Gc {
    fn eq(&self, other: &Gc) -> bool {
        (*self).eq(other)
    }
}

impl PartialEq<char> for Gc {
    fn eq(&self, other: &char) -> bool {
        !self.has_marks() && self.base_char().eq(other)
    }
}

impl PartialEq<str> for Gc {
    fn eq(&self, other: &str) -> bool {
        self.0.eq(other)
    }
}

impl<'a> PartialEq<&'a str> for Gc {
    fn eq(&self, other: &&'a str) -> bool {
        self.0.eq(*other)
    }
}

impl PartialEq<GcBuf> for Gc {
    fn eq(&self, other: &GcBuf) -> bool {
        self.0.eq(other.as_gc())
    }
}

impl PartialEq<String> for Gc {
    fn eq(&self, other: &String) -> bool {
        self.0.eq(&**other)
    }
}

impl<'a> PartialEq<Cow<'a, Gc>> for Gc {
    fn eq(&self, other: &Cow<'a, Gc>) -> bool {
        self.0.eq((*other).deref())
    }
}

impl<'a> PartialEq<char> for &'a Gc {
    fn eq(&self, other: &char) -> bool {
        !self.has_marks() && self.base_char().eq(other)
    }
}

impl<'a> PartialEq<str> for &'a Gc {
    fn eq(&self, other: &str) -> bool {
        self.0.eq(other)
    }
}

impl<'a> PartialEq<GcBuf> for &'a Gc {
    fn eq(&self, other: &GcBuf) -> bool {
        self.0.eq(other.as_gc())
    }
}

impl<'a> PartialEq<String> for &'a Gc {
    fn eq(&self, other: &String) -> bool {
        self.0.eq(&**other)
    }
}

impl<'a> PartialEq<Cow<'a, Gc>> for &'a Gc {
    fn eq(&self, other: &Cow<'a, Gc>) -> bool {
        self.0.eq((*other).deref())
    }
}

impl PartialEq<Gc> for char {
    fn eq(&self, other: &Gc) -> bool {
        self.eq(&other.base_char())
    }
}

impl PartialEq<Gc> for str {
    fn eq(&self, other: &Gc) -> bool {
        self.eq(&other.0)
    }
}

impl<'a> PartialEq<Gc> for &'a str {
    fn eq(&self, other: &Gc) -> bool {
        self.eq(&&other.0)
    }
}

impl PartialEq<Gc> for String {
    fn eq(&self, other: &Gc) -> bool {
        self.eq(&other.as_str())
    }
}

impl<'a> PartialEq<Gc> for Cow<'a, Gc> {
    fn eq(&self, other: &Gc) -> bool {
        (**self).eq(other)
    }
}

impl<'a> PartialEq<&'a Gc> for char {
    fn eq(&self, other: &&'a Gc) -> bool {
        self.eq(&other.base_char())
    }
}

impl<'a> PartialEq<&'a Gc> for str {
    fn eq(&self, other: &&'a Gc) -> bool {
        self.eq(&other.0)
    }
}

impl<'a> PartialEq<&'a Gc> for String {
    fn eq(&self, other: &&'a Gc) -> bool {
        self.eq(&other.as_str())
    }
}

impl<'a> PartialEq<&'a Gc> for Cow<'a, Gc> {
    fn eq(&self, other: &&'a Gc) -> bool {
        (**self).eq(*other)
    }
}

impl<'a> PartialOrd<&'a Gc> for Gc {
    fn partial_cmp(&self, other: &&'a Gc) -> Option<Ordering> {
        self.partial_cmp(*other)
    }
}

impl<'a> PartialOrd<Gc> for &'a Gc {
    fn partial_cmp(&self, other: &Gc) -> Option<Ordering> {
        (*self).partial_cmp(other)
    }
}

impl PartialOrd<char> for Gc {
    fn partial_cmp(&self, other: &char) -> Option<Ordering> {
        if !self.has_marks() {
            self.base_char().partial_cmp(other)
        } else {
            match self.base_char().partial_cmp(other) {
                Some(Ordering::Equal) => Some(Ordering::Less),
                other => other
            }
        }
    }
}

impl PartialOrd<str> for Gc {
    fn partial_cmp(&self, other: &str) -> Option<Ordering> {
        self.0.partial_cmp(other)
    }
}

impl<'a> PartialOrd<&'a str> for Gc {
    fn partial_cmp(&self, other: &&'a str) -> Option<Ordering> {
        self.0.partial_cmp(*other)
    }
}

impl PartialOrd<GcBuf> for Gc {
    fn partial_cmp(&self, other: &GcBuf) -> Option<Ordering> {
        self.0.partial_cmp(other.as_gc())
    }
}

impl PartialOrd<String> for Gc {
    fn partial_cmp(&self, other: &String) -> Option<Ordering> {
        self.0.partial_cmp(&**other)
    }
}

impl<'a> PartialOrd<Cow<'a, Gc>> for Gc {
    fn partial_cmp(&self, other: &Cow<'a, Gc>) -> Option<Ordering> {
        self.0.partial_cmp((*other).deref())
    }
}

impl<'a> PartialOrd<char> for &'a Gc {
    fn partial_cmp(&self, other: &char) -> Option<Ordering> {
        other.partial_cmp(self).map(Ordering::reverse)
    }
}

impl<'a> PartialOrd<str> for &'a Gc {
    fn partial_cmp(&self, other: &str) -> Option<Ordering> {
        self.0.partial_cmp(other)
    }
}

impl<'a> PartialOrd<GcBuf> for &'a Gc {
    fn partial_cmp(&self, other: &GcBuf) -> Option<Ordering> {
        self.0.partial_cmp(other.as_gc())
    }
}

impl<'a> PartialOrd<String> for &'a Gc {
    fn partial_cmp(&self, other: &String) -> Option<Ordering> {
        self.0.partial_cmp(&**other)
    }
}

impl<'a> PartialOrd<Cow<'a, Gc>> for &'a Gc {
    fn partial_cmp(&self, other: &Cow<'a, Gc>) -> Option<Ordering> {
        self.0.partial_cmp((*other).deref())
    }
}

impl PartialOrd<Gc> for char {
    fn partial_cmp(&self, other: &Gc) -> Option<Ordering> {
        self.partial_cmp(&other.base_char())
    }
}

impl PartialOrd<Gc> for str {
    fn partial_cmp(&self, other: &Gc) -> Option<Ordering> {
        self.partial_cmp(&other.0)
    }
}

impl<'a> PartialOrd<Gc> for &'a str {
    fn partial_cmp(&self, other: &Gc) -> Option<Ordering> {
        self.partial_cmp(&&other.0)
    }
}

impl PartialOrd<Gc> for String {
    fn partial_cmp(&self, other: &Gc) -> Option<Ordering> {
        (&**self).partial_cmp(other.as_str())
    }
}

impl<'a> PartialOrd<Gc> for Cow<'a, Gc> {
    fn partial_cmp(&self, other: &Gc) -> Option<Ordering> {
        (**self).partial_cmp(other)
    }
}

impl<'a> PartialOrd<&'a Gc> for char {
    fn partial_cmp(&self, other: &&'a Gc) -> Option<Ordering> {
        self.partial_cmp(&other.base_char())
    }
}

impl<'a> PartialOrd<&'a Gc> for str {
    fn partial_cmp(&self, other: &&'a Gc) -> Option<Ordering> {
        self.partial_cmp(&other.0)
    }
}

impl<'a> PartialOrd<&'a Gc> for String {
    fn partial_cmp(&self, other: &&'a Gc) -> Option<Ordering> {
        (&**self).partial_cmp(other.as_str())
    }
}

impl<'a> PartialOrd<&'a Gc> for Cow<'a, Gc> {
    fn partial_cmp(&self, other: &&'a Gc) -> Option<Ordering> {
        (**self).partial_cmp(*other)
    }
}

impl ToOwned for Gc {
    type Owned = GcBuf;
    fn to_owned(&self) -> Self::Owned {
        unsafe {
            GcBuf::from_string_unchecked(self.0.to_owned())
        }
    }
}

/**
An owned, single Unicode grapheme cluster (akin to `String`).

See [`Gc`](struct.Gc.html) for more details.
*/
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct GcBuf(Box<str>);

impl GcBuf {
    /**
    Create a new `GcBuf` from the given `String`.

    This function *does not* check to ensure the provided string is a single, valid grapheme cluster.
    */
    pub unsafe fn from_string_unchecked(s: String) -> GcBuf {
        GcBuf(s.into_boxed_str())
    }

    /**
    Returns a borrowed grapheme cluster slice.
    */
    pub fn as_gc(&self) -> &Gc {
        unsafe {
            Gc::from_str_unchecked(&self.0)
        }
    }
}

impl AsRef<Gc> for GcBuf {
    fn as_ref(&self) -> &Gc {
        self.as_gc()
    }
}

impl AsRef<str> for GcBuf {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl AsRef<[u8]> for GcBuf {
    fn as_ref(&self) -> &[u8] {
        self.as_str().as_bytes()
    }
}

impl Borrow<Gc> for GcBuf {
    fn borrow(&self) -> &Gc {
        self.as_gc()
    }
}

impl Debug for GcBuf {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        Debug::fmt(&self.0, fmt)
    }
}

impl Default for GcBuf {
    fn default() -> Self {
        unsafe {
            GcBuf::from_string_unchecked(String::from("\u{0}"))
        }
    }
}

impl Deref for GcBuf {
    type Target = Gc;
    fn deref(&self) -> &Gc {
        self.as_gc()
    }
}

impl Display for GcBuf {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(&self.0, fmt)
    }
}

impl<'a> From<&'a Gc> for GcBuf {
    fn from(v: &'a Gc) -> Self {
        unsafe {
            GcBuf::from_string_unchecked(v.as_str().to_owned())
        }
    }
}

impl From<char> for GcBuf {
    fn from(v: char) -> Self {
        unsafe {
            let mut buf = [0; 4];
            let bs = match ::util::encode_utf8_raw(v as u32, &mut buf) {
                Some(len) => {
                    if len < 4 {
                        &buf[..len]
                    } else {
                        debug_unreachable!();
                    }
                },
                None => debug_unreachable!(),
            };
            let s: &str = transmute(bs);
            let s = s.to_owned();
            GcBuf::from_string_unchecked(s)
        }
    }
}

impl Into<Box<str>> for GcBuf {
    fn into(self) -> Box<str> {
        self.0
    }
}

impl Into<String> for GcBuf {
    fn into(self) -> String {
        self.0.into_string()
    }
}

impl Into<Vec<u8>> for GcBuf {
    fn into(self) -> Vec<u8> {
        self.0.into_string().into()
    }
}

macro_rules! as_item {
    ($i:item) => { $i };
}

macro_rules! forward_partial_eq {
    (~ <$lt:tt> $lhs:ty, $rhs:ty) => {
        as_item! {
            impl<$lt> PartialEq<$rhs> for $lhs {
                fn eq(&self, other: &$rhs) -> bool {
                    other.as_gc().eq(self)
                }
            }
        }
    };

    (~ $lhs:ty, $rhs:ty) => {
        impl PartialEq<$rhs> for $lhs {
            fn eq(&self, other: &$rhs) -> bool {
                other.as_gc().eq(self)
            }
        }
    };

    (<$lt:tt> $lhs:ty, $rhs:ty) => {
        as_item! {
            impl<$lt> PartialEq<$rhs> for $lhs {
                fn eq(&self, other: &$rhs) -> bool {
                    self.as_gc().eq(other)
                }
            }
        }
    };

    ($lhs:ty, $rhs:ty) => {
        impl PartialEq<$rhs> for $lhs {
            fn eq(&self, other: &$rhs) -> bool {
                self.as_gc().eq(other)
            }
        }
    };
}

forward_partial_eq! { GcBuf, char }
forward_partial_eq! { GcBuf, str }
forward_partial_eq! { GcBuf, Gc }
forward_partial_eq! { GcBuf, String }
forward_partial_eq! { <'a> GcBuf, &'a str }
forward_partial_eq! { <'a> GcBuf, &'a Gc }
forward_partial_eq! { <'a> GcBuf, Cow<'a, Gc> }

forward_partial_eq! { ~ char, GcBuf }
forward_partial_eq! { ~ str, GcBuf }
forward_partial_eq! { ~ String, GcBuf }
forward_partial_eq! { ~ <'a> &'a str, GcBuf }
forward_partial_eq! { ~ <'a> Cow<'a, Gc>, GcBuf }

macro_rules! forward_partial_ord {
    (~ <$lt:tt> $lhs:ty, $rhs:ty) => {
        as_item! {
            impl<$lt> PartialOrd<$rhs> for $lhs {
                fn partial_cmp(&self, other: &$rhs) -> Option<Ordering> {
                    other.as_gc().partial_cmp(self).map(Ordering::reverse)
                }
            }
        }
    };

    (~ $lhs:ty, $rhs:ty) => {
        impl PartialOrd<$rhs> for $lhs {
            fn partial_cmp(&self, other: &$rhs) -> Option<Ordering> {
                other.as_gc().partial_cmp(self).map(Ordering::reverse)
            }
        }
    };

    (<$lt:tt> $lhs:ty, $rhs:ty) => {
        as_item! {
            impl<$lt> PartialOrd<$rhs> for $lhs {
                fn partial_cmp(&self, other: &$rhs) -> Option<Ordering> {
                    self.as_gc().partial_cmp(other)
                }
            }
        }
    };

    ($lhs:ty, $rhs:ty) => {
        impl PartialOrd<$rhs> for $lhs {
            fn partial_cmp(&self, other: &$rhs) -> Option<Ordering> {
                self.as_gc().partial_cmp(other)
            }
        }
    };
}

forward_partial_ord! { GcBuf, char }
forward_partial_ord! { GcBuf, str }
forward_partial_ord! { GcBuf, Gc }
forward_partial_ord! { GcBuf, String }
forward_partial_ord! { <'a> GcBuf, &'a str }
forward_partial_ord! { <'a> GcBuf, &'a Gc }
forward_partial_ord! { <'a> GcBuf, Cow<'a, Gc> }

forward_partial_ord! { ~ char, GcBuf }
forward_partial_ord! { ~ str, GcBuf }
forward_partial_ord! { ~ String, GcBuf }
forward_partial_ord! { ~ <'a> &'a str, GcBuf }
forward_partial_ord! { ~ <'a> Cow<'a, Gc>, GcBuf }
