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

use parser::{whisper_parse_header, whisper_parse_file, whisper_parse_archive};
use types::{WhisperFile, Header, Point, Archive, ArchiveInfo};
use core::{WhisperResult, WhisperError, ErrorKind};


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


#[derive(Debug)]
pub struct MappedFileStream {
    locking: bool,
    flushing: FlushBehavior,
}


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


use std::time::{SystemTime, Duration, UNIX_EPOCH};


pub fn get_duration_since_epoch() -> Option<Duration> {
    SystemTime::now().duration_since(UNIX_EPOCH).ok()
}


pub fn get_seconds_since_epoch() -> Option<u64> {
    get_duration_since_epoch().map(|d| d.as_secs())
}


// TODO: This should be part of the public API. Maybe? Builder pattern?
// validation happens in the .build() call or something else internal to
// the *fetch() function
struct FetchRequest {
    from: u64,
    until: u64,
    now: u64,
}


impl FetchRequest {
    fn from(mut from: u64, until: u64, now: u64, header: &Header) -> WhisperResult<FetchRequest> {
        let metadata = header.metadata();
        let oldest = now - u64::from(metadata.max_retention());

        // start time is in the future, invalid
        if from > now {
            return Err(WhisperError::from(
                (ErrorKind::InvalidInput, "invalid time range"),
            ));
        }

        // end time is before the oldest value we have, invalid
        if until < oldest {
            return Err(WhisperError::from(
                (ErrorKind::InvalidInput, "invalid time range"),
            ));
        }

        // start time is before the oldest value we have, adjust
        if from < oldest {
            from = oldest;
        }

        Ok(FetchRequest {
            from: from,
            until: until,
            now: now,
        })
    }

    fn retention(&self) -> u64 {
        self.now - self.from
    }
}


fn get_archive_to_use<'a, 'b>(
    request: &'a FetchRequest,
    header: &'b Header,
) -> WhisperResult<&'b ArchiveInfo> {
    let archives = header.archive_info();
    let required_retention = request.retention();

    for archive in archives {
        if u64::from(archive.retention()) >= required_retention {
            return Ok(archive);
        }
    }

    Err(WhisperError::from(
        (ErrorKind::InvalidInput, "no archive available"),
    ))
}

fn get_slice_for_archive<'a, 'b>(
    bytes: &'a [u8],
    archive: &'b ArchiveInfo,
) -> WhisperResult<&'a [u8]> {
    let offset = archive.offset() as usize;

    // These two conditions should never happen but it's nice to handle
    // a corrupted file gracefully here instead of just panicking. This
    // avoids crashing the calling code.
    if offset > bytes.len() {
        return Err(WhisperError::from(
            (ErrorKind::ParseError, "offset exceeds data size"),
        ));
    }

    if offset + archive.archive_size() > bytes.len() {
        return Err(WhisperError::from(
            (ErrorKind::ParseError, "archive exceeds data size"),
        ));
    }

    Ok(&bytes[offset..offset + archive.archive_size()])
}


fn get_points_for_request(
    info: &ArchiveInfo,
    archive: &Archive,
    request: &FetchRequest
) -> Vec<Point> {
    let expected = (request.until - request.from) / u64::from(info.seconds_per_point()) + 1;
    let mut out = Vec::with_capacity(expected as usize);

    println!("Expected num: {}", expected);
    for p in archive.points() {
        let t = u64::from(p.timestamp());

        if t >= request.from {
            if t <= request.until {
                out.push(p.clone());
            } else {
                // timestamp of the current point is past the end timestamp we're
                // interested in. all further points will be too so just bail early
                break;
            }
        }
    }

    println!("Actual num: {}", out.len());

    out
}


pub fn whisper_fetch_points<P>(path: P, from: u64, until: u64) -> WhisperResult<Vec<Point>>
where
    P: AsRef<Path>,
{
    let now = get_seconds_since_epoch().unwrap();
    whisper_fetch_points_with_now(path, from, until, now)
}


pub fn whisper_fetch_points_with_now<P>(
    path: P,
    from: u64,
    until: u64,
    now: u64,
) -> WhisperResult<Vec<Point>>
where
    P: AsRef<Path>,
{
    // Well, this is just nonsense
    if until <= from {
        return Err(WhisperError::from(
            (ErrorKind::InvalidInput, "invalid time range"),
        ));
    }

    let runner = MappedFileStream::new();
    runner.run_immutable(path, |bytes| {
        let header = whisper_parse_header(bytes).to_full_result()?;
        let request = FetchRequest::from(from, until, now, &header)?;
        let archive_info = get_archive_to_use(&request, &header)?;
        let archive_bytes = get_slice_for_archive(bytes, archive_info)?;
        let archive = whisper_parse_archive(archive_bytes, archive_info)
            .to_full_result()?;
        Ok(get_points_for_request(archive_info, &archive, &request))
    })
}


#[cfg(test)]
mod tests {}
