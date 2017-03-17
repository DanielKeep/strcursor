/*
Copyright ⓒ 2017 Daniel Keep.

Licensed under the MIT license (see LICENSE or <http://opensource.org
/licenses/MIT>) or the Apache License, Version 2.0 (see LICENSE of
<http://www.apache.org/licenses/LICENSE-2.0>), at your option. All
files in the project carrying such notice may not be copied, modified,
or distributed except according to those terms.
*/
/*!
Tests for Unicode corner cases.
*/

extern crate strcursor;

use strcursor::StrCursor;

const GR_TEST_STR: &'static str = concat!(
    "a", "b̂", "ċ̲",
    "か", "き̂", "く̲̇",
    "𐌰", "𐌱̂", "𐌲̲̇",
    "𝅝", "𝅗𝅥𝅮", "𝅘𝅥𝅯𝆀",
    "👨", "👨‍👩", "👨‍👩‍👧",
);

/**
Are the grapheme cluster breaks where we expect them to be?
*/
#[test]
fn test_gr_seg() {
    let cur = StrCursor::new_at_start(GR_TEST_STR);
    let r: Vec<_> = cur.iter_after().with_cursor()
        .map(|(gc, cur)| (gc.as_str(), cur.byte_pos()))
        .collect();
    assert_eq!(
        &*r,
        &[
            ("a", 1),
            ("b̂", 4),
            ("ċ̲", 9),
            ("か", 12),
            ("き̂", 17),
            ("く̲̇", 24),
            ("𐌰", 28),
            ("𐌱̂", 34),
            ("𐌲̲̇", 42),
            ("𝅝", 46),
            ("𝅗𝅥𝅮", 54),
            ("𝅘𝅥𝅯𝆀", 66),
            ("👨", 70),
            ("👨‍👩", 81),
            ("👨‍👩‍👧", 99),
        ]
    );
}
