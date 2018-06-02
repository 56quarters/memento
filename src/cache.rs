use std::path::{Path, PathBuf};
use std::fs::File;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

use memmap::Mmap;

use memento_core::errors::MementoResult;
use io::{SliceReaderDirect, SliceReaderMapped};


pub struct ReaderSupplier {
    cache: Arc<Mutex<HashMap<PathBuf, SliceReaderMapped>>>,
}

impl ReaderSupplier {
    pub fn new() -> Self {
        ReaderSupplier {
            cache: Arc::new(Mutex::new(HashMap::new()))
        }
    }

    pub fn new_direct_reader<P>(&mut self, path: P) -> MementoResult<SliceReaderDirect>
    where
        P: AsRef<Path>,
    {
       let file = File::open(path)?;
       Ok(SliceReaderDirect::new(file))
    }

    pub fn new_mapped_reader<P>(&self, path: P) -> MementoResult<SliceReaderMapped>
    where
        P: AsRef<Path>,
    {
        let mut cache = self.cache.lock().unwrap();
        let path_buf = path.as_ref().to_path_buf();

        if !cache.contains_key(&path_buf) {
            let file = File::open(&path)?;
            let map = unsafe { Mmap::map(&file)? };
            cache.insert(path_buf.clone(), SliceReaderMapped::new(map));
        }

        unimplemented!();
        //Ok(cache.get(&path_buf).unwrap())
    }
}
