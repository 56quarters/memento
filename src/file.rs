// read and write to file on disk


#[derive(Debug, Serialize, Deserialize)]
pub struct WhisperFile {
    header: Header,
    data: Data,
}


#[derive(Debug, Serialize, Deserialize)]
pub struct Header {
    metadata: Metadata,
    archive_info: Vec<ArchiveInfo>,
}


#[derive(Debug, Serialize, Deserialize)]
#[repr(u64)]
pub enum AggregationType {
    Average = 1,
    Sum = 2,
    Last = 3,
    Max = 4,
    Min = 5,
    AvgZero = 6,
    AbsMax = 7,
    AbsMin = 8,
}


#[derive(Debug, Serialize, Deserialize)]
pub struct Metadata {
    aggregation: AggregationType,
    max_retention: u64,
    x_files_factor: f32,
    archive_count: u64,
}


#[derive(Debug, Serialize, Deserialize)]
pub struct ArchiveInfo {
    offset: u64,
    seconds_per_point: u64,
    num_points: u64,
}


#[derive(Debug, Serialize, Deserialize)]
pub struct Data {
    archives: Vec<Archive>,
}


#[derive(Debug, Serialize, Deserialize)]
pub struct Archive {
    points: Vec<Point>,
}


#[derive(Debug, Serialize, Deserialize)]
pub struct Point {
    timestamp: u64,
    value: f64,
}
