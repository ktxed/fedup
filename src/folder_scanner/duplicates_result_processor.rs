use std::{
    thread::{self, JoinHandle},
};

use clap::{ValueEnum, builder::Str};
use crossbeam_channel::Receiver;
use log::{error, info};

use crate::Action;

use super::duplicates_group::DuplicatesGroup;

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
    Move(MoveParams)
}

impl ActionInput {
    pub fn from(action: Action, folder: String) -> Self {
        ActionInput::Move{ 0: MoveParams { destination_folder: folder }}
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
fn moveDuplicates(duplicates: &Vec<DuplicatesGroup>, destination_folder: &str) -> () {
    info!("Moving duplicates to {}", destination_folder);
    duplicates.iter()
    .for_each(|group| {
        // sort group by creation date, ascending
        // and skip first element
    })
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
            ActionInput::Move(params) => moveDuplicates(&self.duplicates, &params.destination_folder)
        }
        return  None;
    }
}

