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


pub struct WhisperPoint(f64, i64);


impl WhisperPoint {
    pub fn new(value: f64, timestamp: i64) -> WhisperPoint {
        WhisperPoint(value, timestamp)
    }

    pub fn value(&self) -> f64 {
        self.0
    }

    pub fn timestamp(&self) -> i64 {
        self.1
    }
}


pub trait WhisperArchive {

    fn info(&self) -> WhisperResult<WhisperInfo>;

    fn read(&self, from: i64, until: Option<i64>) -> WhisperResult<Vec<WhisperPoint>>;

    fn write(&mut self, point: &WhisperPoint) -> WhisperResult<()>;

    fn write_many(&mut self, values: &[WhisperPoint]) -> WhisperResult<()>;
}
