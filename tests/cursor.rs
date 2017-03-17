/*
Copyright ⓒ 2015-2017 Daniel Keep.

Licensed under the MIT license (see LICENSE or <http://opensource.org
/licenses/MIT>) or the Apache License, Version 2.0 (see LICENSE of
<http://www.apache.org/licenses/LICENSE-2.0>), at your option. All
files in the project carrying such notice may not be copied, modified,
or distributed except according to those terms.
*/
extern crate strcursor;

mod util;

use strcursor::{Gc, StrCursor};

#[test]
fn test_new_at_start() {
    let cur = StrCursor::new_at_start("abcdef");
    assert_eq!(cur.slice_before(), "");
    assert_eq!(cur.slice_after(), "abcdef");
}

#[test]
fn test_new_at_end() {
    let cur = StrCursor::new_at_end("abcdef");
    assert_eq!(cur.slice_before(), "abcdef");
    assert_eq!(cur.slice_after(), "");
}

#[test]
fn test_new_at_cp_left_of_byte_pos() {
    let s = "This is a 本当 test.";
    let cur = StrCursor::new_at_cp_left_of_byte_pos(s, 11);
    assert_eq!(cur.slice_before(), "This is a ");
    assert_eq!(cur.slice_after(), "本当 test.");
}

#[test]
fn test_new_at_cp_right_of_byte_pos() {
    let s = "This is a 本当 test.";
    let cur = StrCursor::new_at_cp_right_of_byte_pos(s, 11);
    assert_eq!(cur.slice_before(), "This is a 本");
    assert_eq!(cur.slice_after(), "当 test.");
}

