extern crate whisper;

use whisper::encoder::{whisper_encode_header, whisper_encode_file};
use whisper::io::MappedFileStream;
use whisper::parser::{whisper_parse_file, whisper_parse_header};

// Encode tests... need some work


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
    let _ = mapper
        .run_mutable("tests/zero_file.bin", |bytes| {
            assert_eq!(expected_ref, bytes);
            Ok(0)
        })
        .unwrap();
}


#[test]
fn test_mapped_file_stream_immutable() {
    let expected: [u8; 1024] = [0; 1024];
    let expected_ref = &expected as &[u8];

    let mapper = MappedFileStream::new();
    let _ = mapper
        .run_immutable("tests/zero_file.bin", |bytes| {
            assert_eq!(expected_ref, bytes);
            Ok(0)
        })
        .unwrap();
}


#[test]
fn test_whisper_reader_read_with_high_precison_archive() {}


#[test]
fn test_whisper_reader_read_with_lower_precision_archive() {}


#[test]
fn test_whisper_reader_read_with_mixed_timestamp_data_in_archive() {}


#[test]
fn test_whisper_reader_read_valid_header_missing_data() {}
