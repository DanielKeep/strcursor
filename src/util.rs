/*
Copyright ⓒ 2015-2017 Daniel Keep.

Licensed under the MIT license (see LICENSE or <http://opensource.org
/licenses/MIT>) or the Apache License, Version 2.0 (see LICENSE of
<http://www.apache.org/licenses/LICENSE-2.0>), at your option. All
files in the project carrying such notice may not be copied, modified,
or distributed except according to those terms.
*/
/*!
Miscellaneous stuff.
*/

/**
Turns a string, and a byte position within that string, into a raw pointer pointing to that byte position.

This function performs bounds checking to ensure the pointer is valid.  However, if the byte position is equal to the length of the string (*i.e.* it points to the end of the string), the resulting pointer is within bounds, but *not* safe to dereference.

This function *does not* ensure the byte position is at a UTF-8 code point boundary.
*/
#[inline]
pub fn byte_pos_to_ptr(s: &str, byte_pos: usize) -> *const u8 {
    if s.len() < byte_pos {
        panic!("byte position out of bounds: the len is {} but the position is {}",
            s.len(), byte_pos);
    }
    unsafe { s.as_ptr().offset(byte_pos as isize) }
}

/**
Seeks the given pointer left to the first code point boundary.

Assuming `from` is a pointer into `s`, and `s` is not empty, the resulting pointer will lie on a valid UTF-8 code point boundary.

If `from` already points to a code point boundary, it is returned unchanged.
*/
#[inline]
pub unsafe fn seek_utf8_cp_start_left(s: &str, mut from: *const u8) -> *const u8 {
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

/**
Seeks the given pointer right to the first code point boundary.

Assuming `from` is a pointer into `s`, and `s` is not empty, the resulting pointer will lie on a valid UTF-8 code point boundary.  Note that this *includes* the very end of the string, immediately after the last byte in the string.

If `from` already points to a code point boundary, it is returned unchanged.
*/
#[inline]
pub unsafe fn seek_utf8_cp_start_right(s: &str, mut from: *const u8) -> *const u8 {
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

/**
Compares two strings for exact pointer identity.

That is, this only returns `true` if the two strings point to the same location in memory, and are of the same length.
*/
#[inline]
pub fn str_eq_literal(a: &str, b: &str) -> bool {
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

/*

TODO: The following code is nicked from libcore, owing to `encode_utf8` not being stable yet.  Specifically, <https://github.com/rust-lang/rust/blob/3d7cd77e442ce34eaac8a176ae8be17669498ebc/src/libcore/char.rs>.

`encode_utf8` is stable as of Rust 1.15.  However, the minimum version is 1.7, so this needs to hang around until the minimum version is at least 1.15.

*/

// Copyright 2012-2014 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

// UTF-8 ranges and tags for encoding characters
const TAG_CONT: u8    = 0b1000_0000;
const TAG_TWO_B: u8   = 0b1100_0000;
const TAG_THREE_B: u8 = 0b1110_0000;
const TAG_FOUR_B: u8  = 0b1111_0000;
const MAX_ONE_B: u32   =     0x80;
const MAX_TWO_B: u32   =    0x800;
const MAX_THREE_B: u32 =  0x10000;

pub fn encode_utf8_raw(code: u32, dst: &mut [u8]) -> Option<usize> {
    // Marked #[inline] to allow llvm optimizing it away
    if code < MAX_ONE_B && !dst.is_empty() {
        dst[0] = code as u8;
        Some(1)
    } else if code < MAX_TWO_B && dst.len() >= 2 {
        dst[0] = (code >> 6 & 0x1F) as u8 | TAG_TWO_B;
        dst[1] = (code & 0x3F) as u8 | TAG_CONT;
        Some(2)
    } else if code < MAX_THREE_B && dst.len() >= 3  {
        dst[0] = (code >> 12 & 0x0F) as u8 | TAG_THREE_B;
        dst[1] = (code >>  6 & 0x3F) as u8 | TAG_CONT;
        dst[2] = (code & 0x3F) as u8 | TAG_CONT;
        Some(3)
    } else if dst.len() >= 4 {
        dst[0] = (code >> 18 & 0x07) as u8 | TAG_FOUR_B;
        dst[1] = (code >> 12 & 0x3F) as u8 | TAG_CONT;
        dst[2] = (code >>  6 & 0x3F) as u8 | TAG_CONT;
        dst[3] = (code & 0x3F) as u8 | TAG_CONT;
        Some(4)
    } else {
        None
    }
}
