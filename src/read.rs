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
        // validate the that requested ranges are something that we can
        // satisfy with this database and coerce them if required. For
        // example: bump up the starting range to our earliest range if
        // that's the only thing preventing us from handling this request.
        let req = req.normalize(&header)?;
        let archive_info = Self::find_archive(&req, &header)?;
        // Get the section of the mmaped file that we should be looking
        // at based on the archive that can actually be used to satisfy
        // the requested ranges.
        let archive_bytes = self.slice_for_archive(archive_info)?;
        let archive = whisper_parse_archive(archive_bytes, archive_info)
            .to_full_result()?;
        Ok(Self::points_for_request(&archive, &req))
    }
}


#[cfg(test)]
mod tests {
    use super::{FetchRequest, WhisperReader};
    use encoder::{whisper_encode_header, whisper_encode_archive};
    use types::{Header, Metadata, ArchiveInfo, AggregationType, Archive, Point};
    use core::ErrorKind;
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
            60 * 24 * 1, // number of 1 minute points = 60 per hour * 24 hours * 1 days
        );
        let info2 = ArchiveInfo::new(
            info1.offset() + info1.archive_size() as u32, // offset
            300, // seconds per point
            12 * 24 * 7, // number of 5 minute points = 12 per hour * 24 hours * 7 days
        );

        Header::new(metadata, vec![info1, info2])
    }

    fn get_archive(info: &ArchiveInfo, now: time::Tm) -> Archive {
        let start_secs = now.to_timespec().sec as u32 - info.retention();

        let vals = (0..info.num_points())
            .map(|i| start_secs + (i * info.seconds_per_point()))
            .map(|t| Point::new(t, 7.0))
            .collect();

        Archive::new(vals)
    }

    #[test]
    fn test_read_no_archives_for_request() {
        let now = time::strptime("1997-08-27T02:14:00", "%Y-%m-%dT%H:%M:%S").unwrap();
        let from = time::strptime("1997-08-04T12:00:00", "%Y-%m-%dT%H:%M:%S").unwrap();
        let until = time::strptime("1997-08-07T12:00:00", "%Y-%m-%dT%H:%M:%S").unwrap();

        let header = get_file_header();

        let mut req = FetchRequest::default();
        req.with_now_tm(now)
            .with_from_tm(from)
            .with_until_tm(until);

        let mut buf = vec![];
        whisper_encode_header(&mut buf, &header).unwrap();
        buf.shrink_to_fit();

        let reader = WhisperReader::new(&buf);
        let res = reader.read(&req);

        assert!(res.is_err());
        assert_eq!(ErrorKind::InvalidInput, res.unwrap_err().kind());
    }

    #[test]
    fn test_read_invalid_archive_offset() {
        let now = time::strptime("1997-08-27T02:14:00", "%Y-%m-%dT%H:%M:%S").unwrap();
        let from = time::strptime("1997-08-26T12:00:00", "%Y-%m-%dT%H:%M:%S").unwrap();
        let until = time::strptime("1997-08-26T18:00:00", "%Y-%m-%dT%H:%M:%S").unwrap();

        let header = get_file_header();

        let mut req = FetchRequest::default();
        req.with_now_tm(now)
            .with_from_tm(from)
            .with_until_tm(until);

        let mut buf = vec![];
        whisper_encode_header(&mut buf, &header).unwrap();
        buf.shrink_to_fit();

        // The buffer only contains the bytes for the header so the offset
        // of the first archive will violate the first check for the size
        // of the data (making sure it's greater than the offset).
        let reader = WhisperReader::new(&buf);
        let res = reader.read(&req);

        assert!(res.is_err());
        assert_eq!(ErrorKind::ParseError, res.unwrap_err().kind());
    }

    #[test]
    fn test_read_invalid_archive_size() {
        let now = time::strptime("1997-08-27T02:14:00", "%Y-%m-%dT%H:%M:%S").unwrap();
        let from = time::strptime("1997-08-26T12:00:00", "%Y-%m-%dT%H:%M:%S").unwrap();
        let until = time::strptime("1997-08-26T18:00:00", "%Y-%m-%dT%H:%M:%S").unwrap();

        let header = get_file_header();
        let archive1 = get_archive(&header.archive_info()[0], now);
        let archive2 = get_archive(&header.archive_info()[1], now);

        let mut req = FetchRequest::default();
        req.with_now_tm(now)
            .with_from_tm(from)
            .with_until_tm(until);

        let mut buf = vec![];
        whisper_encode_header(&mut buf, &header).unwrap();
        whisper_encode_archive(&mut buf, &archive1).unwrap();
        whisper_encode_archive(&mut buf, &archive2).unwrap();
        buf.shrink_to_fit();

        // The buffer will contain and entire file but we only give the reader
        // slightly more than just the header here so that we make sure to test
        // the case where the header lies!
        let reader = WhisperReader::new(&buf[0..80]);
        let res = reader.read(&req);

        assert!(res.is_err());
        assert_eq!(ErrorKind::ParseError, res.unwrap_err().kind());
    }

    #[test]
    fn test_read_all_points_before_from() {
        let now = time::strptime("1997-08-27T02:14:00", "%Y-%m-%dT%H:%M:%S").unwrap();
        let from = time::strptime("1997-08-26T12:00:00", "%Y-%m-%dT%H:%M:%S").unwrap();
        let until = time::strptime("1997-08-26T18:00:00", "%Y-%m-%dT%H:%M:%S").unwrap();

        let header = get_file_header();
        // Pick a start point for the points in each archive to ensure that the
        // high resolution data (in archive 1, last day) will all be too old to
        // satisfy the request since it's all before the "from" time for the request
        let start = time::strptime("1997-08-20T12:00:00", "%Y-%m-%dT%H:%M:%S").unwrap();
        let archive1 = get_archive(&header.archive_info()[0], start);
        let archive2 = get_archive(&header.archive_info()[1], start);

        let mut req = FetchRequest::default();
        req.with_now_tm(now)
            .with_from_tm(from)
            .with_until_tm(until);

        let mut buf = vec![];
        whisper_encode_header(&mut buf, &header).unwrap();
        whisper_encode_archive(&mut buf, &archive1).unwrap();
        whisper_encode_archive(&mut buf, &archive2).unwrap();
        buf.shrink_to_fit();

        let reader = WhisperReader::new(&buf);
        let res = reader.read(&req);

        assert!(res.is_ok());
        assert!(res.unwrap().is_empty());
    }

    #[test]
    fn test_read_all_points_after_until() {
        let now = time::strptime("1997-08-27T02:14:00", "%Y-%m-%dT%H:%M:%S").unwrap();
        let from = time::strptime("1997-08-26T12:00:00", "%Y-%m-%dT%H:%M:%S").unwrap();
        let until = time::strptime("1997-08-26T18:00:00", "%Y-%m-%dT%H:%M:%S").unwrap();

        let header = get_file_header();
        // Pick a start point for the points in each archive to ensure that the
        // high resolution data (in archive 1, last day) will all be too recent to
        // satisfy the request since it's all after the "until" time for the request
        let start = time::strptime("1997-08-27T18:05:00", "%Y-%m-%dT%H:%M:%S").unwrap();
        let archive1 = get_archive(&header.archive_info()[0], start);
        let archive2 = get_archive(&header.archive_info()[1], start);

        let mut req = FetchRequest::default();
        req.with_now_tm(now)
            .with_from_tm(from)
            .with_until_tm(until);

        let mut buf = vec![];
        whisper_encode_header(&mut buf, &header).unwrap();
        whisper_encode_archive(&mut buf, &archive1).unwrap();
        whisper_encode_archive(&mut buf, &archive2).unwrap();
        buf.shrink_to_fit();

        let reader = WhisperReader::new(&buf);
        let res = reader.read(&req);

        assert!(res.is_ok());
        assert!(res.unwrap().is_empty());
    }

    #[test]
    fn test_read_success_high_resolution() {
        let now = time::strptime("1997-08-27T02:14:00", "%Y-%m-%dT%H:%M:%S").unwrap();
        let from = time::strptime("1997-08-26T12:00:00", "%Y-%m-%dT%H:%M:%S").unwrap();
        let until = time::strptime("1997-08-26T18:01:00", "%Y-%m-%dT%H:%M:%S").unwrap();

        let header = get_file_header();
        // Pick a start time here that ensures that two points in the higher resolution
        // archive overlap with the requested range (right on the tail end of the "until"
        // time).
        let start = time::strptime("1997-08-27T18:00:00", "%Y-%m-%dT%H:%M:%S").unwrap();
        let archive1 = get_archive(&header.archive_info()[0], start);
        let archive2 = get_archive(&header.archive_info()[1], start);

        let mut req = FetchRequest::default();
        req.with_now_tm(now)
            .with_from_tm(from)
            .with_until_tm(until);

        let mut buf = vec![];
        whisper_encode_header(&mut buf, &header).unwrap();
        whisper_encode_archive(&mut buf, &archive1).unwrap();
        whisper_encode_archive(&mut buf, &archive2).unwrap();
        buf.shrink_to_fit();

        let reader = WhisperReader::new(&buf);
        let res = reader.read(&req);

        assert!(res.is_ok());

        let points = res.unwrap();
        assert_eq!(2, points.len());
        assert_eq!(
            &vec![
                Point::new(until.to_timespec().sec as u32 - 60, 7.0),
                Point::new(until.to_timespec().sec as u32, 7.0)
            ],
            &points
        );
    }

    #[test]
    fn test_read_success_low_resolution() {
        let now = time::strptime("1997-08-27T02:14:00", "%Y-%m-%dT%H:%M:%S").unwrap();
        let from = time::strptime("1997-08-20T12:00:00", "%Y-%m-%dT%H:%M:%S").unwrap();
        let until = time::strptime("1997-08-20T18:05:00", "%Y-%m-%dT%H:%M:%S").unwrap();

        let header = get_file_header();
        // Pick a start time here that ensures that two points in the lower resolution
        // archive overlap with the requested range (right on the tail end of the "until"
        // time).
        let start = time::strptime("1997-08-27T18:00:00", "%Y-%m-%dT%H:%M:%S").unwrap();
        let archive1 = get_archive(&header.archive_info()[0], start);
        let archive2 = get_archive(&header.archive_info()[1], start);

        let mut req = FetchRequest::default();
        req.with_now_tm(now)
            .with_from_tm(from)
            .with_until_tm(until);

        let mut buf = vec![];
        whisper_encode_header(&mut buf, &header).unwrap();
        whisper_encode_archive(&mut buf, &archive1).unwrap();
        whisper_encode_archive(&mut buf, &archive2).unwrap();
        buf.shrink_to_fit();

        let reader = WhisperReader::new(&buf);
        let res = reader.read(&req);

        assert!(res.is_ok());

        let points = res.unwrap();
        assert_eq!(2, points.len());
        assert_eq!(
            &vec![
                Point::new(until.to_timespec().sec as u32 - 300, 7.0),
                Point::new(until.to_timespec().sec as u32, 7.0)
            ],
            &points
        );
    }
}
