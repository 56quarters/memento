extern crate chrono;
extern crate memento;

use std::mem;
use std::ptr;
use std::slice;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use chrono::{TimeZone, Utc};
use memento::{FetchRequest, MementoFileReader, MappedFileStream};
use memento::errors::ErrorKind;
use memento::types::Point;

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MementoErrorCode {
    NoError = 0,

    InvalidEncoding = 101,

    IoError = 1001,
    ParseEerror = 1002,
    InvalidTimeRange = 1003,
    InvalidTimeStart = 1004,
    InvalidTimeEnd = 1005,
    NoArchiveAvailable = 1006,
    CorruptDatabase = 1007,
}

impl MementoErrorCode {
    pub fn is_error(&self) -> bool {
        *self != MementoErrorCode::NoError
    }
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

impl Default for MementoErrorCode {
    fn default() -> Self {
        MementoErrorCode::NoError
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct MementoResult {
    pub error: MementoErrorCode,
    pub results: *mut Point,
    pub size: usize,
}

impl MementoResult {
    pub fn from_results(mut res: Vec<Point>) -> MementoResult {
        res.shrink_to_fit();
        let out = MementoResult {
            error: MementoErrorCode::NoError,
            results: (&mut res).as_mut_ptr(),
            size: res.len(),
        };
        mem::forget(res);
        out
    }

    pub fn from_error_code(err: MementoErrorCode) -> MementoResult {
        MementoResult {
            error: err,
            results: ptr::null_mut(),
            size: 0,
        }
    }

    pub fn from_error_kind(err: ErrorKind) -> MementoResult {
        MementoResult {
            error: MementoErrorCode::from(err),
            results: ptr::null_mut(),
            size: 0,
        }
    }

    pub fn free(&mut self) {
        if !self.is_null() {
            unsafe {
                // If this is non-null it was created by Rust code from a valid vector
                // of results, it's safe to recreate the vector here.
                Vec::from_raw_parts(self.results as *mut Point, self.size, self.size);
            }
            self.error = MementoErrorCode::NoError;
            self.results = ptr::null_mut();
            self.size = 0;
        }
    }

    pub fn is_null(&self) -> bool {
        self.results == ptr::null_mut()
    }

    pub fn is_error(&self) -> bool {
        self.error.is_error()
    }
}

impl Default for MementoResult {
    fn default() -> Self {
        MementoResult {
            error: MementoErrorCode::NoError,
            results: ptr::null_mut(),
            size: 0
        }
    }
}

#[allow(unused_variables)]
#[no_mangle]
pub extern "C" fn memento_fetch_path(path: *const c_char, from: u64, until: u64) -> MementoResult {
    assert!(path != ptr::null(), "Unexpected null path string");

    //CStr::from_bytes_with_nul(path as &[u8]);

    let wsp = unsafe { CStr::from_ptr(path).to_owned().to_str().unwrap().to_owned() };

    let stream = MappedFileStream::new();
    let reader = MementoFileReader::new(stream);
    let request = FetchRequest::default()
        .with_from(Utc.timestamp(from as i64, 0))
        .with_until(Utc.timestamp(until as i64, 0));

    match reader.read(wsp, &request) {
        Ok(points) => MementoResult::from_results(points),
        Err(err) => MementoResult::from_error_kind(err.kind()),
    }
}

#[no_mangle]
pub unsafe extern "C" fn mement_result_free(res: *mut MementoResult) {
    assert!(res != ptr::null_mut(), "Unexpected null result pointer");
    (*res).free();
}

#[no_mangle]
pub unsafe extern "C" fn memento_result_is_error(res: *const MementoResult) -> bool {
    assert!(res != ptr::null(), "Unexpected null result pointer");
    (*res).is_error()
}
