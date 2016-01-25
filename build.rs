extern crate rustc_version;
use rustc_version::{version_matches};

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    if version_matches("1.4.0") {
        println!("cargo:rustc-cfg=has_string_into_boxed_string");
    }
}
