//


use nom::{be_u32, be_f32, be_f64};

use file::{AggregationType, Metadata};

trace_macros!(true);

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

named!(parse_max_retention<&[u8], u32>, call!(parse_u32));
named!(parse_x_files_factor<&[u8], f32>, call!(parse_f32));
named!(parse_archive_count<&[u8], u32>, call!(parse_u32));

/*
named!(parse_metadata<&[u8], Metadata>,
       do_parse!(
           agg: call!(parse_aggregation_type) >>
           ret: call!(parse_max_retention) >>
           xff: call!(parse_x_files_factor) >>
           ac: call!(parse_archive_count) >>
           md: || {
               Metadata {
                   aggregation: agg,
                   max_retention: ret,
                   x_files_factor: xff,
                   archive_count: ac,
               }
           } >>
           (md)
       )
);
*/
