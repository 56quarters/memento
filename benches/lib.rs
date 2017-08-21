#![feature(test)]
extern crate test;
extern crate memmap;
extern crate whisper;

use std::io::{Seek, SeekFrom, BufReader};
use std::fs::File;

use whisper::io::{whisper_write_file, whisper_write_header};
use whisper::parser::{whisper_parse_file, whisper_parse_header};
use whisper::io::{
    whisper_read_header,
    whisper_read_file_big_buf,
    whisper_read_file_small_buf,
    whisper_read_file_mmap,
    whisper_read_file_mmap2,
};


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


#[bench]
fn bench_whisper_read_header(b: &mut test::Bencher) {
    let path = "tests/mean_01.wsp";
    let mut reader = BufReader::new(File::open(path).unwrap());
    b.iter(|| {
        reader.seek(SeekFrom::Start(0)).unwrap();
        whisper_read_header(&mut reader).unwrap();
    });
}


#[bench]
fn bench_whisper_read_file_big_buff(b: &mut test::Bencher) {
    let path = "tests/mean_01.wsp";
    let mut reader = BufReader::new(File::open(path).unwrap());
    b.iter(|| {
        reader.seek(SeekFrom::Start(0)).unwrap();
        whisper_read_file_big_buf(&mut reader).unwrap();
    });
}


#[bench]
fn bench_whisper_read_file_small_buf(b: &mut test::Bencher) {
    let path = "tests/mean_01.wsp";
    let mut reader = BufReader::new(File::open(path).unwrap());
    b.iter(|| {
        reader.seek(SeekFrom::Start(0)).unwrap();
        whisper_read_file_small_buf(&mut reader).unwrap();
    });
}


use memmap::{Mmap, Protection};

#[bench]
fn bench_whisper_read_file_mmap(b: &mut test::Bencher) {
    let path = "tests/mean_01.wsp";
    let map = Mmap::open_path(path, Protection::Read).unwrap();
    b.iter(|| {
        whisper_read_file_mmap(&map).unwrap();
    });
}


#[bench]
fn bench_whisper_read_file_mmap2(b: &mut test::Bencher) {
    let path = "tests/mean_01.wsp";
    b.iter(|| {
        whisper_read_file_mmap2(path).unwrap();
    });
}
