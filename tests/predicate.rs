extern crate strcursor;

use strcursor::StrCursor;

#[test]
fn test_before_while() {
    let s = "chuu  ͢chuu ͢ yeah";
    let cur = StrCursor::new_at_left_of_byte_pos(s, 10);
    let r = cur.before_while(|gc| !gc.is_base(char::is_whitespace));
    assert_eq!(
        r,
        (" ͢ch", StrCursor::new_at_left_of_byte_pos(s, 5))
    );
}

#[test]
fn test_after_while() {
    let s = "chuu  ͢chuu ͢ yeah";
    let cur = StrCursor::new_at_left_of_byte_pos(s, 10);
    let r = cur.after_while(|gc| !gc.is_base(char::is_whitespace));
    assert_eq!(
        r,
        ("uu ͢", StrCursor::new_at_left_of_byte_pos(s, 15))
    );
}

#[test]
fn test_cp_before_while() {
    let s = "chuu  ͢chuu ͢ yeah";
    let cur = StrCursor::new_at_left_of_byte_pos(s, 10);
    let r = cur.cp_before_while(|cp| !cp.is_whitespace());
    assert_eq!(
        r,
        ("͢ch", StrCursor::new_at_cp_left_of_byte_pos(s, 6))
    );
}

#[test]
fn test_cp_after_while() {
    let s = "chuu  ͢chuu ͢ yeah";
    let cur = StrCursor::new_at_left_of_byte_pos(s, 10);
    let r = cur.cp_after_while(|cp| !cp.is_whitespace());
    assert_eq!(
        r,
        ("uu", StrCursor::new_at_left_of_byte_pos(s, 12))
    );
}
