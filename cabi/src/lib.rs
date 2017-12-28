extern crate memento;

/*
use memento::{FetchRequest, MappedFileStream, WhisperFileReader};
use memento::errors::{ErrorKind, WhisperError, WhisperResult};
use memento::types::{Header, Point, WhisperFile};
*/

#[no_mangle]
pub extern "C" fn whisper_fetch_path(path: &str, from: u64, until: u64) -> u64 {
    0
}
