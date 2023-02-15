use crate::folder_scanner::file_info::FileInfo;
use crossbeam_channel::Receiver;
use log::info;
use std::{collections::HashSet, thread};

pub struct Collector {
    files: HashSet<FileInfo>,
}

pub fn init(receiver: Receiver<FileInfo>) -> thread::JoinHandle<()> {
    let mut collector = Collector {
        files: HashSet::new(),
    };

    let mut counter = 0;

    return thread::spawn(move || {
        info!("Started new thread");
        loop {
            match receiver.recv() {
                Ok(file_info) => {
                    counter += 1;
                    info!("Inserted new file: {}", file_info.file);
                    collector.files.insert(file_info);
                }
                Err(_) => break,
            }
        }
        info!("Consumed {} files", counter);
    });
}
