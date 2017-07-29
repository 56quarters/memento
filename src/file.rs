// read and write to file on disk


#[derive(Debug, Serialize, Deserialize)]
pub struct WhisperFile {
    header: Header,
    data: Data,
}


// 16 + (12 * num)
#[derive(Debug, Serialize, Deserialize)]
pub struct Header {
    metadata: Metadata,
    archive_info: Vec<ArchiveInfo>,
}


#[derive(Debug, Serialize, Deserialize)]
#[repr(u32)]
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

// 4 + 4 + 4 + 4 = 16
#[derive(Debug, Serialize, Deserialize)]
pub struct Metadata {
    aggregation: AggregationType,
    max_retention: u32,
    x_files_factor: f32,
    archive_count: u32,
}

// 4 + 4 + 4 = 12
#[derive(Debug, Serialize, Deserialize)]
pub struct ArchiveInfo {
    offset: u32,
    seconds_per_point: u32,
    num_points: u32,
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
    timestamp: u32,
    value: f64,
}


#[cfg(test)]
mod tests {
    use super::{WhisperFile, Header, AggregationType, Metadata,
                ArchiveInfo, Data, Archive, Point};


    #[test]
    fn test_serialize_to_json() {
        let p1 = Point { timestamp: 1501355048, value: 45.9 };
        let p2 = Point { timestamp: 1501355058, value: 30.8 };
        let p3 = Point { timestamp: 1501355068, value: 46.0 };
        let p4 = Point { timestamp: 1501355078, value: 35.3 };

        let a1 = Archive { points: vec![p1, p2, p3, p4] };
        let d1 = Data { archives: vec![a1] };

        let m1 = Metadata {
            aggregation: AggregationType::Max,
            max_retention: 86400,
            x_files_factor: 0.3,
            archive_count: 1,
        };



    }
}
