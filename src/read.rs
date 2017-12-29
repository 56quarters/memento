// Memento - A Whisper implemention in Rust
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

use chrono::{DateTime, Duration, TimeZone, Utc};

use io::MappedFileStream;
use memento_core::parser::{memento_parse_archive, memento_parse_header};
use memento_core::types::{Archive, ArchiveInfo, Header, Point};
use memento_core::errors::{ErrorKind, MementoError, MementoResult};


///
///
///
///
///
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct FetchRequest {
    from: DateTime<Utc>,
    until: DateTime<Utc>,
    now: DateTime<Utc>,
}


impl FetchRequest {
    ///
    ///
    ///
    ///
    ///
    pub fn new<T>(from: DateTime<T>, until: DateTime<T>, now: DateTime<T>) -> FetchRequest
    where
        T: TimeZone,
    {
        FetchRequest {
            from: from.with_timezone(&Utc),
            until: until.with_timezone(&Utc),
            now: now.with_timezone(&Utc),
        }
    }

    ///
    ///
    pub fn with_from<T>(mut self, val: DateTime<T>) -> Self
    where
        T: TimeZone,
    {
        self.from = val.with_timezone(&Utc);
        self
    }

    ///
    ///
    pub fn with_until<T>(mut self, val: DateTime<T>) -> Self
    where
        T: TimeZone,
    {
        self.until = val.with_timezone(&Utc);
        self
    }

    ///
    ///
    pub fn with_now<T>(mut self, val: DateTime<T>) -> Self
    where
        T: TimeZone,
    {
        self.now = val.with_timezone(&Utc);
        self
    }

