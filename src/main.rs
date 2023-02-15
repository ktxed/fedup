mod folder_scanner;

use std::thread;
use clap::Parser;
use crossbeam_channel::unbounded;
use log::info;

#[derive(Parser, Debug)]
#[command()]
struct Args {
    #[arg(short, long)]
    folder: String,
}

#[warn(unused_must_use)]
fn main() {
    env_logger::init();

    let (s, r) = unbounded();
    let args = Args::parse();
    let collector_thread = folder_scanner::collector::init(r);
    let scanner_thread = thread::spawn(move || {
        folder_scanner::scanner::scan(&args.folder, &s);
    });
    
    scanner_thread.join();
    collector_thread.join();
    info!("Exiting...");
}
