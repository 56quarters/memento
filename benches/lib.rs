#![feature(test)]
extern crate test;
extern crate whisper;

use whisper::parser::{whisper_parse_file, whisper_parse_header};


#[bench]
fn bench_whisper_parse_header(b: &mut test::Bencher) {
    let bytes = include_bytes!("../tests/mean_01.wsp");
    b.iter(|| {
        whisper_parse_header(bytes);
    });
}


#[bench]
fn bench_whisper_parse_file(b: &mut test::Bencher) {
    let bytes = include_bytes!("../tests/mean_01.wsp");
    b.iter(|| {
        whisper_parse_file(bytes);
    });
}
