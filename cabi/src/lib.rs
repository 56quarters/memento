extern crate memento;

use std::mem;
use std::ptr;
use std::str;
use std::slice;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use memento::errors::ErrorKind;
//use memento::types::{Header, MementoDatabase, Point};

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
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct MementoStr {
    pub data: *const c_char,
    pub len: usize,
    pub owned: bool,
}

impl MementoStr {
    pub fn from_str(s: &str) -> MementoStr {
        MementoStr {
            data: s.as_ptr() as *const c_char,
            len: s.len(),
            owned: false,
        }
    }

    pub fn from_string(mut s: String) -> MementoStr {
        s.shrink_to_fit();
        let val = MementoStr {
            data: s.as_ptr() as *const c_char,
            len: s.len(),
            owned: true
        };
        mem::forget(s);
        val
    }

    pub unsafe fn free(&mut self) {
        // TODO: Is it possible to make this safe? (the from_raw_parts call)
        if self.owned && !self.is_null() {
            // Create an owned string here from our data just so that ownership
            // is taken over the block of memory we were using and it's properly
            // cleaned up.
            String::from_raw_parts(self.data as *mut _, self.len, self.len);
            self.data = ptr::null();
            self.len = 0;
            self.owned = false;
        }
    }

    pub fn is_null(&self) -> bool {
        self.data == ptr::null()
    }

    pub fn as_str(&self) -> &str {
        // TODO: This is wildly unsafe
        unsafe {
            str::from_utf8_unchecked(
                slice::from_raw_parts(self.data as *const _, self.len))
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn memento_new_str(c: *mut c_char) -> MementoStr {
    let s = CString::from_raw(c).into_string().unwrap();
    MementoStr::from_string(s)
}

#[no_mangle]
pub unsafe extern "C" fn memento_free_str(s: *mut MementoStr) {
    (*s).free();
}

#[allow(unused_variables)]
#[no_mangle]
pub extern "C" fn memento_fetch_path(path: *const MementoStr, from: u64, until: u64) -> MementoResult {
    let our_path = unsafe { (*path).clone() };
    println!("Path: {}", our_path.as_str());

    MementoResult {
        error: MementoErrorCode::NoError,
    }
}
