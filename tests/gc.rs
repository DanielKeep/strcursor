/*
Copyright ⓒ 2015-2017 Daniel Keep.

Licensed under the MIT license (see LICENSE or <http://opensource.org
/licenses/MIT>) or the Apache License, Version 2.0 (see LICENSE of
<http://www.apache.org/licenses/LICENSE-2.0>), at your option. All
files in the project carrying such notice may not be copied, modified,
or distributed except according to those terms.
*/
extern crate strcursor;

use strcursor::Gc;

fn gc(s: &str) -> &Gc {
    Gc::from_str(s).unwrap()
}

#[test]
fn test_from_str() {
    assert_eq!(Gc::from_str("a").map(Gc::as_str), Some("a"));
    assert_eq!(Gc::from_str("á").map(Gc::as_str), Some("á"));
    assert_eq!(Gc::from_str("ä").map(Gc::as_str), Some("ä"));
    assert_eq!(Gc::from_str("̈").map(Gc::as_str), Some("̈")); // NB: there is a single combining diaereses in the string.
    assert_eq!(Gc::from_str("字").map(Gc::as_str), Some("字"));
    assert_eq!(Gc::from_str("").map(Gc::as_str), None);
    assert_eq!(Gc::from_str("ab").map(Gc::as_str), None);
}

#[test]
fn test_split_from() {
    fn map<'a>((gr, s): (&'a Gc, &'a str)) -> (&'a str, &'a str) {
        (gr.as_str(), s)
    }

    assert_eq!(Gc::split_from("a").map(map), Some(("a", "")));
    assert_eq!(Gc::split_from("á").map(map), Some(("á", "")));
    assert_eq!(Gc::split_from("ä").map(map), Some(("ä", "")));
    assert_eq!(Gc::split_from("̈").map(map), Some(("̈", ""))); // NB: there is a single combining diaereses in the string.
    assert_eq!(Gc::split_from("字").map(map), Some(("字", "")));
    assert_eq!(Gc::split_from("").map(map), None);
    assert_eq!(Gc::split_from("ab").map(map), Some(("a", "b")));
}

#[test]
fn test_has_marks() {
    assert!(!gc("a").has_marks());
    assert!(!gc("á").has_marks());
    assert!(gc("ä").has_marks());
    assert!(!gc("̈").has_marks());
    assert!(!gc("字").has_marks());
}

#[test]
fn test_base_char() {
    assert_eq!(gc("a").base_char(), 'a');
    assert_eq!(gc("á").base_char(), 'á');
    assert_eq!(gc("ä").base_char(), 'a');
    assert_eq!(gc("̈").base_char(), '̈');
    assert_eq!(gc("字").base_char(), '字');
}

#[test]
fn test_mark_str() {
    assert_eq!(gc("a").mark_str(), "");
    assert_eq!(gc("á").mark_str(), "");
    assert_eq!(gc("ä").mark_str(), "̈");
    assert_eq!(gc("̈").mark_str(), "");
    assert_eq!(gc("字").mark_str(), "");
}
