extern crate cbindgen;

use std::env;

fn main() {
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    cbindgen::Builder::new()
        .with_header("/* begin memento header */")
        .with_trailer("/* end memento header */")
        .with_include_guard("MEMENTO_H_INCLUDED")
        .with_language(cbindgen::Language::C)
        .with_crate(crate_dir)
        .generate()
        .expect("Failed to generate bindings")
        .write_to_file("include/memento.h");
}
