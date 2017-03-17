/*
Copyright ⓒ 2015-2017 Daniel Keep.

Licensed under the MIT license (see LICENSE or <http://opensource.org
/licenses/MIT>) or the Apache License, Version 2.0 (see LICENSE of
<http://www.apache.org/licenses/LICENSE-2.0>), at your option. All
files in the project carrying such notice may not be copied, modified,
or distributed except according to those terms.
*/
/*!
This crate provides a "cursor" type for string slices.  It provides the ability to safely seek back and forth through a string without worrying about producing invalid UTF-8 sequences, or splitting grapheme clusters.

In addition, it provides types to represent single grapheme clusters ([`Gc`](struct.Gc.html) and [`GcBuf`](struct.GcBuf.html)) as distinct from arbitrary string slices.

See the [`StrCursor`](struct.StrCursor.html) type for details.

<style type="text/css">
.link-block { font-family: "Fira Sans"; }
.link-block > p { display: inline-block; }
.link-block > p > strong { font-weight: 500; margin-right: 1em; }
.link-block > ul { display: inline-block; padding: 0; list-style: none; }
.link-block > ul > li {
  font-size: 0.8em;
  background-color: #eee;
  border: 1px solid #ccc;
  padding: 0.3em;
  display: inline-block;
}
</style>
<span></span><div class="link-block">

**Links**

* [Latest Release](https://crates.io/crates/strcursor/)
* [Latest Docs](https://docs.rs/strcursor/%2A/strcursor/index.html)
* [Repository](https://github.com/DanielKeep/strcursor)

<span></span></div>

## Compatibility

`strcursor` is currently supported on `rustc` version 1.7.0 and higher.

`rustc` version 1.1+ is supported by `strcursor` version `0.1.*`.

*/
#[macro_use] extern crate debug_unreachable;
extern crate unicode_segmentation as uniseg;

pub mod iter;

pub use cursor::StrCursor;
pub use grapheme::{Gc, GcBuf};

mod cursor;
mod grapheme;
mod util;
