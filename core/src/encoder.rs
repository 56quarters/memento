// Memento - A Whisper implementation in Rust
//
// Copyright 2017-2018 TSH Labs
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Functions to encode Whisper files to a stream of bytes

use std::io;

use byteorder::{NetworkEndian, WriteBytesExt};

use types::{Archive, ArchiveInfo, Data, Header, MementoDatabase, Metadata, Point};
use errors::MementoResult;

pub fn memento_encode_metadata<W>(writer: &mut W, meta: &Metadata) -> io::Result<()>
where
    W: WriteBytesExt,
{
    writer.write_u32::<NetworkEndian>(meta.aggregation() as u32)?;
    writer.write_u32::<NetworkEndian>(meta.max_retention())?;
    writer.write_f32::<NetworkEndian>(meta.x_files_factor())?;
    writer.write_u32::<NetworkEndian>(meta.archive_count())
}

pub fn memento_encode_archive_infos<W>(writer: &mut W, infos: &[ArchiveInfo]) -> io::Result<()>
where
    W: WriteBytesExt,
{
    for info in infos {
        writer.write_u32::<NetworkEndian>(info.offset())?;
        writer.write_u32::<NetworkEndian>(info.seconds_per_point())?;
        writer.write_u32::<NetworkEndian>(info.num_points())?;
    }

    Ok(())
}

pub fn memento_encode_point<W>(writer: &mut W, point: &Point) -> io::Result<()>
where
    W: WriteBytesExt,
{
    writer.write_u32::<NetworkEndian>(point.timestamp())?;
    writer.write_f64::<NetworkEndian>(point.value())
}

pub fn memento_encode_data<W>(writer: &mut W, data: &Data) -> io::Result<()>
where
    W: WriteBytesExt,
{
    for archive in data.archives() {
        memento_encode_archive(writer, archive)?;
    }

    Ok(())
}

pub fn memento_encode_archive<W>(writer: &mut W, archive: &Archive) -> io::Result<()>
where
    W: WriteBytesExt,
{
    for point in archive.points() {
        memento_encode_point(writer, point)?;
    }

    Ok(())
}

pub fn memento_encode_header<W>(writer: &mut W, header: &Header) -> MementoResult<()>
where
    W: WriteBytesExt,
{
    memento_encode_metadata(writer, header.metadata())?;
    Ok(memento_encode_archive_infos(writer, header.archive_info())?)
}

pub fn memento_encode_database<W>(writer: &mut W, file: &MementoDatabase) -> MementoResult<()>
where
    W: WriteBytesExt,
{
    memento_encode_header(writer, file.header())?;
    Ok(memento_encode_data(writer, file.data())?)
}

#[cfg(test)]
mod tests {
    use types::{AggregationType, Archive, ArchiveInfo, Data, Header, MementoDatabase, Metadata,
                Point};

    use super::{memento_encode_archive_infos, memento_encode_data, memento_encode_metadata,
                memento_encode_point, memento_encode_archive, memento_encode_database,
                memento_encode_header};

    #[test]
    fn test_memento_encode_metadata() {
        let metadata = Metadata::new(AggregationType::Max, 31536000, 0.5, 5);

        // Python: struct.pack('>LLfL', 4, 31536000, 0.5, 5).hex()
        #[cfg_attr(rustfmt, rustfmt_skip)]
        let expected = vec![
            0x00, 0x00, 0x00, 0x04,
            0x01, 0xe1, 0x33, 0x80,
            0x3f, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x05
        ];

        let mut buf = vec![];
        memento_encode_metadata(&mut buf, &metadata).unwrap();

        assert_eq!(&expected, &buf);
    }

    #[test]
    fn test_memento_encode_archive_infos() {
        let info = ArchiveInfo::new(76, 10, 8640);

        // Python: struct.pack('>LLL', 76, 10, 8640).hex()
        #[cfg_attr(rustfmt, rustfmt_skip)]
        let expected = vec![
            0x00, 0x00, 0x00, 0x4c,
            0x00, 0x00, 0x00, 0x0a,
            0x00, 0x00, 0x21, 0xc0
        ];

        let mut buf = vec![];
        memento_encode_archive_infos(&mut buf, &vec![info]).unwrap();

        assert_eq!(&expected, &buf);
    }