#[test]
fn test_new_at_left_of_byte_pos() {
    let s = "Jäger,Jäger,大嫌い,💪❤!";
    let r = (0..s.len()+1).map(|i| (i, StrCursor::new_at_left_of_byte_pos(s, i)))
        .map(|(i, cur)| (i, cur.byte_pos(), cur.after().map(Gc::as_str)))
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

#[test]
fn test_new_at_right_of_byte_pos() {
    let s = "Jäger,Jäger,大嫌い,💪❤!";
    let r = (0..s.len()+1).map(|i| (i, StrCursor::new_at_right_of_byte_pos(s, i)))
        .map(|(i, cur)| (i, cur.byte_pos(), cur.after().map(Gc::as_str)))
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

#[test]
fn test_at_prev_cp() {
    let s = "大嫌い,💪❤";
    let cur = StrCursor::new_at_end(s);
    let bps = util::finite_iterate(cur, StrCursor::at_prev_cp)
        .map(|cur| cur.byte_pos())
        .collect::<Vec<_>>();
    assert_eq!(bps, vec![14, 10, 9, 6, 3, 0]);
}

#[test]
fn test_at_next_cp() {
    let s = "大嫌い,💪❤";
    let cur = StrCursor::new_at_start(s);
    let bps = util::finite_iterate(cur, StrCursor::at_next_cp)
        .map(|cur| cur.byte_pos())
        .collect::<Vec<_>>();
    assert_eq!(bps, vec![3, 6, 9, 10, 14, 17]);
}

#[test]
fn test_at_prev_and_before() {
    let s = "noe\u{0308}l";
    let cur = StrCursor::new_at_end(s);
    let bps = util::finite_iterate_lead(cur, StrCursor::at_prev)
        .map(|cur| (cur.byte_pos(), cur.after().map(Gc::as_str)))
        .collect::<Vec<_>>();
    assert_eq!(bps, vec![
        (6, None),
        (5, Some("l")),
        (2, Some("e\u{0308}")),
        (1, Some("o")),
        (0, Some("n")),
    ]);
}

#[test]
fn test_at_next_and_after() {
    let s = "noe\u{0308}l";
    let cur = StrCursor::new_at_start(s);
    let bps = util::finite_iterate_lead(cur, StrCursor::at_next)
        .map(|cur| (cur.byte_pos(), cur.after().map(Gc::as_str)))
        .collect::<Vec<_>>();
    assert_eq!(bps, vec![
        (0, Some("n")),
        (1, Some("o")),
        (2, Some("e\u{0308}")),
        (5, Some("l")),
        (6, None),
    ]);
}

#[test]
fn test_prev() {
    let s = "Jäger,Jäger,大嫌い,💪❤!";
    let cur = StrCursor::new_at_end(s);
    let r = util::finite_iterate_lead(cur, StrCursor::at_prev)
        .map(|cur| cur.prev().map(|(gr, cur)| (gr.as_str(), cur.byte_pos())))
        .collect::<Vec<_>>();
    assert_eq!(r, vec![
        Some(("!", 32)),
        Some(("❤", 29)),
        Some(("💪", 25)),
        Some((",", 24)),
        Some(("い", 21)),
        Some(("嫌", 18)),
        Some(("大", 15)),
        Some((",", 14)),
        Some(("r", 13)),
        Some(("e", 12)),
        Some(("g", 11)),
        Some(("ä", 8)),
        Some(("J", 7)),
        Some((",", 6)),
        Some(("r", 5)),
        Some(("e", 4)),
        Some(("g", 3)),
        Some(("ä", 1)),
        Some(("J", 0)),
        None,
    ]);
}

#[test]
fn test_prev_cp() {
    let s = "Jäger,Jäger,大嫌い,💪❤!";
    let cur = StrCursor::new_at_end(s);
    let r = util::finite_iterate_lead(cur, StrCursor::at_prev_cp)
        .map(|cur| cur.prev_cp().map(|(cp, cur)| (cp, cur.byte_pos())))
        .collect::<Vec<_>>();
    assert_eq!(r, vec![
        Some(('!', 32)),
        Some(('❤', 29)),
        Some(('💪', 25)),
        Some((',', 24)),
        Some(('い', 21)),
        Some(('嫌', 18)),
        Some(('大', 15)),
        Some((',', 14)),
        Some(('r', 13)),
        Some(('e', 12)),
        Some(('g', 11)),
        Some(('̈', 9)),
        Some(('a', 8)),
        Some(('J', 7)),
        Some((',', 6)),
        Some(('r', 5)),
        Some(('e', 4)),
        Some(('g', 3)),
        Some(('ä', 1)),
        Some(('J', 0)),
        None,
    ]);
}

#[test]
fn test_next() {
    let s = "Jäger,Jäger,大嫌い,💪❤!";
    let cur = StrCursor::new_at_start(s);
    let r = util::finite_iterate_lead(cur, StrCursor::at_next)
        .map(|cur| cur.next().map(|(gr, cur)| (gr.as_str(), cur.byte_pos())))
        .collect::<Vec<_>>();
    assert_eq!(r, vec![
        Some(("J", 1)),
        Some(("ä", 3)),
        Some(("g", 4)),
        Some(("e", 5)),
        Some(("r", 6)),
        Some((",", 7)),
        Some(("J", 8)),
        Some(("ä", 11)),
        Some(("g", 12)),
        Some(("e", 13)),
        Some(("r", 14)),
        Some((",", 15)),
        Some(("大", 18)),
        Some(("嫌", 21)),
        Some(("い", 24)),
        Some((",", 25)),
        Some(("💪", 29)),
        Some(("❤", 32)),
        Some(("!", 33)),
        None,
    ]);
}

#[test]
fn test_next_cp() {
    let s = "Jäger,Jäger,大嫌い,💪❤!";
    let cur = StrCursor::new_at_start(s);
    let r = util::finite_iterate_lead(cur, StrCursor::at_next_cp)
        .map(|cur| cur.next_cp().map(|(cp, cur)| (cp, cur.byte_pos())))
        .collect::<Vec<_>>();
    assert_eq!(r, vec![
        Some(('J', 1)),
        Some(('ä', 3)),
        Some(('g', 4)),
        Some(('e', 5)),
        Some(('r', 6)),
        Some((',', 7)),
        Some(('J', 8)),
        Some(('a', 9)),
        Some(('̈', 11)),
        Some(('g', 12)),
        Some(('e', 13)),
        Some(('r', 14)),
        Some((',', 15)),
        Some(('大', 18)),
        Some(('嫌', 21)),
        Some(('い', 24)),
        Some((',', 25)),
        Some(('💪', 29)),
        Some(('❤', 32)),
        Some(('!', 33)),
        None,
    ]);
}

#[test]
fn test_seek_prev() {
    let s = "Jäger,Jäger,大嫌い,💪❤!";
    let mut cur = StrCursor::new_at_end(s);
    let mut r = vec![];
    for i in 0..19 {
        println!("i: {:?}", i);
        println!("cur.byte_pos(): {:?}", cur.byte_pos());
        cur.seek_prev();
        r.push((cur.after().unwrap().as_str(), cur.byte_pos()));
    }
    assert_eq!(r, vec![
        ("!", 32),
        ("❤", 29),
        ("💪", 25),
        (",", 24),
        ("い", 21),
        ("嫌", 18),
        ("大", 15),
        (",", 14),
        ("r", 13),
        ("e", 12),
        ("g", 11),
        ("ä", 8),
        ("J", 7),
        (",", 6),
        ("r", 5),
        ("e", 4),
        ("g", 3),
        ("ä", 1),
        ("J", 0),
    ]);
}

#[test]
#[should_panic]
fn test_seek_prev_panic() {
    let s = "Jäger,Jäger,大嫌い,💪❤!";
    let mut cur = StrCursor::new_at_start(s);
    cur.seek_prev();
}

#[test]
fn test_seek_prev_cp() {
    let s = "Jäger,Jäger,大嫌い,💪❤!";
    let mut cur = StrCursor::new_at_end(s);
    let mut r = vec![];
    for _ in 0..20 {
        cur.seek_prev_cp();
        r.push((cur.cp_after().unwrap(), cur.byte_pos()));
    }
    assert_eq!(r, vec![
        ('!', 32),
        ('❤', 29),
        ('💪', 25),
        (',', 24),
        ('い', 21),
        ('嫌', 18),
        ('大', 15),
        (',', 14),
        ('r', 13),
        ('e', 12),
        ('g', 11),
        ('̈', 9),
        ('a', 8),
        ('J', 7),
        (',', 6),
        ('r', 5),
        ('e', 4),
        ('g', 3),
        ('ä', 1),
        ('J', 0),
    ]);
}

#[test]
#[should_panic]
fn test_seek_prev_cp_panic() {
    let s = "Jäger,Jäger,大嫌い,💪❤!";
    let mut cur = StrCursor::new_at_start(s);
    cur.seek_prev_cp();
}

#[test]
fn test_seek_next() {
    let s = "Jäger,Jäger,大嫌い,💪❤!";
    let mut cur = StrCursor::new_at_start(s);
    let mut r = vec![];
    for _ in 0..19 {
        cur.seek_next();
        r.push((cur.before().unwrap().as_str(), cur.byte_pos()));
    }
    assert_eq!(r, vec![
        ("J", 1),
        ("ä", 3),
        ("g", 4),
        ("e", 5),
        ("r", 6),
        (",", 7),
        ("J", 8),
        ("ä", 11),
        ("g", 12),
        ("e", 13),
        ("r", 14),
        (",", 15),
        ("大", 18),
        ("嫌", 21),
        ("い", 24),
        (",", 25),
        ("💪", 29),
        ("❤", 32),
        ("!", 33),
    ]);
}

#[test]
#[should_panic]
fn test_seek_next_panic() {
    let s = "Jäger,Jäger,大嫌い,💪❤!";
    let mut cur = StrCursor::new_at_end(s);
    cur.seek_next();
}

#[test]
fn test_seek_next_cp() {
    let s = "Jäger,Jäger,大嫌い,💪❤!";
    let mut cur = StrCursor::new_at_start(s);
    let mut r = vec![];
    for _ in 0..20 {
        cur.seek_next_cp();
        r.push((cur.cp_before().unwrap(), cur.byte_pos()));
    }
    assert_eq!(r, vec![
        ('J', 1),
        ('ä', 3),
        ('g', 4),
        ('e', 5),
        ('r', 6),
        (',', 7),
        ('J', 8),
        ('a', 9),
        ('̈', 11),
        ('g', 12),
        ('e', 13),
        ('r', 14),
        (',', 15),
        ('大', 18),
        ('嫌', 21),
        ('い', 24),
        (',', 25),
        ('💪', 29),
        ('❤', 32),
        ('!', 33),
    ]);
}

#[test]
#[should_panic]
fn test_seek_next_cp_panic() {
    let s = "Jäger,Jäger,大嫌い,💪❤!";
    let mut cur = StrCursor::new_at_end(s);
    cur.seek_next_cp();
}

#[test]
fn test_char_before_and_after() {
    let s = "大嫌い,💪❤";
    let cur = StrCursor::new_at_start(s);
    let r = util::finite_iterate_lead(cur, StrCursor::at_next_cp)
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

#[test]
fn test_slice_until() {
    let s = "they hit, fight, kick, wreak havoc, and rejoice";
    let cur0 = StrCursor::new_at_start(s);
    let cur1 = StrCursor::new_at_end(s);
    let cur2 = StrCursor::new_at_end("nobody knows what they're lookin' for");
    let cur3 = StrCursor::new_at_end(&s[1..]);
    assert_eq!(cur0.slice_until(cur1), Some(s));
    assert_eq!(cur1.slice_until(cur0), Some(""));
    assert_eq!(cur0.slice_until(cur2), None);
    assert_eq!(cur0.slice_until(cur3), None);
}
