//


use nom::{be_u32, be_f32, be_f64};

use file::{AggregationType, Metadata, ArchiveInfo, Header, Point, Archive};


// Basic data types used by the Whisper database format, all big-endian.
named!(parse_u32<&[u8], u32>, flat_map!(take!(4), be_u32));
named!(parse_f32<&[u8], f32>, flat_map!(take!(4), be_f32));
named!(parse_f64<&[u8], f64>, flat_map!(take!(8), be_f64));


named!(parse_aggregation_type<&[u8], AggregationType>,
       do_parse!(
           val: parse_u32 >>
           agg: expr_opt!(match val {
               1 => Some(AggregationType::Average),
               2 => Some(AggregationType::Sum),
               3 => Some(AggregationType::Last),
               4 => Some(AggregationType::Max),
               5 => Some(AggregationType::Min),
               6 => Some(AggregationType::AvgZero),
               7 => Some(AggregationType::AbsMax),
               8 => Some(AggregationType::AbsMin),
               _ => None,
           }) >>
           (agg)
       )
);


named!(parse_metadata<&[u8], Metadata>,
       do_parse!(
           aggregation: call!(parse_aggregation_type) >>
           max_retention: call!(parse_u32) >>
           x_files_factor: call!(parse_f32) >>
           archive_count: call!(parse_u32) >>
           (Metadata::new(
               aggregation,
               max_retention,
               x_files_factor,
               archive_count
           ))
       )
);


named!(parse_archive_info<&[u8], ArchiveInfo>,
       do_parse!(
           offset: call!(parse_u32) >>
           secs_per_point: call!(parse_u32) >>
           num_points: call!(parse_u32) >>
           (ArchiveInfo::new(
               offset,
               secs_per_point,
               num_points
           ))
       )
);


named!(parse_header<&[u8], Header>,
       do_parse!(
           metadata: call!(parse_metadata) >>
           archives: count!(parse_archive_info, metadata.archive_count() as usize) >>
           (Header::new(metadata, archives))
       )
);


named!(parse_point<&[u8], Point>,
       do_parse!(
           timestamp: call!(parse_u32) >>
           value: call!(parse_f64) >>
           (Point::new(timestamp, value))
       )
);



#[cfg(test)]
mod tests {
    use std::mem;
    use nom::IResult;
    use super::{parse_u32, parse_f32, parse_f64};

    // TODO: probably going to need to use byteorder here

    #[test]
    fn test_parse_u32() {
        let expected = 2342u32;
        let as_bytes: [u8; 4] = unsafe { mem::transmute(expected.to_be()) };
        assert_eq!(IResult::Done(&b""[..], expected), parse_u32(&as_bytes));
    }

    #[test]
    fn test_parse_f32() {
    }

    #[test]
    fn test_parse_f64() {
    }
}
