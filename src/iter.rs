/*
Copyright ⓒ 2017 Daniel Keep.

Licensed under the MIT license (see LICENSE or <http://opensource.org
/licenses/MIT>) or the Apache License, Version 2.0 (see LICENSE of
<http://www.apache.org/licenses/LICENSE-2.0>), at your option. All
files in the project carrying such notice may not be copied, modified,
or distributed except according to those terms.
*/
/*!
Iterator types.
*/
use ::{Gc, StrCursor};

/**
A right-to-left iterator over grapheme clusters.
*/
pub struct IterBefore<'a>(
    /// The current cursor position.
    pub StrCursor<'a>,
);

impl<'a> IterBefore<'a> {
    /**
    Add the post-movement cursor position to the iterator items.
    */
    #[inline]
    pub fn with_cursor(self) -> IterBeforeCursor<'a> {
        IterBeforeCursor(self.0)
    }
}

impl<'a> Iterator for IterBefore<'a> {
    type Item = &'a Gc;

    fn next(&mut self) -> Option<Self::Item> {
        match self.0.prev() {
            Some((gc, cur)) => {
                self.0 = cur;
                Some(gc)
            },
            None => None,
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let l = self.0.slice_before().len();
        if l == 0 {
            (0, Some(0))
        } else {
            (1, Some(l))
        }
    }
}

/**
A right-to-left iterator over grapheme clusters and cursor positions.

The `(&Gc, StrCursor)` pairs emitted are equivalent to calling `StrCursor::prev` on the current position.
*/
pub struct IterBeforeCursor<'a>(
    /// The current cursor position.
    pub StrCursor<'a>,
);

impl<'a> Iterator for IterBeforeCursor<'a> {
    type Item = (&'a Gc, StrCursor<'a>);

    fn next(&mut self) -> Option<Self::Item> {
        match self.0.prev() {
            Some((gc, cur)) => {
                self.0 = cur;
                Some((gc, cur))
            },
            None => None,
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let l = self.0.slice_before().len();
        if l == 0 {
            (0, Some(0))
        } else {
            (1, Some(l))
        }
    }
}

/**
A left-to-right iterator over grapheme clusters.
*/
pub struct IterAfter<'a>(
    /// The current cursor position.
    pub StrCursor<'a>,
);

impl<'a> IterAfter<'a> {
    /**
    Add the post-movement cursor position to the iterator items.
    */
    #[inline]
    pub fn with_cursor(self) -> IterAfterCursor<'a> {
        IterAfterCursor(self.0)
    }
}

impl<'a> Iterator for IterAfter<'a> {
    type Item = &'a Gc;

    fn next(&mut self) -> Option<Self::Item> {
        match self.0.next() {
            Some((gc, cur)) => {
                self.0 = cur;
                Some(gc)
            },
            None => None,
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let l = self.0.slice_after().len();
        if l == 0 {
            (0, Some(0))
        } else {
            (1, Some(l))
        }
    }
}

/**
A left-to-right iterator over grapheme clusters and cursor positions.

The `(&Gc, StrCursor)` pairs emitted are equivalent to calling `StrCursor::next` on the current position.
*/
pub struct IterAfterCursor<'a>(
    /// The current cursor position.
    pub StrCursor<'a>,
);

impl<'a> Iterator for IterAfterCursor<'a> {
    type Item = (&'a Gc, StrCursor<'a>);

    fn next(&mut self) -> Option<Self::Item> {
        match self.0.next() {
            Some((gc, cur)) => {
                self.0 = cur;
                Some((gc, cur))
            },
            None => None,
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let l = self.0.slice_after().len();
        if l == 0 {
            (0, Some(0))
        } else {
            (1, Some(l))
        }
    }
}

/**
A right-to-left iterator over code points.
*/
pub struct IterCpBefore<'a>(
    /// The current cursor position.
    pub StrCursor<'a>,
);

impl<'a> IterCpBefore<'a> {
    /**
    Add the post-movement cursor position to the iterator items.
    */
    #[inline]
    pub fn with_cursor(self) -> IterCpBeforeCursor<'a> {
        IterCpBeforeCursor(self.0)
    }
}

impl<'a> Iterator for IterCpBefore<'a> {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        match self.0.prev_cp() {
            Some((cp, cur)) => {
                self.0 = cur;
                Some(cp)
            },
            None => None,
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let l = self.0.slice_before().len();
        if l == 0 {
            (0, Some(0))
        } else {
            (1, Some(l))
        }
    }
}

/**
A right-to-left iterator over code points and cursor positions.

The `(char, StrCursor)` pairs emitted are equivalent to calling `StrCursor::prev_cp` on the current position.
*/
pub struct IterCpBeforeCursor<'a>(
    /// The current cursor position.
    pub StrCursor<'a>,
);

impl<'a> Iterator for IterCpBeforeCursor<'a> {
    type Item = (char, StrCursor<'a>);

    fn next(&mut self) -> Option<Self::Item> {
        match self.0.prev_cp() {
            Some((cp, cur)) => {
                self.0 = cur;
                Some((cp, cur))
            },
            None => None,
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let l = self.0.slice_before().len();
        if l == 0 {
            (0, Some(0))
        } else {
            (1, Some(l))
        }
    }
}

/**
A left-to-right iterator over code points.
*/
pub struct IterCpAfter<'a>(
    /// The current cursor position.
    pub StrCursor<'a>,
);

impl<'a> IterCpAfter<'a> {
    /**
    Add the post-movement cursor position to the iterator items.
    */
    #[inline]
    pub fn with_cursor(self) -> IterCpAfterCursor<'a> {
        IterCpAfterCursor(self.0)
    }
}

impl<'a> Iterator for IterCpAfter<'a> {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        match self.0.next_cp() {
            Some((cp, cur)) => {
                self.0 = cur;
                Some(cp)
            },
            None => None,
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let l = self.0.slice_after().len();
        if l == 0 {
            (0, Some(0))
        } else {
            (1, Some(l))
        }
    }
}

/**
A left-to-right iterator over code points and cursor positions.

The `(char, StrCursor)` pairs emitted are equivalent to calling `StrCursor::next_cp` on the current position.
*/
pub struct IterCpAfterCursor<'a>(
    /// The current cursor position.
    pub StrCursor<'a>,
);

impl<'a> Iterator for IterCpAfterCursor<'a> {
    type Item = (char, StrCursor<'a>);

    fn next(&mut self) -> Option<Self::Item> {
        match self.0.next_cp() {
            Some((cp, cur)) => {
                self.0 = cur;
                Some((cp, cur))
            },
            None => None,
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let l = self.0.slice_after().len();
        if l == 0 {
            (0, Some(0))
        } else {
            (1, Some(l))
        }
    }
}
