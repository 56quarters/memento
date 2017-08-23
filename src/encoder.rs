// Whisper
//
// Copyright 2017 TSH Labs
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Functions to encode Whisper files to a stream of bytes

use std::io;

use byteorder::{WriteBytesExt, NetworkEndian};

use types::{WhisperFile, Header, Metadata, ArchiveInfo, Archive, Point, Data};
use core::WhisperResult;


fn encode_metadata<W>(writer: &mut W, meta: &Metadata) -> io::Result<()>
where
    W: WriteBytesExt,
{
    writer.write_u32::<NetworkEndian>(meta.aggregation() as u32)?;
    writer.write_u32::<NetworkEndian>(meta.max_retention())?;
    writer.write_f32::<NetworkEndian>(meta.x_files_factor())?;
    writer.write_u32::<NetworkEndian>(meta.archive_count())
}


fn encode_archive_info<W>(writer: &mut W, infos: &[ArchiveInfo]) -> io::Result<()>
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


fn encode_point<W>(writer: &mut W, point: &Point) -> io::Result<()>
where
    W: WriteBytesExt,
{
    writer.write_u32::<NetworkEndian>(point.timestamp())?;
    writer.write_f64::<NetworkEndian>(point.value())
}


fn encode_archive<W>(writer: &mut W, archive: &Archive) -> io::Result<()>
where
    W: WriteBytesExt,
{
    for point in archive.points() {
        encode_point(writer, point)?;
    }

    Ok(())
}


fn encode_data<W>(writer: &mut W, data: &Data) -> io::Result<()>
where
    W: WriteBytesExt,
{
    for archive in data.archives() {
        encode_archive(writer, archive)?;
    }

    Ok(())
}


pub fn whisper_encode_header<W>(writer: &mut W, header: &Header) -> WhisperResult<()>
where
    W: WriteBytesExt,
{
    encode_metadata(writer, header.metadata())?;
    Ok(encode_archive_info(writer, header.archive_info())?)
}


pub fn whisper_encode_file<W>(writer: &mut W, file: &WhisperFile) -> WhisperResult<()>
where
    W: WriteBytesExt,
{
    whisper_encode_header(writer, file.header())?;
    Ok(encode_data(writer, file.data())?)
}
