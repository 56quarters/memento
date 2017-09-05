// Whisper
//
// Copyright 2017 TSH Labs
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Utility methods that don't fit anywhere else

use std::time::{SystemTime, Duration, UNIX_EPOCH};


pub fn get_duration_since_epoch() -> Option<Duration> {
    SystemTime::now().duration_since(UNIX_EPOCH).ok()
}


pub fn get_seconds_since_epoch() -> Option<u64> {
    get_duration_since_epoch().map(|d| d.as_secs())
}
