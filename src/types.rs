pub struct WhisperFile {
    header: Header,
    data: Data,
}


pub struct Header {
    metadata: Metadata,
    archive_info: Vec<ArchiveInfo>,
}


pub enum AggregationType {
    Average,
    Sum,
    Last,
    Max,
    Min,
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
    interval: u64,
    valid: f64,
}
