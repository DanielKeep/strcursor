// use std::marker::PhantomData;
// use std::slice;

pub struct StrCursor<'a> {
    s: &'a str,
    at: *const u8,
}

impl<'a> StrCursor<'a> {
    pub fn new_at_start(s: &'a str) -> StrCursor<'a> {
        StrCursor {
            s: s,
            at: s.as_ptr(),
        }
    }

    pub fn new_at_end(s: &'a str) -> StrCursor<'a> {
        unsafe {
            StrCursor {
                s: s,
                at: s.as_ptr().offset(s.len() as isize),
            }
        }
    }

    pub fn new_left_of_byte_pos(s: &'a str, byte_pos: usize) -> StrCursor<'a> {
        unsafe {
            StrCursor {
                s: s,
                at: seek_utf8_cp_start_left(s, &s.as_bytes()[byte_pos]),
            }
        }
    }

    pub fn new_right_of_byte_pos(s: &'a str, byte_pos: usize) -> StrCursor<'a> {
        unsafe {
            StrCursor {
                s: s,
                at: seek_utf8_cp_start_right(s, &s.as_bytes()[byte_pos]),
            }
        }
    }

    pub fn at_prev_cp(mut self) -> Option<StrCursor<'a>> {
        match self.try_seek_left() {
            true => Some(self),
            false => None
        }
    }

    pub fn at_next_cp(mut self) -> Option<StrCursor<'a>> {
        match self.try_seek_right() {
            true => Some(self),
            false => None
        }
    }

    pub fn seek_prev_cp(&mut self) {
        if !self.try_seek_left() {
            panic!("cannot seek past the beginning of a string");
        }
    }

    pub fn seek_next_cp(&mut self) {
        if !self.try_seek_right() {
            panic!("cannot seek past the end of a string");
        }
    }

    pub fn slice_before(&self) -> &str {
        unsafe {
            self.s.slice_unchecked(0, self.byte_pos())
        }
    }

    pub fn slice_after(&self) -> &str {
        unsafe {
            self.s.slice_unchecked(self.byte_pos(), self.s.len())
        }
    }
    
    pub fn char_before(&self) -> Option<char> {
        self.at_prev_cp().and_then(|cur| cur.char_after())
    }
    
    pub fn char_after(&self) -> Option<char> {
        self.slice_after().chars().next()
    }

    pub fn byte_pos(&self) -> usize {
        self.at as usize - self.s.as_ptr() as usize
    }

    fn try_seek_left(&mut self) -> bool {
        unsafe {
            // We just have to ensure that offsetting the `at` pointer *at all* is safe.
            if self.byte_pos() == 0 {
                return false;
            }
            self.at = seek_utf8_cp_start_left(self.s, self.at.offset(-1));
            true
        }
    }

    fn try_seek_right(&mut self) -> bool {
        unsafe {
            // We just have to ensure that offsetting the `at` pointer *at all* is safe.
            if self.byte_pos() == self.s.len() {
                return false;
            }
            self.at = seek_utf8_cp_start_right(self.s, self.at.offset(1));
            true
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
fn test_new_left_of_byte_pos() {
    let s = "This is a 本当 test.";
    let cur = StrCursor::new_left_of_byte_pos(s, 11);
    assert_eq!(cur.slice_before(), "This is a ");
    assert_eq!(cur.slice_after(), "本当 test.");
}

#[cfg(test)]
#[test]
fn test_new_right_of_byte_pos() {
    let s = "This is a 本当 test.";
    let cur = StrCursor::new_right_of_byte_pos(s, 11);
    assert_eq!(cur.slice_before(), "This is a 本");
    assert_eq!(cur.slice_after(), "当 test.");
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
fn test_char_before_and_after() {
    let s = "大嫌い,💪❤";
    let cur = StrCursor::new_at_start(s);
    let r = test_util::finite_iterate_lead(cur, StrCursor::at_next_cp)
        .map(|cur| (cur.byte_pos(), cur.char_before(), cur.char_after()))
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