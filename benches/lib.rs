#![feature(test)]
extern crate chrono;
extern crate memento;
extern crate test;

use chrono::{TimeZone, Utc};
use test::Bencher;
use memento::{FetchRequest, MementoFileReader};

#[bench]
fn benchmark_memento_file_reader_read_header(b: &mut Bencher) {
    let reader = MementoFileReader::new();
    b.iter(|| reader.read_header("tests/upper_01.wsp"));
}

#[bench]
fn benchmark_memento_file_reader_read_database(b: &mut Bencher) {
    let reader = MementoFileReader::new();
    b.iter(|| reader.read_database("tests/upper_01.wsp"));
}

#[bench]
fn benchmark_memento_file_reader_read_range(b: &mut Bencher) {
    let from = Utc.timestamp(1502089980, 0);
    let until = Utc.timestamp(1502259660, 0);
    let now = Utc.timestamp(1502864800, 0);
    let request = FetchRequest::new(from, until, now);

    let reader = MementoFileReader::new();
    b.iter(|| reader.read("tests/upper_01.wsp", &request));
}
