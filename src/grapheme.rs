/*
Copyright ⓒ 2015 Daniel Keep.

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
A slice of a single Unicode grapheme cluster (akin to `str`).
*/
#[derive(Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Gc(str);

impl Gc {
    pub fn from_str(s: &str) -> Option<&Gc> {
        unsafe {
            match UniSeg::graphemes(s, /*is_extended:*/true).next() {
                Some(gr) => Some(Gc::from_str_unchecked(gr)),
                None => None
            }
        }
    }

    pub unsafe fn from_str_unchecked(s: &str) -> &Gc {
        transmute(s)
    }

    pub fn split_from(s: &str) -> Option<(&Gc, &str)> {
        unsafe {
            let gr = match UniSeg::graphemes(s, /*is_extended:*/true).next() {
                Some(gr) => gr,
                None => return None,
            };
            Some((Gc::from_str_unchecked(gr), s.slice_unchecked(gr.len(), s.len())))
        }
    }

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

    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn base_char(&self) -> char {
        unsafe {
            match self.0.chars().next() {
                Some(cp) => cp,
                None => debug_unreachable!(),
            }
        }
    }

    pub fn base(&self) -> &Gc {
        unsafe {
            let base_cp = self.base_char();
            let base_len = base_cp.len_utf8();
            Gc::from_str_unchecked(self.0.slice_unchecked(base_len, self.0.len()))
        }
    }

    pub fn mark_str(&self) -> &str {
        unsafe {
            let base_cp = self.base_char();
            let base_len = base_cp.len_utf8();
            self.0.slice_unchecked(base_len, self.0.len())
        }
    }

    pub fn chars(&self) -> ::std::str::Chars {
        self.0.chars()
    }

    pub fn char_indices(&self) -> ::std::str::CharIndices {
        self.0.char_indices()
    }

    pub fn bytes(&self) -> ::std::str::Bytes {
        self.0.bytes()
    }

    pub fn to_lowercase(&self) -> String {
        self.0.to_lowercase()
    }

    pub fn to_uppercase(&self) -> String {
        self.0.to_uppercase()
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
            GcBuf::unchecked_from_string(self.0.to_owned())
        }
    }
}

/**
An owned, mutable Unicode grapheme cluster (akin to `String`).
*/
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct GcBuf(Box<str>);

impl GcBuf {
    unsafe fn unchecked_from_string(s: String) -> GcBuf {
        GcBuf(s.into_boxed_str())
    }

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
            GcBuf::unchecked_from_string(String::from("\u{0}"))
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
            GcBuf::unchecked_from_string(v.as_str().to_owned())
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
            GcBuf::unchecked_from_string(s)
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
