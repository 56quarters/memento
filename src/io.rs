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

use std::io;
use std::fs::{File, OpenOptions};
use std::path::Path;

use fs2::FileExt;
use memmap::{Mmap, Protection};

use core::WhisperResult;

struct FileLocker<'a> {
    enabled: bool,
    file: &'a File,
}


impl<'a> FileLocker<'a> {
    fn lock_shared(enabled: bool, file: &'a File) -> io::Result<FileLocker<'a>> {
        if enabled {
            file.lock_shared()?;
        }

        Ok(FileLocker {
            enabled: enabled,
            file: file,
        })
    }

    fn lock_exclusive(enabled: bool, file: &'a File) -> io::Result<FileLocker<'a>> {
        if enabled {
            file.lock_exclusive()?;
        }

        Ok(FileLocker {
            enabled: enabled,
            file: file,
        })
    }
}


impl<'a> Drop for FileLocker<'a> {
    fn drop(&mut self) {
        if !self.enabled {
            return;
        }

        // Try to unlock but if it fails there's really nothing to do about
        // it. Probably don't want to be logging from a library and we can't
        // panic or return an Err from a destructor.
        match self.file.unlock() {
            _ => (),
        };
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FlushBehavior {
    NoFlush,
    Flush,
    FlushAsync,
}


///
///
///
///
///
#[derive(Debug)]
pub struct MappedFileStream {
    locking: bool,
    flushing: FlushBehavior,
}

// TODO: How to communicate flush range back from consumer?

// TODO: Need to create a trait for this for testing

impl Default for MappedFileStream {
    fn default() -> Self {
        MappedFileStream {
            locking: true,
            flushing: FlushBehavior::Flush,
        }
    }
}


impl MappedFileStream {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_locking(&mut self, locking: bool) -> &mut Self {
        self.locking = locking;
        self
    }

    pub fn with_flushing(&mut self, flushing: FlushBehavior) -> &mut Self {
        self.flushing = flushing;
        self
    }

    pub fn run_mutable<P, F, T>(&mut self, path: P, consumer: F) -> WhisperResult<T>
    where
        P: AsRef<Path>,
        F: Fn(&mut [u8]) -> WhisperResult<T>,
    {
        let file = OpenOptions::new().read(true).write(true).open(path)?;
        // Not used, we just need it to unlock the file in its destructor
        let _locker = FileLocker::lock_exclusive(self.locking, &file)?;

        let mut mmap = Mmap::open(&file, Protection::ReadWrite)?;
        let res = {
            // Unsafe is OK here since we've obtained an exclusive (write) lock
            let bytes = unsafe { mmap.as_mut_slice() };
            consumer(bytes)?
        };

        match self.flushing {
            FlushBehavior::Flush => mmap.flush()?,
            FlushBehavior::FlushAsync => mmap.flush_async()?,
            _ => (),
        };

        Ok(res)
    }

    pub fn run_immutable<P, F, T>(&self, path: P, consumer: F) -> WhisperResult<T>
    where
        P: AsRef<Path>,
        F: Fn(&[u8]) -> WhisperResult<T>,
    {
        let file = File::open(path)?;
        // Not used, we just need it to unlock the file in its destructor
        let _locker = FileLocker::lock_shared(self.locking, &file)?;

        let mmap = Mmap::open(&file, Protection::Read)?;
        let res = {
            // Unsafe is OK here since we've obtained a shared (read) lock
            let bytes = unsafe { mmap.as_slice() };
            consumer(bytes)?
        };

        Ok(res)
    }
}

#[cfg(test)]
mod tests {



}
