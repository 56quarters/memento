// Memento - A Whisper implementation in Rust
//
// Copyright 2017-2018 TSH Labs
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

extern crate chrono;
extern crate fs2;
extern crate memento_core;
extern crate memmap;

mod read;
mod write;
mod io;

pub use memento_core::types;
pub use memento_core::errors;
pub use read::{FetchRequest, MementoFileReader};
pub use io::MappedFileStream;
