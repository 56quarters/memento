// Whisper
//
// Copyright 2017 TSH Labs
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Functions to read and write parts of the Whisper format to disk

use std::fs::File;
use std::path::Path;

use fs2::FileExt;
use memmap::{Mmap, Protection};

use parser::{whisper_parse_header, whisper_parse_file};
use types::{WhisperFile, Header};
use core::WhisperResult;


// TODO: Explain impl: mmap vs regular I/O (how locks factor in)
pub fn whisper_read_header<P>(path: P) -> WhisperResult<Header>
where
    P: AsRef<Path>,
{
    let file = File::open(path)?;
    file.lock_shared()?;

    let mmap = Mmap::open(&file, Protection::Read)?;
    // Unsafe is OK here since we've obtained a shared (read) lock
    let bytes = unsafe { mmap.as_slice() };
    let res = whisper_parse_header(bytes).to_full_result()?;

    file.unlock()?;
    Ok(res)
}


// TODO: Explain impl: small bufs, vs big buf, vs mmap
pub fn whisper_read_file<P>(path: P) -> WhisperResult<WhisperFile>
where
    P: AsRef<Path>,
{
    // Provide some entry-like API?
    // let res = wrapper.with_slice(|bytes| {
    //     do a bunch of stuff
    // })
    let file = File::open(path)?;
    file.lock_shared()?;

    // Potential extension: madvise(sequential). Didn't seem to make difference
    // in benchmarks but maybe real world use is different
    let mmap = Mmap::open(&file, Protection::Read)?;
    // Unsafe is OK here since we've obtained a shared (read) lock
    let bytes = unsafe { mmap.as_slice() };
    let res = whisper_parse_file(bytes).to_full_result()?;

    file.unlock()?;
    Ok(res)
}


#[cfg(test)]
mod tests {}
