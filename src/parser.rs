// Whisper
//
// Copyright 2017 TSH Labs
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Functions to parse Whisper files from a stream of bytes

use nom::{be_u32, be_f32, be_f64, IResult};

use types::{AggregationType, Metadata, ArchiveInfo, Header, Point, Archive, Data, WhisperFile};


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
        let (remaining, archive) = try_parse!(to_parse, apply!(whisper_parse_archive, info));
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


named!(pub whisper_parse_header<&[u8], Header>,
       do_parse!(
           metadata: parse_metadata                         >>
           archives: apply!(parse_archive_infos, &metadata) >>
           (Header::new(metadata, archives))
       )
);


pub fn whisper_parse_archive<'a, 'b>(
    input: &'a [u8],
    info: &'b ArchiveInfo,
) -> IResult<&'a [u8], Archive> {
    let (remaining, points) = try_parse!(input, count!(parse_point, info.num_points() as usize));
    IResult::Done(remaining, Archive::new(points))
}


named!(pub whisper_parse_file<&[u8], WhisperFile>,
       do_parse!(
           header: whisper_parse_header                      >>
           data:   apply!(parse_data, header.archive_info()) >>
           (WhisperFile::new(header, data))
       )
);


#[cfg(test)]
mod tests {
    use types::{AggregationType, Metadata, ArchiveInfo};

    use super::{parse_aggregation_type, parse_archive_info, parse_archive_infos,
                parse_data, parse_metadata, parse_point, whisper_parse_header,
                whisper_parse_archive, whisper_parse_file};

    const SECONDS_PER_YEAR: u32 = 3600 * 24 * 365;

    #[test]
    fn test_parse_aggregation_type() {
        let bytes = &include_bytes!("../tests/upper_01.wsp")[0..4];
        let res = parse_aggregation_type(bytes).unwrap().1;
        assert_eq!(AggregationType::Max, res);
    }

    #[test]
    fn test_parse_archive_info() {
        let bytes = &include_bytes!("../tests/count_01.wsp")[16..28];
        let res = parse_archive_info(bytes).unwrap().1;

        assert_eq!(76, res.offset());
        assert_eq!(10, res.seconds_per_point());
        assert_eq!(8640, res.num_points());
        assert_eq!(86400, res.retention());
    }

    #[test]
    fn test_parse_archive_infos() {
        let bytes = &include_bytes!("../tests/mean_01.wsp")[16..76];
        let metadata = Metadata::new(
            AggregationType::Average,
            31536000,
            0.5,
            5
        );

        let res = parse_archive_infos(bytes, &metadata).unwrap().1;
        assert_eq!(10, res[0].seconds_per_point());
        assert_eq!(60, res[1].seconds_per_point());
        assert_eq!(300, res[2].seconds_per_point());
        assert_eq!(600, res[3].seconds_per_point());
        assert_eq!(3600, res[4].seconds_per_point());
    }

    #[test]
    fn test_parse_data() {
        // Get the data for the first two archives, stopping at the third
        let bytes = &include_bytes!("../tests/upper_01.wsp")[76..224716];
        let info1 = ArchiveInfo::new(
            76,
            10,
            8640
        );
        let info2 = ArchiveInfo::new(
            103756,
            60,
            10080
        );

        let res = parse_data(bytes, &vec![info1, info2]).unwrap().1;
        assert_eq!(8640, res.archives()[0].points().len());
        assert_eq!(10080, res.archives()[1].points().len());
    }

    #[test]
    fn test_parse_metadata() {
        let bytes = &include_bytes!("../tests/count_01.wsp")[0..16];
        let res = parse_metadata(bytes).unwrap().1;

        assert_eq!(AggregationType::Sum, res.aggregation());
        assert_eq!(31536000, res.max_retention());
        assert_eq!(0.5, res.x_files_factor());
        assert_eq!(5, res.archive_count());
    }

    #[test]
    fn test_parse_point() {
        // first point with a value in the db, first point in second archive
        let bytes = &include_bytes!("../tests/mean_01.wsp")[103756..103768];
        let res = parse_point(bytes).unwrap().1;

        assert_eq!(1501988400, res.timestamp());
        assert_eq!(100.0, res.value());
    }

    #[test]
    fn test_parse_archive() {
        // second archive
        let bytes = &include_bytes!("../tests/upper_01.wsp")[103756..224716];
        let info = ArchiveInfo::new(
            103756,
            60,
            10080
        );

        let res = whisper_parse_archive(bytes, &info).unwrap().1;
        assert_eq!(10080, res.points().len());
    }

    #[test]
    fn test_whisper_parse_header() {
        let bytes = include_bytes!("../tests/mean_01.wsp");
        let res = whisper_parse_header(bytes);
        let header = res.unwrap().1;
        let meta = header.metadata();
        let info = header.archive_info();

        assert_eq!(AggregationType::Average, meta.aggregation());
        assert_eq!(SECONDS_PER_YEAR, meta.max_retention());
        assert_eq!(5, meta.archive_count());

        assert_eq!(10, info[0].seconds_per_point());
        assert_eq!(60, info[1].seconds_per_point());
        assert_eq!(300, info[2].seconds_per_point());
        assert_eq!(600, info[3].seconds_per_point());
        assert_eq!(3600, info[4].seconds_per_point());

        assert_eq!(8640, info[0].num_points());
        assert_eq!(10_080, info[1].num_points());
        assert_eq!(8640, info[2].num_points());
        assert_eq!(25_920, info[3].num_points());
        assert_eq!(8760, info[4].num_points());
    }

    #[test]
    fn test_whisper_parse_file() {
        let bytes = include_bytes!("../tests/mean_01.wsp");
        let res = whisper_parse_file(bytes);
        let file = res.unwrap().1;
        let header = file.header();
        let data = file.data();

        let meta = header.metadata();
        let info = header.archive_info();
        let archives = data.archives();

        assert_eq!(AggregationType::Average, meta.aggregation());
        assert_eq!(SECONDS_PER_YEAR, meta.max_retention());
        assert_eq!(5, meta.archive_count());

        assert_eq!(8640, info[0].num_points());
        assert_eq!(8640, archives[0].points().len());

        assert_eq!(10_080, info[1].num_points());
        assert_eq!(10_080, archives[1].points().len());

        assert_eq!(8640, info[2].num_points());
        assert_eq!(8640, archives[2].points().len());

        assert_eq!(25_920, info[3].num_points());
        assert_eq!(25_920, archives[3].points().len());

        assert_eq!(8760, info[4].num_points());
        assert_eq!(8760, archives[4].points().len());
    }
}
