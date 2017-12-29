extern crate memento;

use memento::errors::{ErrorKind};
use memento::types::{Header, Point, MementoDatabase};


#[repr(u32)]
pub enum MementoErrorCode {
    NoError = 0,
    IoError = 1,
    ParseEerror = 3,
    InvalidTimeRange = 4,
    InvalidTimeStart = 5,
    InvalidTimeEnd = 6,
    NoArchiveAvailable = 7,
    CorruptDatabase = 8,
}


impl From<ErrorKind> for MementoErrorCode {
    fn from(kind: ErrorKind) -> MementoErrorCode {
        match kind {
            ErrorKind::IoError => MementoErrorCode::IoError,
            ErrorKind::ParseError => MementoErrorCode::ParseEerror,
            ErrorKind::InvalidTimeRange => MementoErrorCode::InvalidTimeRange,
            ErrorKind::InvalidTimeStart => MementoErrorCode::InvalidTimeStart,
            ErrorKind::InvalidTimeEnd => MementoErrorCode::InvalidTimeEnd,
            ErrorKind::NoArchiveAvailable => MementoErrorCode::NoArchiveAvailable,
            ErrorKind::CorruptDatabase => MementoErrorCode::CorruptDatabase,
        }
    }
}


#[repr(C)]
pub struct MementoResult {
    pub error: MementoErrorCode,
}


#[no_mangle]
pub extern "C" fn whisper_fetch_path(from: u64, until: u64) -> MementoResult {
    MementoResult { error: MementoErrorCode::NoError }
}
