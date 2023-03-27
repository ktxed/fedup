use super::file_info::FileInfo;

#[derive(Clone)]
pub struct Sample {
    pub file_info: FileInfo,
    pub bytes: Vec<u8>,
}

#[derive(Clone)]
pub struct HashedSample {
    pub sample: Sample,
    pub hash: String,
}

#[derive(Clone)]
pub struct DuplicatesGroup {
    pub item: Vec<HashedSample>
}