use crate::folder_scanner::file_info::FileInfo;
use crossbeam_channel::Receiver;
use log::info;
use std::{
    collections::HashMap,
    thread::{self, JoinHandle},
};

pub struct CollectorResult {
    pub buckets: Vec<Vec<FileInfo>>
}

pub fn init(receiver: Receiver<FileInfo>) -> JoinHandle<CollectorResult> {
    let mut counter = 0;
    let mut buckets: HashMap<u64, Vec<FileInfo>> = HashMap::new();

    let worker = thread::spawn(move || {
        info!("Started new thread");
        loop {
            match receiver.recv() {
                Ok(file_info) => {
                    counter += 1;
                    info!("Inserted new file: {}", file_info.file);
                    bucket_file_by_size(&mut buckets, file_info);
                }
                Err(_) => break,
            }
        }
        info!("Consumed {} files. Filtering...", counter);
        let filtered = filter_potential_duplicates(buckets);
        info!("Found {} potential duplicate pairs", filtered.len());
        let result = CollectorResult { buckets: filtered };
        return result;
    });
    return worker;
}

fn filter_potential_duplicates(buckets: HashMap<u64, Vec<FileInfo>>) -> Vec<Vec<FileInfo>> {
    return buckets.into_iter()
        .filter(|entry| entry.1.len() > 1)
        .map(|entry| entry.1)
        .collect::<Vec<_>>();
}

fn bucket_file_by_size(buckets: &mut HashMap<u64, Vec<FileInfo>>, file_info: FileInfo) {
    if buckets.contains_key(&file_info.size) {
        match buckets.get_mut(&file_info.size) {
            Some(bucket) => bucket.push(file_info),
            None => (),
        }
    } else {
        let bucket_key = file_info.size;
        let mut bucket = Vec::new();
        bucket.push(file_info);
        buckets.insert(bucket_key, bucket);
    }
}
