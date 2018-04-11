// Memento - A Whisper implementation in Rust
//
// Copyright 2017-2018 TSH Labs
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Functions to read parts of the Whisper format from disk

use std::io::{self, Cursor, Read, Seek, SeekFrom};
use std::fmt::{self, Debug};
use std::fs::File;

use memento_core::errors::{MementoError, MementoResult};

// Default buffer size used by `SliceReaderDirect`. Big enough to
// be used for an entire typical Whisper database header (metadata
// + eight or fewer archive info sections).
const DEFAULT_DIRECT_IO_BUFFER: usize = 128;

/// Trait to combine `Seek` and `Read` so that we can use a `Box` to hold
/// a single object that implements both. This wouldn't be possible otherwise
/// since they are not 'auto traits'.
pub trait SeekRead: Seek + Read {}

impl SeekRead for File {}

impl<'a> SeekRead for &'a File {}

impl<T> SeekRead for Cursor<T> where T: AsRef<[u8]> {}

/// Trait for consuming bytes from some type of underlying file as a range of
/// bytes.
///
/// The trait is meant to abstract the differences between direct file I/O and
/// memory mapped files, allowing callers to choose the implementation with the
/// best performance for their use case.
///
/// Offsets are always absolute and computed relative to the start of the file.
pub trait SliceReader {
    /// Consume the entire underlying file, exposed as a slice of bytes.
    ///
    /// Implemenations may choose to buffer the entire file in memory before
    /// exposing it to the consumer. It is up to callers to only invoke this
    /// method when they are interested in the whole file, not just a portion
    /// of it.
    fn consume_all<F, T>(&mut self, consumer: F) -> MementoResult<T>
    where
        F: Fn(&[u8]) -> MementoResult<T>;

    /// Consume the entire underlying file, starting from `offset`, exposed as
    /// a slice of bytes.
    fn consume_from<F, T>(&mut self, offset: u64, consumer: F) -> MementoResult<T>
    where
        F: Fn(&[u8]) -> MementoResult<T>;

    /// Consume a portion of the underlying file, starting from `offset` and
    /// continuing for `len` bytes, exposed as a slice of bytes.
    fn consume<F, T>(&mut self, offset: u64, len: u64, consumer: F) -> MementoResult<T>
    where
        F: Fn(&[u8]) -> MementoResult<T>;
}

/// Implementation of a `SliceReader` that expects to operate on a memory mapped
/// file (something that can be represented as a `&[u8]`).
///
/// Offsets are always computed relative to the start of the mapping.
///
/// # Errors
///
/// Out of bounds or invalid reads will result in an error being returned.
/// Examples of out of bounds or invalid reads include an offset that is
/// larger than the mapping, or a length that results in a read extending
/// beyond the end of the mapping.
pub struct SliceReaderMapped {
    map: Box<AsRef<[u8]>>,
}

impl SliceReaderMapped {
    /// Create a new `SliceReaderMapped` instance that consumes bytes from the
    /// provided byte range (typically a memory mapped file).
    pub fn new<M>(map: M) -> Self where M: AsRef<[u8]> + 'static {
        SliceReaderMapped { map: Box::new(map) }
    }

    fn read_range<F, T>(&mut self, offset: u64, len: Option<u64>, consumer: F) -> MementoResult<T>
    where
        F: Fn(&[u8]) -> MementoResult<T>,
    {
        let start = offset as usize;
        let max = self.map.as_ref().as_ref().len();
        let end = len.map(|n| (offset + n) as usize).unwrap_or(max);

        if start >= max {
            return Err(MementoError::from(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("read range start too large. start {}, max {}", start, max)
            )))
        }

        if end > max {
            return Err(MementoError::from(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("read range end too large. end {}, max {}", end, max)
            )))
        }

        consumer(&self.map.as_ref().as_ref()[start..end])
    }
}

impl SliceReader for SliceReaderMapped {
    fn consume_all<F, T>(&mut self, consumer: F) -> MementoResult<T>
    where
        F: Fn(&[u8]) -> MementoResult<T>,
    {
        self.read_range(0, None, consumer)
    }

    fn consume_from<F, T>(&mut self, offset: u64, consumer: F) -> MementoResult<T>
    where
        F: Fn(&[u8]) -> MementoResult<T>,
    {
        self.read_range(offset, None, consumer)
    }

    fn consume<F, T>(&mut self, offset: u64, len: u64, consumer: F) -> MementoResult<T>
    where
        F: Fn(&[u8]) -> MementoResult<T>,
    {
        self.read_range(offset, Some(len), consumer)
    }
}

