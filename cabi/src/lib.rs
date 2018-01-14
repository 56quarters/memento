// Memento - A Whisper implementation in Rust
//
// Copyright 2017-2018 TSH Labs
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! C compatible API for Memento

extern crate chrono;
extern crate memento;

use std::mem;
use std::ptr;
use std::ffi::CStr;
use std::os::raw::c_char;
use chrono::{TimeZone, Utc};
use memento::{FetchRequest, MementoFileReader};
use memento::errors::ErrorKind;
use memento::types::Point;

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MementoErrorCode {
    NoError = 0,
    InvalidString = 101,
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
#[derive(Debug, Clone, Default, PartialEq)]
pub struct MementoPoint {
    pub value: f64,
    pub timestamp: u32,
}

impl From<Point> for MementoPoint {
    fn from(p: Point) -> Self {
        MementoPoint {
            value: p.value(),
            timestamp: p.timestamp(),
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, PartialEq)]
pub struct MementoResult {
    pub points: *mut MementoPoint,
    pub size: usize,
    pub error: MementoErrorCode,
}

impl MementoResult {
    pub fn from_points(mut res: Vec<MementoPoint>) -> MementoResult {
        res.shrink_to_fit();
        let out = MementoResult {
            error: MementoErrorCode::NoError,
            points: (&mut res).as_mut_ptr(),
            size: res.len(),
        };
        mem::forget(res);
        out
    }

    pub fn from_error_code(err: MementoErrorCode) -> MementoResult {
        MementoResult {
            error: err,
            points: ptr::null_mut(),
            size: 0,
        }
    }

    pub fn from_error_kind(err: ErrorKind) -> MementoResult {
        MementoResult {
            error: MementoErrorCode::from(err),
            points: ptr::null_mut(),
            size: 0,
        }
    }

    pub fn is_error(&self) -> bool {
        self.error.is_error()
    }
}

impl Drop for MementoResult {
    fn drop(&mut self) {
        if !self.points.is_null() {
            unsafe {
                // If our results are non-null they were created by Rust code from a valid
                // vector of results, it's safe to recreate the vector here to ensure the
                // memory is reclaimed.
                Vec::from_raw_parts(self.points as *mut MementoPoint, self.size, self.size);
            }
            self.error = MementoErrorCode::NoError;
            self.points = ptr::null_mut();
            self.size = 0;
        }
    }
}

impl Default for MementoResult {
    fn default() -> Self {
        MementoResult {
            error: MementoErrorCode::NoError,
            points: ptr::null_mut(),
            size: 0,
        }
    }
}

///
///
///
#[no_mangle]
pub extern "C" fn memento_result_fetch(path: *const c_char, from: i64, until: i64) -> *mut MementoResult {
    assert!(!path.is_null(), "memento_result_fetch: unexpected null pointer");
    Box::into_raw(Box::new(_memento_result_fetch(path, from, until)))
}

fn _memento_result_fetch(path: *const c_char, from: i64, until: i64) -> MementoResult {
    let c_str = unsafe { CStr::from_ptr(path) };
    let wsp = match c_str.to_str() {
        Ok(v) => v,
        Err(_) => return MementoResult::from_error_code(MementoErrorCode::InvalidString),
    };

    let reader = MementoFileReader::default();
    let request = FetchRequest::default()
        .with_from(Utc.timestamp(from, 0))
        .with_until(Utc.timestamp(until, 0));

    match reader.read(wsp, &request) {
        Ok(points) => {
            MementoResult::from_points(points.into_iter().map(|p| MementoPoint::from(p)).collect())
        }
        Err(err) => MementoResult::from_error_kind(err.kind()),
    }
}

///
///
///
#[no_mangle]
pub extern "C" fn memento_result_free(res: *mut MementoResult) {
    assert!(!res.is_null(), "memento_result_free: unexpected null pointer");
    // Turn our pointer to a result object back into a Boxed type so it can be dropped.
    // The destructor for our result object will in turn, convert the c-style array
    // (pointer + length) back into a Rust vector so that it can be properly dropped as
    // well.
    unsafe { Box::from_raw(res) };
}

///
///
///
#[no_mangle]
pub extern "C" fn memento_result_is_error(res: *const MementoResult) -> bool {
    assert!(!res.is_null(), "memento_result_is_error: unexpected null pointer");
    unsafe { (*res).is_error() }
}
