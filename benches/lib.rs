#![feature(test)]
extern crate chrono;
extern crate memento;
extern crate test;

use chrono::{TimeZone, Utc};
use test::Bencher;
use memento::{MappedFileStream, MementoFileReader, FetchRequest};

#[bench]
fn benchmark_mapped_file_stream_with_locking(b: &mut Bencher) {
    let stream = MappedFileStream::new(true);
    b.iter(|| stream.run_immutable("tests/upper_01.wsp", |_: &[u8]| Ok(())));
}

#[bench]
fn benchmark_mapped_file_stream_no_locking(b: &mut Bencher) {
    let stream = MappedFileStream::new(false);
    b.iter(|| stream.run_immutable("tests/upper_01.wsp", |_: &[u8]| Ok(())));
}

#[bench]
fn benchmark_memento_file_reader_with_lock_read_header(b: &mut Bencher) {
    let stream = MappedFileStream::new(true);
    let reader = MementoFileReader::new(stream);
    b.iter(|| reader.read_header("tests/upper_01.wsp"));
}

#[bench]
fn benchmark_memento_file_reader_with_lock_read_database(b: &mut Bencher) {
    let stream = MappedFileStream::new(true);
    let reader = MementoFileReader::new(stream);
    b.iter(|| reader.read_database("tests/upper_01.wsp"));
}

#[bench]
fn benchmark_memento_file_reader_with_lock_read_range(b: &mut Bencher) {
    let from = Utc.timestamp(1502089980, 0);
    let until = Utc.timestamp(1502259660, 0);
    let now = Utc.timestamp(1502864800, 0);
    let request = FetchRequest::new(from, until, now);

    let stream = MappedFileStream::new(true);
    let reader = MementoFileReader::new(stream);

    b.iter(|| reader.read("tests/upper_01.wsp", &request));
}

#[bench]
fn benchmark_memento_file_reader_no_lock_read_header(b: &mut Bencher) {
    let stream = MappedFileStream::new(false);
    let reader = MementoFileReader::new(stream);
    b.iter(|| reader.read_header("tests/upper_01.wsp"));
}

#[bench]
fn benchmark_memento_file_reader_no_lock_read_database(b: &mut Bencher) {
    let stream = MappedFileStream::new(false);
    let reader = MementoFileReader::new(stream);
    b.iter(|| reader.read_database("tests/upper_01.wsp"));
}

#[bench]
fn benchmark_memento_file_reader_no_lock_read_range(b: &mut Bencher) {
    let from = Utc.timestamp(1502089980, 0);
    let until = Utc.timestamp(1502259660, 0);
    let now = Utc.timestamp(1502864800, 0);
    let request = FetchRequest::new(from, until, now);

    let stream = MappedFileStream::new(false);
    let reader = MementoFileReader::new(stream);

    b.iter(|| reader.read("tests/upper_01.wsp", &request));
}