impl Debug for SliceReaderMapped {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "SliceReaderMapped {{ map: &[...] }}")
    }
}

/// Implementation of a `SliceReader` that expects to operate on a `SeekRead`
/// implementation that refers to a file that we are doing I/O on directly.
///
/// Offsets are always computed relative to the start of the file. Any seeks
/// performed are always absolute and relative to the start of the file. The
/// cursor position is not reset after the read is performed.
///
/// The instance maintains a heap allocated buffer that is used for each read,
/// and grown to the needed capacity for the read. The buffer never shrinks in
/// capacity.
///
/// # Errors
///
/// Invalid reads will result in an error being returned. Examples of invalid
/// reads include an offset that is larger than the underlying file, or a length
/// that results in a read extending beyond the end of the file.
pub struct SliceReaderDirect {
    buffer: Vec<u8>,
    reader: Box<SeekRead>,
}

impl SliceReaderDirect {
    /// Create a new `SliceReaderDirect` instance that consumes bytes from the
    /// provided reader. The buffer used for reads is allocated upon creation
    /// with a default size appropriate for reading a Whisper file header.
    pub fn new<R>(reader: R) -> Self where R: SeekRead + 'static {
        SliceReaderDirect {
            buffer: Vec::with_capacity(DEFAULT_DIRECT_IO_BUFFER),
            reader: Box::new(reader)
        }
    }

    fn read_range<F, T>(&mut self, offset: u64, len: Option<u64>, consumer: F) -> MementoResult<T>
    where
        F: Fn(&[u8]) -> MementoResult<T>,
    {
        let reader = &mut self.reader;
        // Note that we have no way to know if the seek went beyond the end
        // of the reader. Since this is allowed by the trait and the result of
        // it is implementation defined, we don't bother trying to prevent it
        // here. Instead, we'll just return an error if we can't read anything.
        reader.seek(SeekFrom::Start(offset))?;
        self.buffer.clear();

        // If we have a specific number of bytes to read. Ensure that our
        // buffer is sized correctly and then limit the number of bytes we'll
        // read from our reader instance. Otherwise just read until EOF
        let bytes_read = if let Some(v) = len {
            let needed = v as i64 - self.buffer.capacity() as i64;
            if needed > 0 {
                self.buffer.reserve_exact(needed as usize);
            }

            reader.take(v).read_to_end(&mut self.buffer)?
        } else {
            reader.read_to_end(&mut self.buffer)?
        };

        // If we weren't able to read anything from our reader, we may have
        // ended up seeking beyond the end of the file. Return an error here
        // since this likely indicates a problem and trying to consume 0 bytes
        // isn't very useful.
        if bytes_read == 0 {
            return Err(MementoError::from(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("no bytes to read. offset {}", offset)
            )))
        }

        // If there was an explicit number of bytes we were supposed to read
        // and it doesn't match the number actually read, return an error since
        // this likely indicates a corrupt file or other problem.
        if let Some(v) = len {
            if (bytes_read as u64) < v {
                return Err(MementoError::from(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("short read. wanted {}, read {}", v, bytes_read)
                )))
            }
        }

        consumer(&self.buffer)
    }
}

impl SliceReader for SliceReaderDirect {
    fn consume_all<F, T>(&mut self, consumer: F) -> MementoResult<T>
    where
        F: Fn(&[u8]) -> MementoResult<T>,
    {
        self.read_range(0, None, consumer)
    }

    fn consume_from<F, T>(&mut self, offset: u64, consumer: F) -> MementoResult<T>
    where
        F: Fn(&[u8]) -> MementoResult<T>,
    {
        self.read_range(offset, None, consumer)
    }

    fn consume<F, T>(&mut self, offset: u64, len: u64, consumer: F) -> MementoResult<T>
    where
        F: Fn(&[u8]) -> MementoResult<T>,
    {
        self.read_range(offset, Some(len), consumer)
    }
}

impl Debug for SliceReaderDirect {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "SliceReaderDirect {{ buffer: {:?}, reader: {{...}} }}", self.buffer)
    }
}

#[cfg(test)]
mod tests {
    use std::io::{Cursor};
    use memento_core::errors::ErrorKind;
    use super::{SliceReader, SliceReaderMapped, SliceReaderDirect};

    fn get_bytes() -> Vec<u8> {
        vec![0xDE, 0xAD, 0xBE, 0xEF]
    }

