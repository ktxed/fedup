use std::{
    thread::{self, JoinHandle}, path::Path, fs,
};

use clap::{ValueEnum, builder::Str};
use crossbeam_channel::Receiver;
use log::{error, info, warn};

use crate::Action;
use data_encoding::BASE64;
use super::duplicates_group::{DuplicatesGroup, HashedSample};

pub struct DeduplicatorResultProcessor {
    pub duplicates: Vec<DuplicatesGroup>
}

pub trait ResultProcessor<T> {
    fn start(receiver: Receiver<Option<DuplicatesGroup>>) -> JoinHandle<T>;
    fn apply(&self, action: ActionInput) -> Option<()>;
}

#[derive(Clone, Debug)]
pub struct MoveParams {
    destination_folder: String
}

#[derive(Debug)]
pub enum ActionInput {
    Move(MoveParams),
    Report
}

impl ActionInput {
    pub fn from(action: Action, folder: Option<String>) -> Self {
        match action {
            Action::Move => ActionInput::Move{ 0: MoveParams { destination_folder: folder.unwrap() }},
            Action::Report => ActionInput::Report,
            Action::Delete => todo!(),
        }
    }
}

fn listen(r: Receiver<Option<DuplicatesGroup>>) -> Vec<DuplicatesGroup> {
    let mut duplicates: Vec<DuplicatesGroup> = vec![];
    loop {
        match r.recv() {
            Ok(maybe_group) => match maybe_group {
                Some(group) => {
                    duplicates.push(group);
                }
                None => {
                    info!("Received all duplicate pairs");
                    break;
                }
            },
            Err(e) => {
                error!("Error: {}", e);
                break;
            }
        }
    }
    return duplicates;
}

impl DeduplicatorResultProcessor {
    fn new(duplicates: Vec<DuplicatesGroup>) -> Self {
        DeduplicatorResultProcessor {
            duplicates: duplicates
        }
    }
}

/**
 * base64 encode original file name and path and move it to the destination folder
 * the older file in a group is left in its original location
 */
fn move_duplicates(duplicates: &Vec<DuplicatesGroup>, destination_folder: &str) -> () {
    info!("Moving duplicates to {}", destination_folder);
    duplicates.into_iter()
    .for_each(|group| {
        let i = reorder_group(group.item.to_vec());
        i.iter().skip(1).for_each(|item| {
            let input_file_path = item.sample.file_info.file.clone();
            let b64name = BASE64.encode(input_file_path.as_bytes());
            let target_file_path = Path::new(destination_folder).join(b64name);
            let input_path = Path::new(&item.sample.file_info.file);
            move_file(input_path, target_file_path.as_path());
        })
    })
}

// sort group by ascending path in order to keep files with the shortest paths
// can be changed to order files by date in order to move only newer duplicates
fn reorder_group(mut group: Vec<HashedSample>) -> Vec<HashedSample> {
    group.sort_by_key(|hs| hs.sample.file_info.file.len());     
    group
}

fn report_duplicates(duplicates: &Vec<DuplicatesGroup>) -> () {
    info!("Duplicates summary. There are {} duplicate groups", duplicates.len());
    duplicates.into_iter()
    .for_each(|group| {
        info!("Displaying duplicate group with {} items...", group.item.len());
        let i = reorder_group(group.item.to_vec());
        info!("To keep: {}", i[0].sample.file_info.file);
        i.iter().skip(1).for_each(|item| {
            info!("To move: {}", item.sample.file_info.file);
        })
    })
}

fn move_file(input_file_path: &Path, target_file_path: &Path) {
    info!("Moving {} to {}", input_file_path.to_string_lossy(), target_file_path.to_string_lossy());
    match fs::rename(input_file_path, target_file_path) {
        Ok(_) => info!("Moved."),
        Err(error) => warn!("Move failed: {}", error)
    };
}

impl ResultProcessor<DeduplicatorResultProcessor> for DeduplicatorResultProcessor {
    fn start(receiver: Receiver<Option<DuplicatesGroup>>) -> JoinHandle<DeduplicatorResultProcessor> {
        let r = receiver.clone();
        let listener = thread::spawn(move || {
            info!("Started processor");
            return DeduplicatorResultProcessor::new(listen(r));
        });
        return listener;
    }
    fn apply(&self, action: ActionInput) -> Option<()> {
        if self.duplicates.len() == 0 {
            return None;
        }
        info!("Applying action {:?} for {} duplicate pairs", action, self.duplicates.len());
        match action {
            ActionInput::Move(params) => move_duplicates(&self.duplicates, &params.destination_folder),
            ActionInput::Report => report_duplicates(&self.duplicates)
        }
        return  None;
    }
}

