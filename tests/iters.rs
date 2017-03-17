/*
Copyright ⓒ 2017 Daniel Keep.

Licensed under the MIT license (see LICENSE or <http://opensource.org
/licenses/MIT>) or the Apache License, Version 2.0 (see LICENSE of
<http://www.apache.org/licenses/LICENSE-2.0>), at your option. All
files in the project carrying such notice may not be copied, modified,
or distributed except according to those terms.
*/
extern crate strcursor;

use strcursor::StrCursor;

#[test]
fn test_iter_before() {
    let s = "a黒café🍵!";
    let cur = StrCursor::new_at_end(s);
    let r: Vec<_> = cur.iter_before()
        .map(|gc| gc.as_str()).collect();
    assert_eq!(&*r, &[
        "!",
        "🍵",
        "é",
        "f",
        "a",
        "c",
        "黒",
        "a",
    ]);
}

#[test]
fn test_iter_before_cur() {
    let s = "a黒café🍵!";
    let cur = StrCursor::new_at_end(s);
    let r: Vec<_> = cur.iter_before().with_cursor()
        .map(|(gc, cur)| (gc.as_str(), cur.byte_pos())).collect();
    assert_eq!(&*r, &[
        ("!", 14),
        ("🍵", 10),
        ("é", 7),
        ("f", 6),
        ("a", 5),
        ("c", 4),
        ("黒", 1),
        ("a", 0),
    ]);
}

#[test]
fn test_iter_after() {
    let s = "a黒café🍵!";
    let cur = StrCursor::new_at_start(s);
    let r: Vec<_> = cur.iter_after()
        .map(|gc| gc.as_str()).collect();
    assert_eq!(&*r, &[
        "a",
        "黒",
        "c",
        "a",
        "f",
        "é",
        "🍵",
        "!",
    ]);
}

#[test]
fn test_iter_after_cur() {
    let s = "a黒café🍵!";
    let cur = StrCursor::new_at_start(s);
    let r: Vec<_> = cur.iter_after().with_cursor()
        .map(|(gc, cur)| (gc.as_str(), cur.byte_pos())).collect();
    assert_eq!(&*r, &[
        ("a", 1),
        ("黒", 4),
        ("c", 5),
        ("a", 6),
        ("f", 7),
        ("é", 10),
        ("🍵", 14),
        ("!", 15),
    ]);
}

#[test]
fn test_iter_cp_before() {
    let s = "a黒café🍵!";
    let cur = StrCursor::new_at_end(s);
    let r: Vec<_> = cur.iter_cp_before()
        .collect();
    assert_eq!(&*r, &[
        '!',
        '🍵',
        '́',
        'e',
        'f',
        'a',
        'c',
        '黒',
        'a',
    ]);
}

#[test]
fn test_iter_cp_before_cur() {
    let s = "a黒café🍵!";
    let cur = StrCursor::new_at_end(s);
    let r: Vec<_> = cur.iter_cp_before().with_cursor()
        .map(|(cp, cur)| (cp, cur.byte_pos())).collect();
    assert_eq!(&*r, &[
        ('!', 14),
        ('🍵', 10),
        ('́', 8),
        ('e', 7),
        ('f', 6),
        ('a', 5),
        ('c', 4),
        ('黒', 1),
        ('a', 0),
    ]);
}

#[test]
fn test_iter_cp_after() {
    let s = "a黒café🍵!";
    let cur = StrCursor::new_at_start(s);
    let r: Vec<_> = cur.iter_cp_after()
        .collect();
    assert_eq!(&*r, &[
        'a',
        '黒',
        'c',
        'a',
        'f',
        'e',
        '́',
        '🍵',
        '!',
    ]);
}

#[test]
fn test_iter_cp_after_cur() {
    let s = "a黒café🍵!";
    let cur = StrCursor::new_at_start(s);
    let r: Vec<_> = cur.iter_cp_after().with_cursor()
        .map(|(cp, cur)| (cp, cur.byte_pos())).collect();
    assert_eq!(&*r, &[
        ('a', 1),
        ('黒', 4),
        ('c', 5),
        ('a', 6),
        ('f', 7),
        ('e', 8),
        ('́', 10),
        ('🍵', 14),
        ('!', 15),
    ]);
}
