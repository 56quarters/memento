extern crate chrono;
extern crate memento;

use chrono::{TimeZone, Utc};
use memento::types::AggregationType;
use memento::{FetchRequest, MementoFileReader};

#[test]
fn test_memento_file_reader_read_header() {
    let reader = MementoFileReader::new();
    let header = reader.read_header("tests/upper_01.wsp").unwrap();

    assert_eq!(AggregationType::Max, header.metadata().aggregation());
    assert_eq!(5, header.metadata().archive_count());
}

#[test]
fn test_memento_file_reader_read_database() {
    let reader = MementoFileReader::new();
    let database = reader.read_database("tests/upper_01.wsp").unwrap();
    let header = database.header();

    assert_eq!(AggregationType::Max, header.metadata().aggregation());
    assert_eq!(5, database.data().archives().len());
}

#[test]
fn test_memento_file_reader_read() {
    let from = Utc.timestamp(1502089980, 0);
    let until = Utc.timestamp(1502259660, 0);
    let now = Utc.timestamp(1502864800, 0);
    let request = FetchRequest::new(from, until, now);

    let reader = MementoFileReader::new();
    let response = reader.read("tests/upper_01.wsp", &request).unwrap();
    let info = response.archive();
    let points = response.points();

    assert_eq!(300, info.seconds_per_point());
    assert_eq!(566, points.len());
}
