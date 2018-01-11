// Memento - A Whisper implementation in Rust
//
// Copyright 2017-2018 TSH Labs
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Functions to parse Whisper files from a stream of bytes

use nom::{IResult, be_f32, be_f64, be_u32};

use types::{AggregationType, Archive, ArchiveInfo, Data, Header, MementoDatabase, Metadata, Point};

named!(parse_aggregation_type<&[u8], AggregationType>,
       switch!(be_u32,
               1 => value!(AggregationType::Average) |
               2 => value!(AggregationType::Sum)     |
               3 => value!(AggregationType::Last)    |
               4 => value!(AggregationType::Max)     |
               5 => value!(AggregationType::Min)     |
               6 => value!(AggregationType::AvgZero) |
               7 => value!(AggregationType::AbsMax)  |
               8 => value!(AggregationType::AbsMin)
       )
);

named!(parse_archive_info<&[u8], ArchiveInfo>,
       do_parse!(
           offset:         be_u32 >>
           secs_per_point: be_u32 >>
           num_points:     be_u32 >>
           (ArchiveInfo::new(
               offset,
               secs_per_point,
               num_points
           ))
       )
);

fn parse_archive_infos<'a, 'b>(
    input: &'a [u8],
    metadata: &'b Metadata,
) -> IResult<&'a [u8], Vec<ArchiveInfo>> {
    let (remaining, infos) = try_parse!(
        input,
        count!(parse_archive_info, metadata.archive_count() as usize)
    );
    IResult::Done(remaining, infos)
}

fn parse_data<'a, 'b>(input: &'a [u8], infos: &'b [ArchiveInfo]) -> IResult<&'a [u8], Data> {
    let mut archives = Vec::with_capacity(infos.len());
    let mut to_parse = input;

    for info in infos {
        let (remaining, archive) = try_parse!(to_parse, apply!(memento_parse_archive, info));
        to_parse = remaining;
        archives.push(archive);
    }

    IResult::Done(to_parse, Data::new(archives))
}

named!(parse_metadata<&[u8], Metadata>,
       do_parse!(
           aggregation:    parse_aggregation_type >>
           max_retention:  be_u32                 >>
           x_files_factor: be_f32                 >>
           archive_count:  be_u32                 >>
           (Metadata::new(
               aggregation,
               max_retention,
               x_files_factor,
               archive_count
           ))
       )
);

named!(parse_point<&[u8], Point>,
       do_parse!(
           timestamp: be_u32 >>
           value:     be_f64 >>
           (Point::new(timestamp, value))
       )
);

named!(pub memento_parse_header<&[u8], Header>,
       do_parse!(
           metadata: parse_metadata                         >>
           archives: apply!(parse_archive_infos, &metadata) >>
           (Header::new(metadata, archives))
       )
);

pub fn memento_parse_archive<'a, 'b>(
    input: &'a [u8],
    info: &'b ArchiveInfo,
) -> IResult<&'a [u8], Archive> {
    let (remaining, points) = try_parse!(input, count!(parse_point, info.num_points() as usize));

    IResult::Done(remaining, Archive::new(points))
}

named!(pub memento_parse_database<&[u8], MementoDatabase>,
       do_parse!(
           header: memento_parse_header                      >>
           data:   apply!(parse_data, header.archive_info()) >>
           (MementoDatabase::new(header, data))
       )
);

#[cfg(test)]
mod tests {
    use types::{AggregationType, Archive, ArchiveInfo, Data, Header, MementoDatabase, Metadata,
                Point};

    use super::{memento_parse_archive, memento_parse_database, memento_parse_header,
                parse_aggregation_type, parse_archive_info, parse_archive_infos, parse_data,
                parse_metadata, parse_point};

    #[test]
    fn test_parse_aggregation_type() {
        // Python: struct.pack('>L', 4).hex()
        let bytes = vec![0x00, 0x00, 0x00, 0x04];
        let res = parse_aggregation_type(&bytes).unwrap().1;
        assert_eq!(AggregationType::Max, res);
    }

    #[test]
    fn test_parse_archive_info() {
        let expected = ArchiveInfo::new(76, 10, 8640);

        // Python: struct.pack('>LLL', 76, 10, 8640).hex()
        #[cfg_attr(rustfmt, rustfmt_skip)]
        let bytes = vec![
            0x00, 0x00, 0x00, 0x4c,
            0x00, 0x00, 0x00, 0x0a,
            0x00, 0x00, 0x21, 0xc0
        ];

        let res = parse_archive_info(&bytes).unwrap().1;
        assert_eq!(expected, res);
    }

    #[test]
    fn test_parse_archive_infos() {
        let expected = ArchiveInfo::new(28, 10, 8640);
        let metadata = Metadata::new(AggregationType::Average, 86400, 0.5, 1);

        // Python: struct.pack('>LLL', 28, 10, 8640).hex()
        #[cfg_attr(rustfmt, rustfmt_skip)]
        let bytes = vec![
            0x00, 0x00, 0x00, 0x1c,
            0x00, 0x00, 0x00, 0x0a,
            0x00, 0x00, 0x21, 0xc0

        ];

        let res = parse_archive_infos(&bytes, &metadata).unwrap().1;
        assert_eq!(vec![expected], res);
    }

