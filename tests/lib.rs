extern crate whisper;

use whisper::file::{AggregationType, Archive, ArchiveInfo, WhisperFile,
                    Header, Metadata, Data};
use whisper::parse::{whisper_parse_file, whisper_parse_header};


#[test]
fn test_parse_header_mean() {
    let bytes = include_bytes!("mean_01.wsp");
    let header = whisper_parse_header(bytes);
    println!("mean header: {:?}", header.to_result().unwrap());
}

#[test]
fn test_parse_header_count() {
    let bytes = include_bytes!("count_01.wsp");
    let header = whisper_parse_header(bytes);
    println!("count header: {:?}", header.to_result().unwrap());
}

#[test]
fn test_parse_header_upper() {
    let bytes = include_bytes!("upper_01.wsp");
    let header = whisper_parse_header(bytes);
    println!("upper header: {:?}", header.to_result().unwrap());
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
