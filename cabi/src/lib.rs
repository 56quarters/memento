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
use memento::types::{Metadata, Point};

// Just reuse our existing aggreation enum
pub use memento::types::AggregationType;

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
pub struct MementoPointsResult {
    pub points: *mut MementoPoint,
    pub size: usize,
    pub error: MementoErrorCode,
}

impl MementoPointsResult {
    pub fn from_points(mut res: Vec<MementoPoint>) -> Self {
        res.shrink_to_fit();
        let out = MementoPointsResult {
            error: MementoErrorCode::NoError,
            points: (&mut res).as_mut_ptr(),
            size: res.len(),
        };
        mem::forget(res);
        out
    }

    pub fn from_error_code(err: MementoErrorCode) -> Self {
        MementoPointsResult {
            error: err,
            points: ptr::null_mut(),
            size: 0,
        }
    }

    pub fn from_error_kind(err: ErrorKind) -> Self {
        MementoPointsResult {
            error: MementoErrorCode::from(err),
            points: ptr::null_mut(),
            size: 0,
        }
    }

    pub fn is_error(&self) -> bool {
        self.error.is_error()
    }
}

impl Drop for MementoPointsResult {
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

#[repr(C)]
#[derive(Debug, Clone, PartialEq)]
pub struct MementoMetadata {
    pub aggregation: AggregationType,
    pub max_retention: u32,
    pub x_files_factor: f32,
    pub archive_count: u32,
}

impl From<Metadata> for MementoMetadata {
    fn from(meta: Metadata) -> Self {
        MementoMetadata {
            aggregation: meta.aggregation(),
            max_retention: meta.max_retention(),
            x_files_factor: meta.x_files_factor(),
            archive_count: meta.archive_count(),
        }
    }
}

///
///
///
#[no_mangle]
pub extern "C" fn memento_header_fetch(path: *const c_char) {
    unimplemented!();
}

///
///
///
#[no_mangle]
pub extern "C" fn memento_header_is_error(path: *const c_char) {
    unimplemented!();
}

///
///
///
#[no_mangle]
pub extern "C" fn memento_header_free(path: *const c_char) {
    unimplemented!();
}

/// Fetch points contained in a Whisper database file between the
/// given start and end times (unix timestamps in seconds).
///
/// The returned pointer will never be null. Callers must check the
/// return value with the `memento_result_is_error` function before
/// trying to use the array of points associated with it. If the response
/// was successful, `points` will be a pointer to the start of an array
/// of points and `size` will be the length of the array. If the response
/// was unsucessful, `points` will be null and `error` will contain an
/// error code indicating what went wrong.
///
/// Results must be freed via calling `memento_result_free` for both
/// successful responses and error responses.
///
/// This method will panic if the given path pointer is null.
#[no_mangle]
pub extern "C" fn memento_points_fetch(
    path: *const c_char,
    from: i64,
    until: i64,
) -> *mut MementoPointsResult {
    assert!(
        !path.is_null(),
        "memento_points_fetch: unexpected null pointer"
    );
    Box::into_raw(Box::new(_memento_points_fetch(path, from, until)))
}

fn _memento_points_fetch(path: *const c_char, from: i64, until: i64) -> MementoPointsResult {
    let c_str = unsafe { CStr::from_ptr(path) };
    let wsp = match c_str.to_str() {
        Ok(v) => v,
        Err(_) => return MementoPointsResult::from_error_code(MementoErrorCode::InvalidString),
    };

    let reader = MementoFileReader::default();
    let request = FetchRequest::default()
        .with_from(Utc.timestamp(from, 0))
        .with_until(Utc.timestamp(until, 0));

    match reader.read(wsp, &request) {
        Ok(points) => MementoPointsResult::from_points(
            points.into_iter().map(|p| MementoPoint::from(p)).collect(),
        ),
        Err(err) => MementoPointsResult::from_error_kind(err.kind()),
    }
}

/// Free memory used by this result and potentially any points associated
/// with it. This method will panic if the given result pointer is null.
#[no_mangle]
pub extern "C" fn memento_points_free(res: *mut MementoPointsResult) {
    assert!(
        !res.is_null(),
        "memento_points_free: unexpected null pointer"
    );
    // Turn our pointer to a result object back into a Boxed type so it can be dropped.
    // The destructor for our result object will in turn, convert the c-style array
    // (pointer + length) back into a Rust vector so that it can be properly dropped as
    // well.
    unsafe { Box::from_raw(res) };
}

/// Return true if this result is an error, false otherwise. This
/// method will panic if the given result pointer is null.
#[no_mangle]
pub extern "C" fn memento_points_is_error(res: *const MementoPointsResult) -> bool {
    assert!(
        !res.is_null(),
        "memento_points_is_error: unexpected null pointer"
    );
    unsafe { (*res).is_error() }
}