    ///
    ///
    ///
    ///
    fn normalize(&self, header: &Header) -> MementoResult<Self> {
        let metadata = header.metadata();
        let oldest = self.now - Duration::seconds(i64::from(metadata.max_retention()));

        // Well, this is just nonsense
        if self.until <= self.from {
            return Err(MementoError::from((
                ErrorKind::InvalidTimeRange,
                "invalid time range",
            )));
        }

        // start time is in the future, invalid
        if self.from > self.now {
            return Err(MementoError::from((
                ErrorKind::InvalidTimeStart,
                "invalid from time",
            )));
        }

        // end time is before the oldest value we have, invalid
        if self.until < oldest {
            return Err(MementoError::from((
                ErrorKind::InvalidTimeEnd,
                "invalid until time",
            )));
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
        self.now.signed_duration_since(self.from)
    }
}


impl Default for FetchRequest {
    ///
    ///
    fn default() -> Self {
        let now = Utc::now();
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
#[derive(Debug, Clone)]
pub struct MementoFileReader {
    mapper: MappedFileStream,
}


impl MementoFileReader {
    ///
    ///
    ///
    pub fn new(mapper: MappedFileStream) -> MementoFileReader {
        MementoFileReader { mapper: mapper }
    }

    ///
    ///
    ///
    pub fn read<P>(&self, path: P, req: &FetchRequest) -> MementoResult<Vec<Point>>
    where
        P: AsRef<Path>,
    {
        self.mapper.run_immutable(path, |bytes| {
            let reader = MementoReader::new(bytes);
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
struct MementoReader<'a> {
    bytes: &'a [u8],
}

impl<'a> MementoReader<'a> {
    ///
    ///
    fn new(bytes: &'a [u8]) -> MementoReader<'a> {
        MementoReader { bytes: bytes }
    }

    ///
    ///
    ///
    fn find_archive<'b, 'c>(
        req: &'b FetchRequest,
        header: &'c Header,
    ) -> MementoResult<&'c ArchiveInfo> {
        let archives = header.archive_info();
        let required_retention = req.retention();

        for archive in archives {
            let retention = Duration::seconds(i64::from(archive.retention()));
            if retention >= required_retention {
                return Ok(archive);
            }
        }

        Err(MementoError::from((
            ErrorKind::NoArchiveAvailable,
            "no archive available",
        )))
    }

    ///
    ///
    ///
    fn slice_for_archive(&self, archive: &ArchiveInfo) -> MementoResult<&[u8]> {
        let offset = archive.offset() as usize;
        // These two conditions should never happen but it's nice to handle
        // a corrupted file gracefully here instead of just panicking. This
        // avoids crashing the calling code.
        if offset > self.bytes.len() {
            return Err(MementoError::from((
                ErrorKind::CorruptDatabase,
                "offset exceeds data size",
            )));
        }

        if offset + archive.archive_size() > self.bytes.len() {
            return Err(MementoError::from((
                ErrorKind::CorruptDatabase,
                "archive exceeds data size",
            )));
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
            .filter(|p| Utc.timestamp(i64::from(p.timestamp()), 0) >= request.from)
            .filter(|p| Utc.timestamp(i64::from(p.timestamp()), 0) <= request.until)
            .cloned()
            .collect()
    }

    ///
    ///
    ///
    fn read(&self, req: &FetchRequest) -> MementoResult<Vec<Point>> {
        let header = memento_parse_header(self.bytes).to_full_result()?;
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
        let archive = memento_parse_archive(archive_bytes, archive_info).to_full_result()?;
        Ok(Self::points_for_request(&archive, &req))
    }
}


#[cfg(test)]
mod tests {
    use chrono::{DateTime, Utc};

    use memento_core::errors::ErrorKind;
    use memento_core::encoder::{memento_encode_archive, memento_encode_header};
    use memento_core::types::{AggregationType, Archive, ArchiveInfo, Header, Metadata, Point};

    use super::{FetchRequest, MementoReader};

    fn get_file_header() -> Header {
        let metadata = Metadata::new(
            AggregationType::Average,
            3600 * 24 * 30, // max retention
            0.3,            // x files factor
            2,              // two archives
        );
        let info1 = ArchiveInfo::new(
            (Metadata::storage() + ArchiveInfo::storage() * 2) as u32, // offset
            60,                                                        // seconds per point
            60 * 24 * 1, // number of 1 minute points = 60 per hour * 24 hours * 1 days
        );
        let info2 = ArchiveInfo::new(
            info1.offset() + info1.archive_size() as u32, // offset
            300,                                          // seconds per point
            12 * 24 * 7, // number of 5 minute points = 12 per hour * 24 hours * 7 days
        );

        Header::new(metadata, vec![info1, info2])
    }

    fn get_archive(info: &ArchiveInfo, now: DateTime<Utc>) -> Archive {
        let start_secs = now.timestamp() as u32 - info.retention();

        let vals = (0..info.num_points())
            .map(|i| start_secs + (i * info.seconds_per_point()))
            .map(|t| Point::new(t, 7.0))
            .collect();

        Archive::new(vals)
    }

    fn parse_utc(val: &str) -> DateTime<Utc> {
        DateTime::parse_from_str(val, "%Y-%m-%dT%H:%M:%S%z")
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap()
    }

    #[test]
    fn test_read_no_archives_for_request() {
        let now = parse_utc("1997-08-27T02:14:00+0000");
        let from = parse_utc("1997-08-04T12:00:00+0000");
        let until = parse_utc("1997-08-07T12:00:00+0000");

        let header = get_file_header();

        let req = FetchRequest::default()
            .with_now(now)
            .with_from(from)
            .with_until(until);

        let mut buf = vec![];
        memento_encode_header(&mut buf, &header).unwrap();
        buf.shrink_to_fit();

        let reader = MementoReader::new(&buf);
        let res = reader.read(&req);

        assert!(res.is_err());
        assert_eq!(ErrorKind::NoArchiveAvailable, res.unwrap_err().kind());
    }

    #[test]
    fn test_read_invalid_archive_offset() {
        let now = parse_utc("1997-08-27T02:14:00+0000");
        let from = parse_utc("1997-08-26T12:00:00+0000");
        let until = parse_utc("1997-08-26T18:00:00+0000");

        let header = get_file_header();

        let req = FetchRequest::default()
            .with_now(now)
            .with_from(from)
            .with_until(until);

        let mut buf = vec![];
        memento_encode_header(&mut buf, &header).unwrap();
        buf.shrink_to_fit();

        // The buffer only contains the bytes for the header so the offset
        // of the first archive will violate the first check for the size
        // of the data (making sure it's greater than the offset).
        let reader = MementoReader::new(&buf);
        let res = reader.read(&req);

        assert!(res.is_err());
        assert_eq!(ErrorKind::CorruptDatabase, res.unwrap_err().kind());
    }

    #[test]
    fn test_read_invalid_archive_size() {
        let now = parse_utc("1997-08-27T02:14:00+0000");
        let from = parse_utc("1997-08-26T12:00:00+0000");
        let until = parse_utc("1997-08-26T18:00:00+0000");

        let header = get_file_header();
        let archive1 = get_archive(&header.archive_info()[0], now);
        let archive2 = get_archive(&header.archive_info()[1], now);

        let req = FetchRequest::default()
            .with_now(now)
            .with_from(from)
            .with_until(until);

        let mut buf = vec![];
        memento_encode_header(&mut buf, &header).unwrap();
        memento_encode_archive(&mut buf, &archive1).unwrap();
        memento_encode_archive(&mut buf, &archive2).unwrap();
        buf.shrink_to_fit();

        // The buffer will contain and entire file but we only give the reader
        // slightly more than just the header here so that we make sure to test
        // the case where the header lies!
        let reader = MementoReader::new(&buf[0..80]);
        let res = reader.read(&req);

        assert!(res.is_err());
        assert_eq!(ErrorKind::CorruptDatabase, res.unwrap_err().kind());
    }

    #[test]
    fn test_read_all_points_before_from() {
        let now = parse_utc("1997-08-27T02:14:00+0000");
        let from = parse_utc("1997-08-26T12:00:00+0000");
        let until = parse_utc("1997-08-26T18:00:00+0000");

        let header = get_file_header();
        // Pick a start point for the points in each archive to ensure that the
        // high resolution data (in archive 1, last day) will all be too old to
        // satisfy the request since it's all before the "from" time for the request
        let start = parse_utc("1997-08-20T12:00:00+0000");
        let archive1 = get_archive(&header.archive_info()[0], start);
        let archive2 = get_archive(&header.archive_info()[1], start);

        let req = FetchRequest::default()
            .with_now(now)
            .with_from(from)
            .with_until(until);

        let mut buf = vec![];
        memento_encode_header(&mut buf, &header).unwrap();
        memento_encode_archive(&mut buf, &archive1).unwrap();
        memento_encode_archive(&mut buf, &archive2).unwrap();
        buf.shrink_to_fit();

        let reader = MementoReader::new(&buf);
        let res = reader.read(&req);

        assert!(res.is_ok());
        assert!(res.unwrap().is_empty());
    }

    #[test]
    fn test_read_all_points_after_until() {
        let now = parse_utc("1997-08-27T02:14:00+0000");
        let from = parse_utc("1997-08-26T12:00:00+0000");
        let until = parse_utc("1997-08-26T18:00:00+0000");

        let header = get_file_header();
        // Pick a start point for the points in each archive to ensure that the
        // high resolution data (in archive 1, last day) will all be too recent to
        // satisfy the request since it's all after the "until" time for the request
        let start = parse_utc("1997-08-27T18:05:00+0000");
        let archive1 = get_archive(&header.archive_info()[0], start);
        let archive2 = get_archive(&header.archive_info()[1], start);

        let req = FetchRequest::default()
            .with_now(now)
            .with_from(from)
            .with_until(until);

        let mut buf = vec![];
        memento_encode_header(&mut buf, &header).unwrap();
        memento_encode_archive(&mut buf, &archive1).unwrap();
        memento_encode_archive(&mut buf, &archive2).unwrap();
        buf.shrink_to_fit();

        let reader = MementoReader::new(&buf);
        let res = reader.read(&req);

        assert!(res.is_ok());
        assert!(res.unwrap().is_empty());
    }

    #[test]
    fn test_read_success_high_resolution() {
        let now = parse_utc("1997-08-27T02:14:00+0000");
        let from = parse_utc("1997-08-26T12:00:00+0000");
        let until = parse_utc("1997-08-26T18:01:00+0000");

        let header = get_file_header();
        // Pick a start time here that ensures that two points in the higher resolution
        // archive overlap with the requested range (right on the tail end of the "until"
        // time).
        let start = parse_utc("1997-08-27T18:00:00+0000");
        let archive1 = get_archive(&header.archive_info()[0], start);
        let archive2 = get_archive(&header.archive_info()[1], start);

        let req = FetchRequest::default()
            .with_now(now)
            .with_from(from)
            .with_until(until);

        let mut buf = vec![];
        memento_encode_header(&mut buf, &header).unwrap();
        memento_encode_archive(&mut buf, &archive1).unwrap();
        memento_encode_archive(&mut buf, &archive2).unwrap();
        buf.shrink_to_fit();

        let reader = MementoReader::new(&buf);
        let res = reader.read(&req);

        assert!(res.is_ok());

        let points = res.unwrap();
        assert_eq!(2, points.len());
        assert_eq!(
            &vec![
                Point::new(until.timestamp() as u32 - 60, 7.0),
                Point::new(until.timestamp() as u32, 7.0),
            ],
            &points
        );
    }

    #[test]
    fn test_read_success_low_resolution() {
        let now = parse_utc("1997-08-27T02:14:00+0000");
        let from = parse_utc("1997-08-20T12:00:00+0000");
        let until = parse_utc("1997-08-20T18:05:00+0000");

        let header = get_file_header();
        // Pick a start time here that ensures that two points in the lower resolution
        // archive overlap with the requested range (right on the tail end of the "until"
        // time).
        let start = parse_utc("1997-08-27T18:00:00+0000");
        let archive1 = get_archive(&header.archive_info()[0], start);
        let archive2 = get_archive(&header.archive_info()[1], start);

        let req = FetchRequest::default()
            .with_now(now)
            .with_from(from)
            .with_until(until);

        let mut buf = vec![];
        memento_encode_header(&mut buf, &header).unwrap();
        memento_encode_archive(&mut buf, &archive1).unwrap();
        memento_encode_archive(&mut buf, &archive2).unwrap();
        buf.shrink_to_fit();

        let reader = MementoReader::new(&buf);
        let res = reader.read(&req);

        assert!(res.is_ok());

        let points = res.unwrap();
        assert_eq!(2, points.len());
        assert_eq!(
            &vec![
                Point::new(until.timestamp() as u32 - 300, 7.0),
                Point::new(until.timestamp() as u32, 7.0),
            ],
            &points
        );
    }

    #[test]
    fn test_fetch_request_normalize_nonsense_request() {}

    #[test]
    fn test_fetch_request_normalize_future_start_time() {}

    #[test]
    fn test_fetch_request_normalize_end_exceeds_retention() {}

    #[test]
    fn test_fetch_request_normalize_from_older_than_oldest() {}

    #[test]
    fn test_fetch_request_normalize_from_not_older_than_oldest() {}
}
