mod folder_scanner;

use std::{thread, sync::{Arc, Mutex}, time};
use clap::{ Parser };
use crossbeam_channel::unbounded;
use folder_scanner::duplicates_result_processor::Action;
use log::{info, error};

use crate::folder_scanner::{duplicates_group::DuplicatesGroup, duplicates_result_processor::{DeduplicatorResultProcessor, ResultProcessor}};

#[derive(Parser, Debug)]
#[command()]
struct Args {
    #[arg(short, long)]
    folder: String,
    
    #[arg(short, long, value_enum)]
    action: Action,
}

#[warn(unused_must_use)]
fn main() {
    env_logger::init();

    let (s, r) = unbounded();
    let args = Args::parse();

    info!("Input folder {}", args.folder);
    info!("Action for duplicates: {:?}", args.action);

    let collector_thread = folder_scanner::collector::init(r);
    let scanner_thread = thread::spawn(move || {
        folder_scanner::scanner::scan(&args.folder, &s);
    });
    
    scanner_thread.join();
    
    let (s, r) = unbounded();

    match collector_thread.join() {
        Ok(collector_result) => {
            match s.send(collector_result) {
                Ok(_) => (),
                Err(_) => error!("Failed sending message"),
            }
        }
        Err(error) => error!("Collector failed {:?}", error)
    }

    let (ss, result_receiver ) = unbounded();

    let processor = DeduplicatorResultProcessor { receiver: result_receiver };
    folder_scanner::deduplicator::deduplicate(r, &ss).join();

    processor.start().join();
    
    info!("Exiting...");
}
