// Memento - A Whisper implementation in Rust
//
// Copyright 2017-2018 TSH Labs
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::mem;
use std::ptr;
use std::ffi::CStr;
use std::os::raw::c_char;
use memento::MementoFileReader;
use memento::errors::ErrorKind;
use memento::types::{AggregationType, ArchiveInfo, Header, Metadata};
use common::MementoErrorCode;

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

#[repr(C)]
pub struct MementoArchiveInfo {
    pub offset: u32,
    pub seconds_per_point: u32,
    pub num_points: u32,
}

impl From<ArchiveInfo> for MementoArchiveInfo {
    fn from(info: ArchiveInfo) -> Self {
        MementoArchiveInfo {
            offset: info.offset(),
            seconds_per_point: info.seconds_per_point(),
            num_points: info.num_points(),
        }
    }
}

#[repr(C)]
pub struct MementoHeader {
    pub metadata: MementoMetadata,
    pub archives: *mut MementoArchiveInfo,
    pub size: usize,
}

impl From<Header> for MementoHeader {
    fn from(header: Header) -> Self {
        let meta = MementoMetadata::from(header.metadata().clone());
        let mut archives: Vec<MementoArchiveInfo> = header.archive_info().iter()
            .map(|i| MementoArchiveInfo::from(i.clone()))
            .collect();

        archives.shrink_to_fit();
        let out = MementoHeader {
            metadata: meta,
            archives: (&mut archives).as_mut_ptr(),
            size: archives.len(),
        };
        mem::forget(archives);
        out
    }
}

impl Drop for MementoHeader {
    fn drop(&mut self) {
        if !self.archives.is_null() {
            unsafe {
                // Convert back into a Rust type to free the memory
                Vec::from_raw_parts(self.archives as *mut MementoArchiveInfo, self.size, self.size);
            }
        }
    }
}

#[repr(C)]
pub struct MementoHeaderResult {
    pub header: *mut MementoHeader,
    pub error: MementoErrorCode,
}

impl MementoHeaderResult {
    pub fn from_header(header: MementoHeader) -> Self {
        MementoHeaderResult {
            header: Box::into_raw(Box::new(header)),
            error: MementoErrorCode::NoError,
        }
    }

    pub fn from_error_code(err: MementoErrorCode) -> Self {
        MementoHeaderResult {
            header: ptr::null_mut(),
            error: err,
        }
    }

    pub fn from_error_kind(err: ErrorKind) -> Self {
        MementoHeaderResult {
            header: ptr::null_mut(),
            error: MementoErrorCode::from(err),
        }
    }

    pub fn is_error(&self) -> bool {
        self.error.is_error()
    }
}

impl Drop for MementoHeaderResult {
    fn drop(&mut self) {
        if !self.header.is_null() {
            unsafe {
                // Convert back into a Rust type to free the memory
                Box::from_raw(self.header);
            }
        }
    }
}

///
///
///
#[no_mangle]
pub extern "C" fn memento_header_fetch(path: *const c_char) -> *mut MementoHeaderResult {
    assert!(
        !path.is_null(),
        "memento_header_fetch: unexpected null pointer"
    );

    Box::into_raw(Box::new(_memento_header_fetch(path)))
}

fn _memento_header_fetch(path: *const c_char) -> MementoHeaderResult {
    let c_str = unsafe { CStr::from_ptr(path) };
    let wsp = match c_str.to_str() {
        Ok(v) => v,
        Err(_) => return MementoHeaderResult::from_error_code(MementoErrorCode::InvalidString),
    };

    let reader = MementoFileReader::default();
    match reader.read_header(wsp) {
        Ok(header) => MementoHeaderResult::from_header(MementoHeader::from(header)),
        Err(err) => MementoHeaderResult::from_error_kind(err.kind()),
    }
}

///
///
///
#[no_mangle]
pub extern "C" fn memento_header_is_error(res: *const MementoHeaderResult) -> bool {
    assert!(
        !res.is_null(),
        "memento_header_is_error: unexpected null pointer"
    );

    unsafe { (*res).is_error() }
}

///
///
///
#[no_mangle]
pub extern "C" fn memento_header_free(res: *mut MementoHeaderResult) {
    assert!(
        !res.is_null(),
        "memento_header_free: unexpected null pointer"
    );

    // Turn our pointer to a result object back into a boxed type so it can be dropped.
    unsafe { Box::from_raw(res) };
}
