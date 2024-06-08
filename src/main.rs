mod folder_scanner;

use clap::{Parser, ValueEnum};
use crossbeam_channel::unbounded;
use log::{error, info};
use std::thread;

use crate::folder_scanner::duplicates_result_processor::{
    ActionInput, DeduplicatorResultProcessor, ResultProcessor,
};

#[derive(Clone, Debug, ValueEnum)]
pub enum Action {
    Move,
    Report,
    Delete,
}

#[derive(Parser, Debug)]
#[command()]
struct Args {
    #[arg(short, long)]
    folder: String,

    #[arg(short, long, value_enum)]
    action: Action,

    #[arg(short, long)]
    destination_folder: String,
}

#[warn(unused_must_use)]
fn main() {
    std::env::set_var("RUST_LOG", "info");
    env_logger::init();

    let (s, r) = unbounded();
    let args = Args::parse();

    info!("Input folder {}", args.folder);
    info!("Action for duplicates: {:?}", args.action);

    let collector_thread = folder_scanner::collector::init(r);
    let scanner_thread = thread::spawn(move || {
        folder_scanner::scanner::scan(&args.folder, &s);
    });

    scanner_thread.join().expect("Concurrency error");

    let (s, r) = unbounded();

    match collector_thread.join() {
        Ok(collector_result) => match s.send(collector_result) {
            Ok(_) => (),
            Err(_) => error!("Failed sending message"),
        },
        Err(error) => error!("Collector failed {:?}", error),
    }

    let (ss, result_receiver) = unbounded();

    folder_scanner::deduplicator::deduplicate(r, &ss)
        .join()
        .expect("Deduplication thread failed");

    let processor: DeduplicatorResultProcessor =
        DeduplicatorResultProcessor::start(result_receiver)
            .join()
            .unwrap();

    let action = match args.action {
        Action::Move => ActionInput::from(Action::Move, Some(args.destination_folder)),
        Action::Report => ActionInput::from(Action::Report, None),
        Action::Delete => todo!(),
    };

    processor.apply(action);

    info!("Exiting...");
}
