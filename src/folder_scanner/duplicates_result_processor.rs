use std::{
    sync::{Arc, Mutex},
    thread::{self, JoinHandle},
};

use crossbeam_channel::Receiver;
use log::{error, info};

use super::duplicates_group::DuplicatesGroup;

pub struct DeduplicatorResultProcessor {
    pub receiver: Receiver<Option<DuplicatesGroup>>,
}

pub trait ResultProcessor {
    fn start(&self) -> JoinHandle<()>;
}

fn listen(r: Receiver<Option<DuplicatesGroup>>) {
    let mut counter = 0;
    loop {
        match r.recv() {
            Ok(maybe_group) => match maybe_group {
                Some(group) => {
                    let path = &group.item.get(0).unwrap().sample.file_info.file;
                    info!("Duplicate group found, first path {}", path);
                    group
                        .item
                        .iter()
                        .for_each(|item| info!("---- path: {}", item.sample.file_info.file));
                    counter += 1;
                }
                None => {
                    info!("Processing finished. Received {} groups", counter);
                    break;
                }
            },
            Err(e) => {
                error!("Error: {}", e);
                break;
            }
        }
    }
}

impl ResultProcessor for DeduplicatorResultProcessor {
    fn start(&self) -> JoinHandle<()> {
        let r = self.receiver.clone();
        return thread::spawn(move || {
            info!("Started processor");
            listen(r);
        });
    }
}
