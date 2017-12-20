extern crate memento;

use memento::{FetchRequest, MappedFileStream, WhisperFileReader};
use memento::errors::{ErrorKind, WhisperError, WhisperResult};
use memento::types::{Header, Point, WhisperFile};

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
