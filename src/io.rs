// Whisper
//
// Copyright 2017 TSH Labs
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Functions to read and write parts of the Whisper format to disk

use std::fs::File;
use std::io;

use byteorder::{WriteBytesExt, NetworkEndian};
use fs2::FileExt;
use memmap::{Mmap, Protection};

use parser::{whisper_parse_header, whisper_parse_file};
use types::{WhisperFile, Header, Metadata, ArchiveInfo, Archive, Point, Data};
use core::WhisperResult;


fn write_metadata<W>(writer: &mut W, meta: &Metadata) -> io::Result<()>
where
    W: WriteBytesExt,
{
    writer.write_u32::<NetworkEndian>(meta.aggregation() as u32)?;
    writer.write_u32::<NetworkEndian>(meta.max_retention())?;
    writer.write_f32::<NetworkEndian>(meta.x_files_factor())?;
    writer.write_u32::<NetworkEndian>(meta.archive_count())
}


fn write_archive_info<W>(writer: &mut W, infos: &[ArchiveInfo]) -> io::Result<()>
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


fn write_point<W>(writer: &mut W, point: &Point) -> io::Result<()>
where
    W: WriteBytesExt,
{
    writer.write_u32::<NetworkEndian>(point.timestamp())?;
    writer.write_f64::<NetworkEndian>(point.value())
}


fn write_archive<W>(writer: &mut W, archive: &Archive) -> io::Result<()>
where
    W: WriteBytesExt,
{
    for point in archive.points() {
        write_point(writer, point)?;
    }

    Ok(())
}


fn write_data<W>(writer: &mut W, data: &Data) -> io::Result<()>
where
    W: WriteBytesExt,
{
    for archive in data.archives() {
        write_archive(writer, archive)?;
    }

    Ok(())
}


pub fn whisper_write_header<W>(writer: &mut W, header: &Header) -> WhisperResult<()>
where
    W: WriteBytesExt,
{
    write_metadata(writer, header.metadata())?;
    Ok(write_archive_info(writer, header.archive_info())?)
}


pub fn whisper_write_file<W>(writer: &mut W, file: &WhisperFile) -> WhisperResult<()>
where
    W: WriteBytesExt,
{
    whisper_write_header(writer, file.header())?;
    Ok(write_data(writer, file.data())?)
}


// TODO: Explain impl: mmap vs regular I/O (how locks factor in)
pub fn whisper_read_header(path: &str) -> WhisperResult<Header> {
    let file = File::open(path)?;
    file.lock_shared()?;

    let mmap = Mmap::open(&file, Protection::Read)?;
    // Unsafe is OK here since we've obtained a shared (read) lock
    let bytes = unsafe { mmap.as_slice() };
    let res = whisper_parse_header(bytes).to_full_result()?;

    file.unlock()?;
    Ok(res)
}


// TODO: Explain impl: small bufs, vs big buf, vs mmap
pub fn whisper_read_file(path: &str) -> WhisperResult<WhisperFile> {
    // Provide some entry-like API?
    // let res = wrapper.with_slice(|bytes| {
    //     do a bunch of stuff
    // })
    let file = File::open(path)?;
    file.lock_shared()?;

    // Potential extension: madvise(sequential). Didn't seem to make difference
    // in benchmarks but maybe real world use is different (or maybe non-ssd use
    // is different).
    let mmap = Mmap::open(&file, Protection::Read)?;
    // Unsafe is OK here since we've obtained a shared (read) lock
    let bytes = unsafe { mmap.as_slice() };
    let res = whisper_parse_file(bytes).to_full_result()?;

    file.unlock()?;
    Ok(res)
}


#[cfg(test)]
mod tests {}
