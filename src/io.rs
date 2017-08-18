// write to files?

use std::io;
use byteorder::{WriteBytesExt, NetworkEndian};
use types::{WhisperFile, Header, Metadata, ArchiveInfo, Archive, Point, Data};


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


pub fn whisper_write_header<W>(writer: &mut W, header: &Header) -> io::Result<()>
where
    W: WriteBytesExt,
{
    write_metadata(writer, header.metadata())?;
    write_archive_info(writer, header.archive_info())
}


pub fn whisper_write_file<W>(writer: &mut W, file: &WhisperFile) -> io::Result<()>
where
    W: WriteBytesExt,
{
    whisper_write_header(writer, file.header())?;
    write_data(writer, file.data())
}


#[cfg(test)]
mod tests {}