    #[test]
    fn test_memento_encode_point() {
        let point = Point::new(1511396041, 42.0);

        // Python: struct.pack('>Ld', 1511396041, 42.0).hex()
        #[cfg_attr(rustfmt, rustfmt_skip)]
        let expected = vec![
            0x5a, 0x16, 0x12, 0xc9,
            0x40, 0x45, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
        ];

        let mut buf = vec![];
        memento_encode_point(&mut buf, &point).unwrap();

        assert_eq!(&expected, &buf);
    }

    #[test]
    fn test_memento_encode_data() {
        let point1 = Point::new(1511396041, 42.0);
        let point2 = Point::new(1511396051, 42.0);
        let archive = Archive::new(vec![point1, point2]);
        let data = Data::new(vec![archive]);

        // Python:
        // struct.pack('>Ld', 1511396041, 42.0).hex()
        // struct.pack('>Ld', 1511396051, 42.0).hex()
        #[cfg_attr(rustfmt, rustfmt_skip)]
        let expected = vec![
            0x5a, 0x16, 0x12, 0xc9,
            0x40, 0x45, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,

            0x5a, 0x16, 0x12, 0xd3,
            0x40, 0x45, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
        ];

        let mut buf = vec![];
        memento_encode_data(&mut buf, &data).unwrap();

        assert_eq!(&expected, &buf);
    }

    #[test]
    fn test_memento_encode_archive() {
        let point1 = Point::new(1511396041, 42.0);
        let point2 = Point::new(1511396051, 42.0);
        let archive = Archive::new(vec![point1, point2]);

        // Python:
        // struct.pack('>Ld', 1511396041, 42.0).hex()
        // struct.pack('>Ld', 1511396051, 42.0).hex()
        #[cfg_attr(rustfmt, rustfmt_skip)]
        let expected = vec![
            0x5a, 0x16, 0x12, 0xc9,
            0x40, 0x45, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,

            0x5a, 0x16, 0x12, 0xd3,
            0x40, 0x45, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
        ];

        let mut buf = vec![];
        memento_encode_archive(&mut buf, &archive).unwrap();

        assert_eq!(&expected, &buf);
    }

    #[test]
    fn test_memento_encode_header() {
        let metadata = Metadata::new(AggregationType::Min, 86400, 0.5, 1);
        let info = ArchiveInfo::new(28, 10, 8640);
        let header = Header::new(metadata, vec![info]);

        // Python:
        // struct.pack('>LLfL', 5, 86400, 0.5, 1).hex()
        // struct.pack('>LLL', 28, 10, 8640).hex()
        #[cfg_attr(rustfmt, rustfmt_skip)]
        let expected = vec![
            0x00, 0x00, 0x00, 0x05,
            0x00, 0x01, 0x51, 0x80,
            0x3f, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x01,

            0x00, 0x00, 0x00, 0x1c,
            0x00, 0x00, 0x00, 0x0a,
            0x00, 0x00, 0x21, 0xc0
        ];

        let mut buf = vec![];
        memento_encode_header(&mut buf, &header).unwrap();

        assert_eq!(&expected, &buf);
    }

    #[test]
    fn test_memento_encode_database() {
        let metadata = Metadata::new(AggregationType::Min, 86400, 0.5, 1);
        let info = ArchiveInfo::new(28, 10, 8640);
        let header = Header::new(metadata, vec![info]);
        let point1 = Point::new(1511396041, 42.0);
        let point2 = Point::new(1511396051, 42.0);
        let archive = Archive::new(vec![point1, point2]);
        let data = Data::new(vec![archive]);
        let file = MementoDatabase::new(header, data);

        // Python:
        // struct.pack('>LLfL', 5, 86400, 0.5, 1).hex()
        // struct.pack('>LLL', 28, 10, 8640).hex()
        // struct.pack('>Ld', 1511396041, 42.0).hex()
        // struct.pack('>Ld', 1511396051, 42.0).hex()
        #[cfg_attr(rustfmt, rustfmt_skip)]
        let expected = vec![
            0x00, 0x00, 0x00, 0x05,
            0x00, 0x01, 0x51, 0x80,
            0x3f, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x01,

            0x00, 0x00, 0x00, 0x1c,
            0x00, 0x00, 0x00, 0x0a,
            0x00, 0x00, 0x21, 0xc0,

            0x5a, 0x16, 0x12, 0xc9,
            0x40, 0x45, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,

            0x5a, 0x16, 0x12, 0xd3,
            0x40, 0x45, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
        ];

        let mut buf = vec![];
        memento_encode_database(&mut buf, &file).unwrap();

        assert_eq!(&expected, &buf);
    }
}