    #[test]
    fn test_parse_data() {
        let point1 = Point::new(1511396041, 42.0);
        let point2 = Point::new(1511396051, 42.0);
        let archive = Archive::new(vec![point1, point2]);
        let expected = Data::new(vec![archive]);

        let info = ArchiveInfo::new(28, 10, 2);

        // Python:
        // struct.pack('>Ld', 1511396041, 42.0).hex()
        // struct.pack('>Ld', 1511396051, 42.0).hex()
        #[cfg_attr(rustfmt, rustfmt_skip)]
        let bytes = vec![
            0x5a, 0x16, 0x12, 0xc9,
            0x40, 0x45, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,

            0x5a, 0x16, 0x12, 0xd3,
            0x40, 0x45, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
        ];

        let res = parse_data(&bytes, &vec![info]).unwrap().1;
        assert_eq!(expected, res);
    }

    #[test]
    fn test_parse_metadata() {
        let expected = Metadata::new(AggregationType::Sum, 86400, 0.5, 1);

        // Python: struct.pack('>LLfL', 2, 86400, 0.5, 1).hex()
        #[cfg_attr(rustfmt, rustfmt_skip)]
        let bytes = vec![
            0x00, 0x00, 0x00, 0x02,
            0x00, 0x01, 0x51, 0x80,
            0x3f, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x01
        ];

        let res = parse_metadata(&bytes).unwrap().1;
        assert_eq!(expected, res);
    }

    #[test]
    fn test_parse_point() {
        let expected = Point::new(1511396041, 42.0);

        // Python: struct.pack('>Ld', 1511396041, 42.0).hex()
        #[cfg_attr(rustfmt, rustfmt_skip)]
        let bytes = vec![
            0x5a, 0x16, 0x12, 0xc9,
            0x40, 0x45, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];

        let res = parse_point(&bytes).unwrap().1;
        assert_eq!(expected, res);
    }

    #[test]
    fn test_memento_parse_archive() {
        let point1 = Point::new(1511396041, 42.0);
        let point2 = Point::new(1511396051, 42.0);
        let expected = Archive::new(vec![point1, point2]);

        let info = ArchiveInfo::new(28, 10, 2);

        // Python:
        // struct.pack('>Ld', 1511396041, 42.0).hex()
        // struct.pack('>Ld', 1511396051, 42.0).hex()
        #[cfg_attr(rustfmt, rustfmt_skip)]
        let bytes = vec![
            0x5a, 0x16, 0x12, 0xc9,
            0x40, 0x45, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,

            0x5a, 0x16, 0x12, 0xd3,
            0x40, 0x45, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
        ];

        let res = memento_parse_archive(&bytes, &info).unwrap().1;
        assert_eq!(expected, res);
    }

    #[test]
    fn test_memento_parse_header() {
        let metadata = Metadata::new(AggregationType::Min, 86400, 0.5, 1);
        let info = ArchiveInfo::new(28, 10, 8640);
        let expected = Header::new(metadata, vec![info]);

        // Python:
        // struct.pack('>LLfL', 5, 86400, 0.5, 1).hex()
        // struct.pack('>LLL', 28, 10, 8640).hex()
        #[cfg_attr(rustfmt, rustfmt_skip)]
        let bytes = vec![
            0x00, 0x00, 0x00, 0x05,
            0x00, 0x01, 0x51, 0x80,
            0x3f, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x01,

            0x00, 0x00, 0x00, 0x1c,
            0x00, 0x00, 0x00, 0x0a,
            0x00, 0x00, 0x21, 0xc0
        ];

        let res = memento_parse_header(&bytes).unwrap().1;
        assert_eq!(expected, res);
    }

    #[test]
    fn test_memento_parse_database() {
        let metadata = Metadata::new(AggregationType::Min, 86400, 0.5, 1);
        let info = ArchiveInfo::new(28, 10, 2);
        let header = Header::new(metadata, vec![info]);
        let point1 = Point::new(1511396041, 42.0);
        let point2 = Point::new(1511396051, 42.0);
        let archive = Archive::new(vec![point1, point2]);
        let data = Data::new(vec![archive]);
        let expected = MementoDatabase::new(header, data);

        // Python:
        // struct.pack('>LLfL', 5, 86400, 0.5, 1).hex()
        // struct.pack('>LLL', 28, 10, 2).hex()
        // struct.pack('>Ld', 1511396041, 42.0).hex()
        // struct.pack('>Ld', 1511396051, 42.0).hex()
        #[cfg_attr(rustfmt, rustfmt_skip)]
        let bytes = vec![
            0x00, 0x00, 0x00, 0x05,
            0x00, 0x01, 0x51, 0x80,
            0x3f, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x01,

            0x00, 0x00, 0x00, 0x1c,
            0x00, 0x00, 0x00, 0x0a,
            0x00, 0x00, 0x00, 0x02,

            0x5a, 0x16, 0x12, 0xc9,
            0x40, 0x45, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,

            0x5a, 0x16, 0x12, 0xd3,
            0x40, 0x45, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];

        let res = memento_parse_database(&bytes);
        assert_eq!(expected, res.unwrap().1);
    }
}
