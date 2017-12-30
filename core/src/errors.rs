// Memento - A Whisper implemention in Rust
//
// Copyright 2017 TSH Labs
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Core types for the Whisper library

use std::error;
use std::io;
use std::fmt;

use nom;

pub type MementoResult<T> = Result<T, MementoError>;

#[derive(Debug)]
enum ErrorRepr {
    IoError(io::Error),
    ParseError(nom::IError),
    WithDescription(ErrorKind, &'static str),
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum ErrorKind {
    IoError,
    ParseError,
    InvalidTimeRange,
    InvalidTimeStart,
    InvalidTimeEnd,
    NoArchiveAvailable,
    CorruptDatabase,
}

#[derive(Debug)]
pub struct MementoError {
    repr: ErrorRepr,
}

impl MementoError {
    pub fn kind(&self) -> ErrorKind {
        match self.repr {
            ErrorRepr::IoError(_) => ErrorKind::IoError,
            ErrorRepr::ParseError(_) => ErrorKind::ParseError,
            ErrorRepr::WithDescription(kind, _) => kind,
        }
    }
}

impl fmt::Display for MementoError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self.repr {
            ErrorRepr::IoError(ref err) => err.fmt(f),
            ErrorRepr::ParseError(ref err) => match *err {
                nom::IError::Error(ref e) => e.fmt(f),
                nom::IError::Incomplete(need) => write!(f, "incomplete: {:?}", need),
            },
            ErrorRepr::WithDescription(_, desc) => desc.fmt(f),
        }
    }
}

impl error::Error for MementoError {
    fn description(&self) -> &str {
        match self.repr {
            ErrorRepr::IoError(ref err) => err.description(),
            ErrorRepr::ParseError(ref err) => match *err {
                nom::IError::Error(ref e) => e.description(),
                nom::IError::Incomplete(_) => "incomplete",
            },
            ErrorRepr::WithDescription(_, desc) => desc,
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match self.repr {
            ErrorRepr::IoError(ref err) => Some(err),
            ErrorRepr::ParseError(ref err) => match *err {
                nom::IError::Error(ref e) => Some(e),
                _ => None,
            },
            _ => None,
        }
    }
}

impl From<io::Error> for MementoError {
    fn from(err: io::Error) -> MementoError {
        MementoError {
            repr: ErrorRepr::IoError(err),
        }
    }
}

impl From<nom::IError> for MementoError {
    fn from(err: nom::IError) -> MementoError {
        MementoError {
            repr: ErrorRepr::ParseError(err),
        }
    }
}

impl From<(ErrorKind, &'static str)> for MementoError {
    fn from((kind, msg): (ErrorKind, &'static str)) -> MementoError {
        MementoError {
            repr: ErrorRepr::WithDescription(kind, msg),
        }
    }
}
