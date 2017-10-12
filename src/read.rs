// Whisper
//
// Copyright 2017 TSH Labs
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//!

use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use io::MappedFileStream;
use parser::{whisper_parse_header, whisper_parse_archive};
use types::{Header, Point, Archive, ArchiveInfo};
use core::{WhisperResult, WhisperError, ErrorKind};


fn get_seconds_since_epoch() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .expect(concat!(
            "Unable to determine the number of seconds since UNIX Epoch. ",
            "Please ensure sytem time is set correctly"
        ))
}


pub struct FetchRequest {
    from: u64,
    until: u64,
    now: u64,
}


impl FetchRequest {
    pub fn new() -> FetchRequest {
        let now = get_seconds_since_epoch();
        let from = now - 24 * 60 * 60;
        let until = now;

        FetchRequest {
            from: from,
            until: until,
            now: now,
        }
    }

    pub fn with_from(&mut self, val: u64) -> &mut Self {
        self.from = val;
        self
    }

    pub fn with_until(&mut self, val: u64) -> &mut Self {
        self.until = val;
        self
    }

    pub fn with_now(&mut self, val: u64) -> &mut Self {
        self.now = val;
        self
    }

    fn validate(&self, header: &Header) -> WhisperResult<()> {
        let metadata = header.metadata();
        let oldest = self.now - u64::from(metadata.max_retention());

        // Well, this is just nonsense
        if self.until <= self.from {
            return Err(WhisperError::from(
                (ErrorKind::InvalidInput, "invalid time range"),
            ));
        }

        // start time is in the future, invalid
        if self.from > self.now {
            return Err(WhisperError::from(
                (ErrorKind::InvalidInput, "invalid time range"),
            ));
        }

        // end time is before the oldest value we have, invalid
        if self.until < oldest {
            return Err(WhisperError::from(
                (ErrorKind::InvalidInput, "invalid time range"),
            ));
        }

        // start time is before the oldest value we have, adjust
        if self.from < oldest {
            //self.from = oldest;
        }

        Ok(())
    }

    fn retention(&self) -> u64 {
        self.now - self.from
    }
}


impl Default for FetchRequest {
    /// Default request is for the previous 24 hours of data
    fn default() -> Self {
        let now = get_seconds_since_epoch();
        let from = now - 24 * 60 * 60;
        let until = now;

        FetchRequest {
            from: from,
            until: until,
            now: now,
        }
    }
}


pub struct WhisperReader {
    mapper: MappedFileStream,
}


impl WhisperReader {
    pub fn new(mapper: MappedFileStream) -> WhisperReader {
        WhisperReader {
            mapper: mapper
        }
    }

    fn get_archive_to_use<'a, 'b>(
        req: &'a FetchRequest,
        header: &'b Header
    ) -> WhisperResult<&'b ArchiveInfo> {

        let archives = header.archive_info();
        let required_retention = req.retention();

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
        archive: &'b ArchiveInfo
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

    fn get_points_for_request(archive: &Archive, request: &FetchRequest) -> Vec<Point> {
        archive.points().iter()
            .filter(|p| u64::from(p.timestamp()) >= request.from)
            .filter(|p| u64::from(p.timestamp()) <= request.until)
            .map(|p| p.clone())
            .collect()
    }

    pub fn read<P>(&self, path: P, req: &FetchRequest) -> WhisperResult<Vec<Point>>
    where
        P: AsRef<Path>
    {
        self.mapper.run_immutable(path, |bytes| {
            let header = whisper_parse_header(bytes).to_full_result()?;
            req.validate(&header)?;

            let archive_info = Self::get_archive_to_use(&req, &header)?;
            let archive_bytes = Self::get_slice_for_archive(bytes, archive_info)?;
            let archive = whisper_parse_archive(archive_bytes, archive_info)
                .to_full_result()?;
            Ok(Self::get_points_for_request(&archive, &req))
        })
    }
}
