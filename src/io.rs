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

use std::io::{self, Read};
use std::usize;

use byteorder::{WriteBytesExt, NetworkEndian};

use parser::{whisper_parse_metadata, whisper_parse_archive_infos, whisper_parse_data,
             whisper_parse_file, whisper_parse_archive};
use types::{WhisperFile, Header, Metadata, ArchiveInfo, Archive, Point, Data};
use core::WhisperResult;


const DEFAULT_HEADER_READ_BUF: usize = 128;


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


pub fn whisper_read_header<R>(reader: &mut R) -> WhisperResult<Header>
where
    R: Read,
{
    let mut buf = Vec::with_capacity(DEFAULT_HEADER_READ_BUF);
    let mut handle = reader.take(Metadata::storage() as u64);
    let meta_read = handle.read_to_end(&mut buf)?;

    let reader = handle.into_inner();
    let metadata = whisper_parse_metadata(&buf[0..meta_read]).to_full_result()?;

    let mut handle = reader.take(
        ArchiveInfo::storage() as u64 * metadata.archive_count() as u64,
    );

    let _r = handle.read_to_end(&mut buf)?;
    let archive_infos = whisper_parse_archive_infos(&buf[meta_read..], &metadata)
        .to_full_result()?;

    Ok(Header::new(metadata, archive_infos))
}


pub fn whisper_read_file_big_buf<R>(reader: &mut R) -> WhisperResult<WhisperFile>
where
    R: Read,
{
    let header = whisper_read_header(reader)?;

    let left = header.file_size() - header.size();
    let mut buf: Vec<u8> = Vec::with_capacity(left);
    let mut handle = reader.take(left as u64);
    let _r = handle.read_to_end(&mut buf)?;

    let data = whisper_parse_data(&buf, header.archive_info()).to_full_result()?;

    Ok(WhisperFile::new(header, data))
}

pub fn whisper_read_file_small_buf<R>(reader: &mut R) -> WhisperResult<WhisperFile>
where
    R: Read,
{
    let header = whisper_read_header(reader)?;

    let buf_sz = header.archive_info().iter()
        .map(|a| a.num_points() as usize * Point::storage())
        .max()
        .unwrap_or(0);

    let mut buf = Vec::with_capacity(buf_sz);
    let archives = header.archive_info().iter()
        .flat_map(|info| {
            let mut handle = reader.take(info.archive_size() as u64);
            let _r = handle.read_to_end(&mut buf);
            whisper_parse_archive(&buf, &info).to_full_result()
        }).collect::<Vec<Archive>>();

    Ok(WhisperFile::new(header, Data::new(archives)))
}


use memmap::{Mmap, Protection};
use std::fs::File;

// TODO: Explain impl: small bufs, vs big buf, vs mmap
pub fn whisper_read_file_mmap(map: &Mmap) -> WhisperResult<WhisperFile> {
    // start with path
    // into file
    // lock file (shared)
    // mmap
    // read / parse
    // unlock
    let bytes = unsafe { map.as_slice() };
    Ok(whisper_parse_file(bytes).to_full_result()?)
}

// TODO: Explain impl: small bufs, vs big buf, vs mmap
pub fn whisper_read_file_mmap2(path: &str) -> WhisperResult<WhisperFile> {
    // start with path
    // into file
    // lock file (shared)
    // mmap
    // read / parse
    // unlock
    let map = Mmap::open_path(path, Protection::Read).unwrap();
    let bytes = unsafe { map.as_slice() };
    Ok(whisper_parse_file(bytes).to_full_result()?)
}


#[cfg(test)]
mod tests {}
