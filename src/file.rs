// read and write to file on disk


#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct WhisperFile {
    header: Header,
    data: Data,
}


impl WhisperFile {
    pub fn new(header: Header, data: Data) -> WhisperFile {
        WhisperFile {
            header: header,
            data: data,
        }
    }

    pub fn header(&self) -> &Header {
        &self.header
    }

    pub fn data(&self) -> &Data {
        &self.data
    }
}


// 16 + (12 * num)
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Header {
    metadata: Metadata,
    archive_info: Vec<ArchiveInfo>,
}


impl Header {
    pub fn new(metadata: Metadata, archive_info: Vec<ArchiveInfo>) -> Header {
        Header {
            metadata: metadata,
            archive_info: archive_info,
        }
    }

    pub fn metadata(&self) -> &Metadata {
        &self.metadata
    }

    pub fn archive_info(&self) -> &[ArchiveInfo] {
        &self.archive_info
    }
}


#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq)]
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
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Metadata {
    aggregation: AggregationType,
    max_retention: u32,
    x_files_factor: f32,
    archive_count: u32,
}


impl Metadata {
    pub fn new(
        aggregation: AggregationType,
        max_retention: u32,
        x_files_factor: f32,
        archive_count: u32,
    ) -> Metadata {
        Metadata {
            aggregation: aggregation,
            max_retention: max_retention,
            x_files_factor: x_files_factor,
            archive_count: archive_count,
        }
    }

    pub fn aggregation(&self) -> AggregationType {
        self.aggregation
    }

    pub fn max_retention(&self) -> u32 {
        self.max_retention
    }

    pub fn x_files_factor(&self) -> f32 {
        self.x_files_factor
    }

    pub fn archive_count(&self) -> u32 {
        self.archive_count
    }
}


// 4 + 4 + 4 = 12
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct ArchiveInfo {
    offset: u32,
    seconds_per_point: u32,
    num_points: u32,
}


impl ArchiveInfo {
    pub fn new(offset: u32, seconds_per_point: u32, num_points: u32) -> ArchiveInfo {
        ArchiveInfo {
            offset: offset,
            seconds_per_point: seconds_per_point,
            num_points: num_points,
        }
    }

    pub fn offset(&self) -> u32 {
        self.offset
    }

    pub fn seconds_per_point(&self) -> u32 {
        self.seconds_per_point
    }

    pub fn num_points(&self) -> u32 {
        self.num_points
    }
}


#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
pub struct Data {
    archives: Vec<Archive>,
}


impl Data {
    pub fn new(archives: Vec<Archive>) -> Data {
        Data {
            archives: archives,
        }
    }

    pub fn archives(&self) -> &[Archive] {
        &self.archives
    }
}


#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
pub struct Archive {
    points: Vec<Point>,
}


impl Archive {
    pub fn new(points: Vec<Point>) -> Archive {
        Archive {
            points: points
        }
    }

    pub fn points(&self) -> &[Point] {
        &self.points
    }
}


// 4 + 8 = 12
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Point {
    timestamp: u32,
    value: f64,
}


impl Point {
    pub fn new(timestamp: u32, value: f64) -> Point {
        Point {
            timestamp: timestamp,
            value: value,
        }
    }

    pub fn timestamp(&self) -> u32 {
        self.timestamp
    }

    pub fn value(&self) -> f64 {
        self.value
    }
}


#[cfg(test)]
mod tests {
}
