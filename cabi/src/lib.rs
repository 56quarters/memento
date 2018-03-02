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

mod common;
mod header;
mod points;

// Just reuse our existing aggreation enum
pub use memento::types::AggregationType;

pub use common::MementoErrorCode;
pub use header::{memento_header_fetch, memento_header_free, memento_header_is_error,
                 MementoArchiveInfo, MementoHeader, MementoHeaderResult, MementoMetadata};
pub use points::{memento_points_fetch, memento_points_fetch_full, memento_points_free,
                 memento_points_is_error, MementoPoint, MementoPointsResult};
