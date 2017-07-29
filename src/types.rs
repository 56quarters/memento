// Domain objects that callers would actually work with


pub enum ErrorKind {
}

pub struct WhisperError {
    repr: ErrorRepr,
}

enum ErrorRepr {
}

pub type WhisperResult<T> = Result<T, WhisperError>;


// Some metadata?
pub struct WhisperInfo;


pub struct WhisperPoint(u64, f64);


impl WhisperPoint {
    pub fn new(timestamp: u64, value: f64) -> WhisperPoint {
        WhisperPoint(timestamp, value)
    }

    pub fn timestamp(&self) -> u64 {
        self.0
    }

    pub fn value(&self) -> f64 {
        self.1
    }
}


pub trait WhisperArchive {

    fn info(&self) -> WhisperResult<WhisperInfo>;

    fn read(&self, from: u64, until: Option<u64>) -> WhisperResult<Vec<WhisperPoint>>;

    fn write(&mut self, point: &WhisperPoint) -> WhisperResult<()>;

    fn write_many(&mut self, values: &[WhisperPoint]) -> WhisperResult<()>;
}
