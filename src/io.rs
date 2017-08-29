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

use std::fs::{File, OpenOptions};
use std::path::Path;

use fs2::FileExt;
use memmap::{Mmap, Protection};

use parser::{whisper_parse_header, whisper_parse_file};
use types::{WhisperFile, Header};
use core::WhisperResult;


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FlushBehavior {
    NoFlush,
    Flush,
    FlushAsync,
}


#[derive(Debug)]
pub struct MappedFileStream {
    locking: bool,
    flushing: FlushBehavior,
}


impl Default for MappedFileStream {
    fn default() -> Self {
        MappedFileStream { locking: true, flushing: FlushBehavior::Flush }
    }
}


impl MappedFileStream {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn locking(mut self, locking: bool) -> Self {
        self.locking = locking;
        self
    }

    pub fn flushing(mut self, flushing: FlushBehavior) -> Self {
        self.flushing = flushing;
        self
    }

    pub fn run_mutable<P, F, T>(&mut self, path: P, consumer: F) -> WhisperResult<T>
    where
        P: AsRef<Path>,
        F: Fn(&mut [u8]) -> WhisperResult<T>,
    {
        let file = OpenOptions::new().read(true).write(true).open(path)?;
        if self.locking {
            file.lock_exclusive()?;
        }

        let mut mmap = Mmap::open(&file, Protection::ReadWrite)?;
        let res = {
            // Unsafe is OK here since we've obtained an exclusive (write) lock
            let bytes = unsafe { mmap.as_mut_slice() };
            consumer(bytes)?
        };

        match self.flushing {
            FlushBehavior::Flush => mmap.flush()?,
            FlushBehavior::FlushAsync => mmap.flush_async()?,
            _ => ()
        };

        if self.locking {
            file.unlock()?;
        }

        Ok(res)

    }

    pub fn run_immutable<P, F, T>(&self, path: P, consumer: F) -> WhisperResult<T>
    where
        P: AsRef<Path>,
        F: Fn(&[u8]) -> WhisperResult<T>,
    {
        let file = File::open(path)?;
        if self.locking {
            file.lock_shared()?;
        }

        let mmap = Mmap::open(&file, Protection::Read)?;
        let res = {
            // Unsafe is OK here since we've obtained a shared (read) lock
            let bytes = unsafe { mmap.as_slice() };
            consumer(bytes)?
        };

        if self.locking {
            file.unlock()?;
        }

        Ok(res)
    }
}

// TODO: Explain impl: mmap vs regular I/O (how locks factor in)
pub fn whisper_read_header<P>(path: P) -> WhisperResult<Header>
where
    P: AsRef<Path>,
{
    let runner = MappedFileStream::new();
    runner.run_immutable(path, |bytes| {
        Ok(whisper_parse_header(bytes).to_full_result()?)
    })
}


// TODO: Explain impl: small bufs, vs big buf, vs mmap
pub fn whisper_read_file<P>(path: P) -> WhisperResult<WhisperFile>
where
    P: AsRef<Path>,
{
    let runner = MappedFileStream::new();
    runner.run_immutable(path, |bytes| {
        Ok(whisper_parse_file(bytes).to_full_result()?)
    })
}


#[cfg(test)]
mod tests {}
