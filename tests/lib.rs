extern crate whisper;

use whisper::types::AggregationType;
use whisper::encoder::{whisper_encode_header, whisper_encode_file};
use whisper::io::MappedFileStream;
use whisper::parser::{whisper_parse_file, whisper_parse_header};

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
    assert_eq!(10_080, info[1].num_points());
    assert_eq!(8640, info[2].num_points());
    assert_eq!(25_920, info[3].num_points());
    assert_eq!(8760, info[4].num_points());
}

#[test]
fn test_parse_file_mean() {
    let bytes = include_bytes!("mean_01.wsp");
    let res = whisper_parse_file(bytes);
    let file = res.unwrap().1;
    let header = file.header();
    let data = file.data();

    let meta = header.metadata();
    let info = header.archive_info();
    let archives = data.archives();

    assert_eq!(AggregationType::Average, meta.aggregation());
    assert_eq!(SECONDS_PER_YEAR, meta.max_retention());
    assert_eq!(5, meta.archive_count());

    assert_eq!(8640, info[0].num_points());
    assert_eq!(8640, archives[0].points().len());

    assert_eq!(10_080, info[1].num_points());
    assert_eq!(10_080, archives[1].points().len());

    assert_eq!(8640, info[2].num_points());
    assert_eq!(8640, archives[2].points().len());

    assert_eq!(25_920, info[3].num_points());
    assert_eq!(25_920, archives[3].points().len());

    assert_eq!(8760, info[4].num_points());
    assert_eq!(8760, archives[4].points().len());
}

#[test]
fn test_read_parse_encode_header() {
    // The header of this whisper file is only the first 76 bytes
    let header_bytes = &include_bytes!("mean_01.wsp")[0..76];
    let header = whisper_parse_header(header_bytes).unwrap().1;

    let mut buf = vec![];
    whisper_encode_header(&mut buf, &header).unwrap();

    assert_eq!(header_bytes, &buf as &[u8]);
}

#[test]
fn test_read_parse_encode_file() {
    let file_bytes = &include_bytes!("mean_01.wsp")[..];
    let file = whisper_parse_file(file_bytes).unwrap().1;

    let mut buf = vec![];
    whisper_encode_file(&mut buf, &file).unwrap();

    assert_eq!(file_bytes, &buf as &[u8]);
}

#[test]
fn test_mapped_file_stream_mutable() {
    let mut expected: [u8; 1024] = [0; 1024];
    let expected_ref = &mut expected as &mut [u8];

    let mut mapper = MappedFileStream::new();
    let _ = mapper.run_mutable("tests/zero_file.bin", |bytes| {
        assert_eq!(expected_ref, bytes);
        Ok(0)
    }).unwrap();
}


#[test]
fn test_mapped_file_stream_immutable() {
    let expected: [u8; 1024] = [0; 1024];
    let expected_ref = &expected as &[u8];

    let mapper = MappedFileStream::new();
    let _ = mapper.run_immutable("tests/zero_file.bin", |bytes| {
        assert_eq!(expected_ref, bytes);
        Ok(0)
    }).unwrap();
}
