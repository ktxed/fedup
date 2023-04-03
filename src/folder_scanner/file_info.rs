use std::time::SystemTime;

#[derive(Clone)]
pub struct FileInfo {
    pub file: String,
    pub size: u64,
    pub date: SystemTime
}
