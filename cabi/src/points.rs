// Memento - A Whisper implementation in Rust
//
// Copyright 2017-2018 TSH Labs
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms

use std::mem;
use std::ptr;
use std::ffi::CStr;
use std::os::raw::c_char;
use chrono::{TimeZone, Utc};
use memento::{FetchRequest, MementoFileReader};
use memento::errors::ErrorKind;
use memento::types::Point;
use common::MementoErrorCode;

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
                // Convert back into a Rust type to free the memory
                Vec::from_raw_parts(self.points as *mut MementoPoint, self.size, self.size);
            }
        }
    }
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
/// The result must be freed by calling `memento_points_free` for both
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
    Box::into_raw(Box::new(_memento_points_fetch(
        path,
        from,
        Some(until),
        None,
    )))
}

/// Fetch points contained in a Whisper database file between the
/// given start and end times (unix timestamps in seconds) using the
/// given `now` time to determine if the request can be satisfied.
///
/// The returned pointer will never be null. Callers must check the
/// return value with the `memento_result_is_error` function before
/// trying to use the array of points associated with it. If the response
/// was successful, `points` will be a pointer to the start of an array
/// of points and `size` will be the length of the array. If the response
/// was unsucessful, `points` will be null and `error` will contain an
/// error code indicating what went wrong.
///
/// The result must be freed by calling `memento_points_free` for both
/// successful responses and error responses.
///
/// This method will panic if the given path pointer is null.
#[no_mangle]
pub extern "C" fn memento_points_fetch_full(
    path: *const c_char,
    from: i64,
    until: i64,
    now: i64,
) -> *mut MementoPointsResult {
    assert!(
        !path.is_null(),
        "memento_points_fetch_full: unexpected null pointer"
    );
    Box::into_raw(Box::new(_memento_points_fetch(
        path,
        from,
        Some(until),
        Some(now),
    )))
}

fn _memento_points_fetch(
    path: *const c_char,
    from: i64,
    until: Option<i64>,
    now: Option<i64>,
) -> MementoPointsResult {
    let c_str = unsafe { CStr::from_ptr(path) };
    let wsp = match c_str.to_str() {
        Ok(v) => v,
        Err(_) => return MementoPointsResult::from_error_code(MementoErrorCode::InvalidString),
    };

    let reader = MementoFileReader::default();
    let until_ts = until.map(|v| Utc.timestamp(v, 0)).unwrap_or_else(Utc::now);
    let now_ts = now.map(|v| Utc.timestamp(v, 0)).unwrap_or_else(Utc::now);
    let request = FetchRequest::new(Utc.timestamp(from, 0), until_ts, now_ts);

    match reader.read(wsp, &request) {
        Ok(response) => {
            let points: Vec<Point> = response.into();
            MementoPointsResult::from_points(
                points.into_iter().map(|p| MementoPoint::from(p)).collect(),
            )
        },
        Err(err) => MementoPointsResult::from_error_kind(err.kind()),
    }
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

/// Free memory used by this result and potentially any points associated
/// with it. This method will panic if the given result pointer is null.
#[no_mangle]
pub extern "C" fn memento_points_free(res: *mut MementoPointsResult) {
    assert!(
        !res.is_null(),
        "memento_points_free: unexpected null pointer"
    );
    // Turn our pointer to a result object back into a Boxed type so it can be dropped.
    unsafe { Box::from_raw(res) };
}
