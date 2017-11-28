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
use std::fs::File;
use std::path::Path;

use fs2::FileExt;
use memmap::{Mmap, Protection};

use memento_core::errors::WhisperResult;

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


///
///
///
///
///
#[derive(Debug, Clone)]
pub struct MappedFileStream {
    locking: bool,
}

// TODO: Need to create a trait for this for testing

impl Default for MappedFileStream {
    fn default() -> Self {
        MappedFileStream {
            locking: true,
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
    use super::MappedFileStream;

    #[test]
    fn test_mapped_file_stream_immutable() {
        let expected: [u8; 1024] = [0; 1024];
        let expected_ref = &expected as &[u8];

        let mapper = MappedFileStream::new();
        let _ = mapper
            .run_immutable("tests/zero_file.bin", |bytes| {
                assert_eq!(expected_ref, bytes);
                Ok(0)
            })
        .unwrap();
    }
}