    #[test]
    fn test_slice_reader_mapped_consume_all_success() {
        let map = get_bytes();
        let expected = map.clone();

        let mut slice_reader = SliceReaderMapped::new(map);
        let res = slice_reader.consume_all(|v| {
            Ok(Vec::from(v))
        });

        assert_eq!(expected, res.unwrap());
    }

    #[test]
    fn test_slice_reader_mapped_consume_from_success() {
        let map = get_bytes();
        let expected = map.clone();

        let mut slice_reader = SliceReaderMapped::new(map);
        let res = slice_reader.consume_from(0, |v| {
            Ok(Vec::from(v))
        });

        assert_eq!(expected, res.unwrap());
    }

    #[test]
    fn test_slice_reader_mapped_consume_from_bad_offset() {
        let map = get_bytes();
        let mut slice_reader = SliceReaderMapped::new(map);
        let res = slice_reader.consume_from(4, |v| {
            Ok(Vec::from(v))
        });

        let err = res.unwrap_err();
        assert_eq!(ErrorKind::IoError, err.kind());
    }

    #[test]
    fn test_slice_reader_mapped_consume_success() {
        let map = vec![0xDE, 0xAD, 0xBE, 0xEF];
        let expected = map.clone();

        let mut slice_reader = SliceReaderMapped::new(map);
        let res = slice_reader.consume(0, 4, |v| {
            Ok(Vec::from(v))
        });

        assert_eq!(expected, res.unwrap());
    }

    #[test]
    fn test_slice_reader_mapping_consume_bad_offset() {
        let map = get_bytes();
        let mut slice_reader = SliceReaderMapped::new(map);
        let res = slice_reader.consume(4, 4, |v| {
            Ok(Vec::from(v))
        });

        let err = res.unwrap_err();
        assert_eq!(ErrorKind::IoError, err.kind());
    }

    #[test]
    fn test_slice_reader_mapping_consume_bad_length() {
        let map = get_bytes();
        let mut slice_reader = SliceReaderMapped::new(map);
        let res = slice_reader.consume(2, 4, |v| {
            Ok(Vec::from(v))
        });

        let err = res.unwrap_err();
        assert_eq!(ErrorKind::IoError, err.kind());
    }

    #[test]
    fn test_slice_reader_direct_consume_all_success() {
        let bytes = get_bytes();
        let expected = bytes.clone();
        let reader = Cursor::new(bytes);

        let mut slice_reader = SliceReaderDirect::new(reader);
        let res = slice_reader.consume_all(|v| {
            Ok(Vec::from(v))
        });

        assert_eq!(expected, res.unwrap());
    }

    #[test]
    fn test_slice_reader_direct_consume_from_success() {
        let bytes = get_bytes();
        let expected = bytes.clone();
        let reader = Cursor::new(bytes);

        let mut slice_reader = SliceReaderDirect::new(reader);
        let res = slice_reader.consume_from(0, |v| {
            Ok(Vec::from(v))
        });

        assert_eq!(expected, res.unwrap());
    }

    #[test]
    fn test_slice_reader_direct_consume_from_bad_offset() {
        let bytes = get_bytes();
        let reader = Cursor::new(bytes);

        let mut slice_reader = SliceReaderDirect::new(reader);
        let res = slice_reader.consume_from(4, |v| {
            Ok(Vec::from(v))
        });

        let err = res.unwrap_err();
        assert_eq!(ErrorKind::IoError, err.kind());
    }

    #[test]
    fn test_slice_reader_direct_consume_success() {
        let bytes = get_bytes();
        let expected = bytes.clone();
        let reader = Cursor::new(bytes);

        let mut slice_reader = SliceReaderDirect::new(reader);
        let res = slice_reader.consume(0, 4, |v| {
            Ok(Vec::from(v))
        });

        assert_eq!(expected, res.unwrap());
    }

    #[test]
    fn test_slice_reader_direct_consume_bad_offset() {
        let bytes = get_bytes();
        let reader = Cursor::new(bytes);

        let mut slice_reader = SliceReaderDirect::new(reader);
        let res = slice_reader.consume(4, 4, |v| {
            Ok(Vec::from(v))
        });

        let err = res.unwrap_err();
        assert_eq!(ErrorKind::IoError, err.kind());
    }

    #[test]
    fn test_slice_reader_direct_consume_bad_length() {
        let bytes = get_bytes();
        let reader = Cursor::new(bytes);

        let mut slice_reader = SliceReaderDirect::new(reader);
        let res = slice_reader.consume(2, 4, |v| {
            Ok(Vec::from(v))
        });

        let err = res.unwrap_err();
        assert_eq!(ErrorKind::IoError, err.kind());
    }
}
