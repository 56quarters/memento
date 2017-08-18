#![feature(test)]
extern crate test;
extern crate whisper;

use whisper::io::{whisper_write_file, whisper_write_header};
use whisper::parser::{whisper_parse_file, whisper_parse_header};


#[bench]
fn bench_whisper_parse_header(b: &mut test::Bencher) {
    let bytes = include_bytes!("../tests/mean_01.wsp");
    b.iter(|| { whisper_parse_header(bytes).unwrap(); });
}


#[bench]
fn bench_whisper_parse_file(b: &mut test::Bencher) {
    let bytes = include_bytes!("../tests/mean_01.wsp");
    b.iter(|| { whisper_parse_file(bytes).unwrap(); });
}


#[bench]
fn bench_whisper_write_header(b: &mut test::Bencher) {
    let bytes = &include_bytes!("../tests/mean_01.wsp")[0..76];
    let header = whisper_parse_header(bytes).unwrap().1;
    let mut buf = Vec::with_capacity(bytes.len());

    b.iter(|| { whisper_write_header(&mut buf, &header).unwrap(); });
}


#[bench]
fn bench_whisper_write_file(b: &mut test::Bencher) {
    let bytes = &include_bytes!("../tests/mean_01.wsp")[..];
    let file = whisper_parse_file(bytes).unwrap().1;
    let mut buf = Vec::with_capacity(bytes.len());

    b.iter(|| { whisper_write_file(&mut buf, &file).unwrap(); });
}
