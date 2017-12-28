extern crate cbindgen;

use std::env;

fn main() {
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    cbindgen::Builder::new()
        .with_header("/* begin generated memento header (see cabi/build.rs) */")
        .with_trailer("/* end generated memento header (see cabi/build.rs) */")
        .with_include_guard("MEMENTO_H_INCLUDED")
        .with_language(cbindgen::Language::C)
        .with_crate_and_name(crate_dir, "memento-cabi")
        .with_parse_deps(true)
        .generate()
        .expect("Failed to generate bindings")
        .write_to_file("include/memento.h");
}
