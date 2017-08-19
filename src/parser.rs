//


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


named!(parse_point<&[u8], Point>,
       do_parse!(
           timestamp: be_u32 >>
           value:     be_f64 >>
           (Point::new(timestamp, value))
       )
);


fn parse_archive<'a, 'b>(input: &'a [u8], info: &'b ArchiveInfo) -> IResult<&'a [u8], Archive> {
    let (remaining, points) = try_parse!(input, count!(parse_point, info.num_points() as usize));
    IResult::Done(remaining, Archive::new(points))
}


fn parse_data<'a, 'b>(input: &'a [u8], infos: &'b [ArchiveInfo]) -> IResult<&'a [u8], Data> {
    let mut archives = Vec::with_capacity(infos.len());
    let mut to_parse = input;

    for info in infos {
        let (remaining, archive) = try_parse!(to_parse, apply!(parse_archive, info));
        to_parse = remaining;

        archives.push(archive);
    }

    IResult::Done(to_parse, Data::new(archives))
}

named!(pub whisper_parse_metadata<&[u8], Metadata>,
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



pub fn whisper_parse_archive_infos<'a, 'b>(
    input: &'a [u8],
    metadata: &'b Metadata,
) -> IResult<&'a [u8], Vec<ArchiveInfo>> {
    let (remaining, infos) = try_parse!(
        input,
        count!(parse_archive_info, metadata.archive_count() as usize)
    );
    IResult::Done(remaining, infos)
}


named!(pub whisper_parse_header<&[u8], Header>,
       do_parse!(
           metadata: whisper_parse_metadata                         >>
           archives: apply!(whisper_parse_archive_infos, &metadata) >>
           (Header::new(metadata, archives))
       )
);


named!(pub whisper_parse_file<&[u8], WhisperFile>,
       do_parse!(
           header: whisper_parse_header                      >>
           data:   apply!(parse_data, header.archive_info()) >>
           (WhisperFile::new(header, data))
       )
);


#[cfg(test)]
mod tests {
    #[test]
    fn test_parse_aggregation_type() {}

    #[test]
    fn test_parse_metadata() {}

    #[test]
    fn test_parse_archive_info() {}

    #[test]
    fn test_parse_archive_infos() {}

    #[test]
    fn test_parse_point() {}

    #[test]
    fn test_parse_archive() {}

    #[test]
    fn test_parse_data() {}

    #[test]
    fn test_whisper_parse_header() {}

    #[test]
    fn test_whisper_parse_file() {}
}
