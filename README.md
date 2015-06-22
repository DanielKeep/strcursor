# `strcursor`

**Note**: This is something of a work-in-progress.  It has tests, but hasn't been exhaustively vetted.

This provides a `StrCursor` type that allows you to seek back and forth through a `&str`.  Importantly, it respects both codepoint and *grapheme cluster* boundaries.

If you're not sure what those words mean: don't use methods with `cp` in the name; they're probably not what you want.
