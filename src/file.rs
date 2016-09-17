// read and write to file on disk


pub struct WhisperFile {
    header: Header,
    data: Data,
}


pub struct Header {
    metadata: Metadata,
    archive_info: Vec<ArchiveInfo>,
}


#[repr(u64)]
pub enum AggregationType {
    Average = 0,
    Sum = 1,
    Last = 2,
    Max = 3,
    Min = 4,
}


pub struct Metadata {
    aggregation: AggregationType,
    max_retention: u64,
    x_files_factor: f64,
    archive_count: u64,
}


pub struct ArchiveInfo {
    offset: u64,
    seconds_per_point: u64,
    num_points: u64,
}


pub struct Data {
    archives: Vec<Archive>,
}


pub struct Archive {
    points: Vec<Point>,
}


pub struct Point {
    timestamp: u64,
    value: f64,
}
