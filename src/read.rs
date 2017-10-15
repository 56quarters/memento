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

use time::{self, Timespec, Tm, Duration};

use io::MappedFileStream;
use parser::{whisper_parse_header, whisper_parse_archive};
use types::{Header, Point, Archive, ArchiveInfo};
use core::{WhisperResult, WhisperError, ErrorKind};


///
///
///
///
///
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct FetchRequest {
    from: Timespec,
    until: Timespec,
    now: Timespec,
}


impl FetchRequest {
    ///
    ///
    ///
    ///
    ///
    pub fn new(from: Timespec, until: Timespec, now: Timespec) -> FetchRequest {
        FetchRequest {
            from: from,
            until: until,
            now: now,
        }
    }

    ///
    ///
    pub fn with_from(&mut self, val: Timespec) -> &mut Self {
        self.from = val;
        self
    }

    ///
    ///
    pub fn with_from_tm(&mut self, val: Tm) -> &mut Self {
        self.with_from(val.to_timespec())
    }

    ///
    ///
    pub fn with_until(&mut self, val: Timespec) -> &mut Self {
        self.until = val;
        self
    }

    ///
    ///
    pub fn with_until_tm(&mut self, val: Tm) -> &mut Self {
        self.with_until(val.to_timespec())
    }

    ///
    ///
    pub fn with_now(&mut self, val: Timespec) -> &mut Self {
        self.now = val;
        self
    }

    ///
    ///
    pub fn with_now_tm(&mut self, val: Tm) -> &mut Self {
        self.with_now(val.to_timespec())
    }

    ///
    ///
    ///
    ///
    fn normalize(&self, header: &Header) -> WhisperResult<Self> {
        let metadata = header.metadata();
        let oldest = self.now - Duration::seconds(i64::from(metadata.max_retention()));

        // Well, this is just nonsense
        if self.until <= self.from {
            return Err(WhisperError::from(
                (ErrorKind::InvalidInput, "invalid time range"),
            ));
        }

        // start time is in the future, invalid
        if self.from > self.now {
            return Err(WhisperError::from(
                (ErrorKind::InvalidInput, "invalid from time"),
            ));
        }

        // end time is before the oldest value we have, invalid
        if self.until < oldest {
            return Err(WhisperError::from(
                (ErrorKind::InvalidInput, "invalid until time"),
            ));
        }

        // start time is before the oldest value we have, adjust
        let from = if self.from < oldest {
            oldest
        } else {
            self.from
        };

        Ok(FetchRequest::new(from, self.until, self.now))
    }

    ///
    fn retention(&self) -> Duration {
        self.now - self.from
    }
}


impl Default for FetchRequest {
    ///
    ///
    fn default() -> Self {
        let now = time::get_time();
        let from = now - Duration::days(1);
        let until = now;

        FetchRequest::new(from, until, now)
    }
}


///
///
///
///
///
#[derive(Debug)]
pub struct WhisperFileReader {
    mapper: MappedFileStream,
}


impl WhisperFileReader {
    ///
    ///
    ///
    pub fn new(mapper: MappedFileStream) -> WhisperFileReader {
        WhisperFileReader { mapper: mapper }
    }

    ///
    ///
    ///
    pub fn read<P>(&self, path: P, req: &FetchRequest) -> WhisperResult<Vec<Point>>
    where
        P: AsRef<Path>,
    {
        self.mapper.run_immutable(path, |bytes| {
            let reader = WhisperReader::new(bytes);
            reader.read(req)
        })
    }
}


///
///
///
///
///
#[derive(Debug)]
struct WhisperReader<'a> {
    bytes: &'a [u8],
}

impl<'a> WhisperReader<'a> {
    ///
    ///
    fn new(bytes: &'a [u8]) -> WhisperReader<'a> {
        WhisperReader { bytes: bytes }
    }

    ///
    ///
    ///
    fn find_archive<'b, 'c>(
        req: &'b FetchRequest,
        header: &'c Header,
    ) -> WhisperResult<&'c ArchiveInfo> {
        let archives = header.archive_info();
        let required_retention = req.retention();

        for archive in archives {
            let retention = Duration::seconds(i64::from(archive.retention()));
            if retention >= required_retention {
                return Ok(archive);
            }
        }

        Err(WhisperError::from(
            (ErrorKind::InvalidInput, "no archive available"),
        ))
    }

    ///
    ///
    ///
    fn slice_for_archive(&self, archive: &ArchiveInfo) -> WhisperResult<&[u8]> {
        let offset = archive.offset() as usize;
        // These two conditions should never happen but it's nice to handle
        // a corrupted file gracefully here instead of just panicking. This
        // avoids crashing the calling code.
        if offset > self.bytes.len() {
            return Err(WhisperError::from(
                (ErrorKind::ParseError, "offset exceeds data size"),
            ));
        }

        if offset + archive.archive_size() > self.bytes.len() {
            return Err(WhisperError::from(
                (ErrorKind::ParseError, "archive exceeds data size"),
            ));
        }

        Ok(&self.bytes[offset..offset + archive.archive_size()])
    }

    ///
    ///
    ///
    fn points_for_request(archive: &Archive, request: &FetchRequest) -> Vec<Point> {
        archive
            .points()
            .iter()
            .filter(|p| {
                Timespec::new(i64::from(p.timestamp()), 0) >= request.from
            })
            .filter(|p| {
                Timespec::new(i64::from(p.timestamp()), 0) <= request.until
            })
            .cloned()
            .collect()
    }

    ///
    ///
    ///
    fn read(&self, req: &FetchRequest) -> WhisperResult<Vec<Point>> {
        let header = whisper_parse_header(self.bytes).to_full_result()?;
        let req = req.normalize(&header)?;
        let archive_info = Self::find_archive(&req, &header)?;
        let archive_bytes = self.slice_for_archive(archive_info)?;
        let archive = whisper_parse_archive(archive_bytes, archive_info)
            .to_full_result()?;
        Ok(Self::points_for_request(&archive, &req))
    }
}


#[cfg(test)]
mod tests {
    use super::FetchRequest;
    use types::{Header, Metadata, ArchiveInfo, AggregationType};
    use time;

    fn get_file_header() -> Header {
        let metadata = Metadata::new(
            AggregationType::Average,
            3600 * 24 * 30, // max retention
            0.3, // x files factor
            2, // two archives
        );
        let info1 = ArchiveInfo::new(
            (Metadata::storage() + ArchiveInfo::storage() * 2) as u32, // offset
            60, // seconds per point
            60 * 24 * 7, // number of 1 minute points = 60 per hour * 24 hours * 7 days
        );
        let info2 = ArchiveInfo::new(
            info1.offset() + info1.archive_size() as u32, // offset
            300, // seconds per point
            12 * 24 * 30, // number of 5 minute points = 12 per hour * 24 hours * 30 days
        );

        Header::new(metadata, vec![info1, info2])
    }

    // TODO: Should these all be variations of testing the .read() method?

    #[test]
    fn test_find_archive_no_archives() {
        let req = FetchRequest::default();
        let header = get_file_header();

    }

    #[test]
    fn test_find_archive_success() {}

    #[test]
    fn test_slice_for_archive_invalid_offset() {}

    #[test]
    fn test_slice_for_archive_invalid_archive_size() {}

    #[test]
    fn test_slice_for_archive_success() {}

    #[test]
    fn test_points_for_request_all_points_after_from() {}

    #[test]
    fn test_points_for_request_all_points_before_until() {}

    #[test]
    fn test_points_for_request_mixed_data_in_middle() {}

    #[test]
    fn test_points_for_request_success() {}

}
