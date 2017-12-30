#![feature(test)]
extern crate memento_core;
extern crate test;

use memento_core::encoder::{memento_encode_database, memento_encode_header};
use memento_core::parser::{memento_parse_database, memento_parse_header};

#[bench]
fn bench_memento_parse_header(b: &mut test::Bencher) {
    let bytes = include_bytes!("../../tests/mean_01.wsp");
    b.iter(|| {
        memento_parse_header(bytes).unwrap();
    });
}

#[bench]
fn bench_memento_parse_database(b: &mut test::Bencher) {
    let bytes = include_bytes!("../../tests/mean_01.wsp");
    b.iter(|| {
        memento_parse_database(bytes).unwrap();
    });
}

#[bench]
fn bench_memento_encode_header(b: &mut test::Bencher) {
    let bytes = &include_bytes!("../../tests/mean_01.wsp")[0..76];
    let header = memento_parse_header(bytes).unwrap().1;
    let mut buf = Vec::with_capacity(bytes.len());

    b.iter(|| {
        memento_encode_header(&mut buf, &header).unwrap();
        buf.clear();
    });
}

#[bench]
fn bench_memento_encode_database(b: &mut test::Bencher) {
    let bytes = &include_bytes!("../../tests/mean_01.wsp")[..];
    let file = memento_parse_database(bytes).unwrap().1;
    let mut buf = Vec::with_capacity(bytes.len());

    b.iter(|| {
        memento_encode_database(&mut buf, &file).unwrap();
        buf.clear();
    });
}
