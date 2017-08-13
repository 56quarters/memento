extern crate whisper;

use whisper::file::{AggregationType, Archive, ArchiveInfo, WhisperFile,
                    Header, Metadata, Data};
use whisper::parse::{whisper_parse_file, whisper_parse_header};

const SECONDS_PER_YEAR: u32 = 3600 * 24 * 365;

#[test]
fn test_parse_header_mean() {
    let bytes = include_bytes!("mean_01.wsp");
    let res = whisper_parse_header(bytes);
    let header = res.unwrap().1;
    let meta = header.metadata();
    let info = header.archive_info();

    assert_eq!(AggregationType::Average, meta.aggregation());
    assert_eq!(SECONDS_PER_YEAR, meta.max_retention());
    assert_eq!(5, meta.archive_count());

    assert_eq!(10, info[0].seconds_per_point());
    assert_eq!(60, info[1].seconds_per_point());
    assert_eq!(300, info[2].seconds_per_point());
    assert_eq!(600, info[3].seconds_per_point());
    assert_eq!(3600, info[4].seconds_per_point());

    assert_eq!(8640, info[0].num_points());
    assert_eq!(10080, info[1].num_points());
    assert_eq!(8640, info[2].num_points());
    assert_eq!(25920, info[3].num_points());
    assert_eq!(8760, info[4].num_points());
}

#[test]
fn test_parse_header_count() {
    let bytes = include_bytes!("count_01.wsp");
    let res = whisper_parse_header(bytes);
    let header = res.unwrap().1;
    let meta = header.metadata();
    let info = header.archive_info();

    assert_eq!(AggregationType::Sum, meta.aggregation());
    assert_eq!(SECONDS_PER_YEAR, meta.max_retention());
    assert_eq!(5, meta.archive_count());

    assert_eq!(10, info[0].seconds_per_point());
    assert_eq!(60, info[1].seconds_per_point());
    assert_eq!(300, info[2].seconds_per_point());
    assert_eq!(600, info[3].seconds_per_point());
    assert_eq!(3600, info[4].seconds_per_point());

    assert_eq!(8640, info[0].num_points());
    assert_eq!(10080, info[1].num_points());
    assert_eq!(8640, info[2].num_points());
    assert_eq!(25920, info[3].num_points());
    assert_eq!(8760, info[4].num_points());
}

#[test]
fn test_parse_header_upper() {
    let bytes = include_bytes!("upper_01.wsp");
    let res = whisper_parse_header(bytes);
    let header = res.unwrap().1;
    let meta = header.metadata();
    let info = header.archive_info();

    assert_eq!(AggregationType::Max, meta.aggregation());
    assert_eq!(SECONDS_PER_YEAR, meta.max_retention());
    assert_eq!(5, meta.archive_count());

    assert_eq!(10, info[0].seconds_per_point());
    assert_eq!(60, info[1].seconds_per_point());
    assert_eq!(300, info[2].seconds_per_point());
    assert_eq!(600, info[3].seconds_per_point());
    assert_eq!(3600, info[4].seconds_per_point());

    assert_eq!(8640, info[0].num_points());
    assert_eq!(10080, info[1].num_points());
    assert_eq!(8640, info[2].num_points());
    assert_eq!(25920, info[3].num_points());
    assert_eq!(8760, info[4].num_points());
}

#[test]
fn test_parse_file_mean() {
    let bytes = include_bytes!("mean_01.wsp");
}

#[test]
fn test_parse_file_count() {
    let bytes = include_bytes!("count_01.wsp");
}

#[test]
fn test_parse_file_upper() {
    let bytes = include_bytes!("upper_01.wsp");
}
