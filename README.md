# `strcursor`

**Note**: This is something of a work-in-progress.  It has tests, but hasn't been exhaustively vetted.

This provides a `StrCursor` type that allows you to seek back and forth through a `&str`.  Importantly, it respects both codepoint and *grapheme cluster* boundaries.

If you're not sure what those words mean: don't use methods with `cp` in the name; they're probably not what you want.

## License

Licensed under either of

* MIT license (see [LICENSE](LICENSE) or <http://opensource.org/licenses/MIT>)
* Apache License, Version 2.0 (see [LICENSE](LICENSE) or <http://www.apache.org/licenses/LICENSE-2.0>)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you shall be dual licensed as above, without any additional terms or conditions.
