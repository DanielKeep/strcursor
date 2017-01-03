# `strcursor`

**Note**: This is something of a work-in-progress.  It has tests, but hasn't been exhaustively vetted.

This crate provides a "cursor" type for string slices.  It provides the ability to safely seek back and forth through a string without worrying about producing invalid UTF-8 sequences, or splitting grapheme clusters.

In addition, it provides types to represent single grapheme clusters (`Gc`) and `GcBuf`) as distinct from arbitrary string slices.

See the `StrCursor` type for details.

**Links**

* [Latest Release](https://crates.io/crates/strcursor/)
* [Latest Docs](https://danielkeep.github.io/strcursor/doc/strcursor/index.html)
* [Repository](https://github.com/DanielKeep/strcursor)

## Compatibility

`strcursor` is currently supported on `rustc` version 1.7.0 and higher.

## License

Licensed under either of

* MIT license (see [LICENSE](LICENSE) or <http://opensource.org/licenses/MIT>)
* Apache License, Version 2.0 (see [LICENSE](LICENSE) or <http://www.apache.org/licenses/LICENSE-2.0>)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you shall be dual licensed as above, without any additional terms or conditions.
