// Whisper
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


pub type WhisperResult<T> = Result<T, WhisperError>;


#[derive(Debug)]
enum ErrorRepr {
    IoError(io::Error),
    ParseError(nom::IError),
    WithDescription(ErrorKind, &'static str),
}


// TODO: Add some more variants to this:
// * InvalidDateRange
// * InvalidDateStart
// * InvalidDateEnd
// * CorruptDatabase

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum ErrorKind {
    IoError, // Errors reading or writing beyond our control
    ParseError, // Malformed whisper files
    InvalidInput, // Invalid input from a user
}


#[derive(Debug)]
pub struct WhisperError {
    repr: ErrorRepr,
}


impl WhisperError {
    pub fn kind(&self) -> ErrorKind {
        match self.repr {
            ErrorRepr::IoError(_) => ErrorKind::IoError,
            ErrorRepr::ParseError(_) => ErrorKind::ParseError,
            ErrorRepr::WithDescription(kind, _) => kind,
        }
    }
}


impl fmt::Display for WhisperError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self.repr {
            ErrorRepr::IoError(ref err) => err.fmt(f),
            ErrorRepr::ParseError(ref err) => {
                match *err {
                    nom::IError::Error(ref e) => e.fmt(f),
                    nom::IError::Incomplete(need) => write!(f, "incomplete: {:?}", need),
                }
            }
            ErrorRepr::WithDescription(_, desc) => desc.fmt(f),
        }
    }
}


impl error::Error for WhisperError {
    fn description(&self) -> &str {
        match self.repr {
            ErrorRepr::IoError(ref err) => err.description(),
            ErrorRepr::ParseError(ref err) => {
                match *err {
                    nom::IError::Error(ref e) => e.description(),
                    nom::IError::Incomplete(_) => "incomplete",
                }
            }
            ErrorRepr::WithDescription(_, desc) => desc,
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match self.repr {
            ErrorRepr::IoError(ref err) => Some(err),
            ErrorRepr::ParseError(ref err) => {
                match *err {
                    nom::IError::Error(ref e) => Some(e),
                    _ => None,
                }
            }
            _ => None,
        }
    }
}


impl From<io::Error> for WhisperError {
    fn from(err: io::Error) -> WhisperError {
        WhisperError { repr: ErrorRepr::IoError(err) }
    }
}


impl From<nom::IError> for WhisperError {
    fn from(err: nom::IError) -> WhisperError {
        WhisperError { repr: ErrorRepr::ParseError(err) }
    }
}


impl From<(ErrorKind, &'static str)> for WhisperError {
    fn from((kind, msg): (ErrorKind, &'static str)) -> WhisperError {
        WhisperError { repr: ErrorRepr::WithDescription(kind, msg) }
    }
}
