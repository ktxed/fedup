use std::fs::{self, DirEntry};
use std::io::{self, ErrorKind};
use std::path::Path;
use std::time::SystemTime;
use crossbeam_channel::Sender;
use log::info;
use crate::folder_scanner::file_info::FileInfo;

fn visit_dirs(dir: &Path, cb: &dyn Fn(&DirEntry)) -> io::Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                visit_dirs(&path, cb)?;
            } else {
                cb(&entry);
            }
        }
    } else {
        return Err(io::Error::new(ErrorKind::Other, "Path is not a folder"));
    }
    Ok(())
}

pub fn scan(folder: &String, sender: &Sender<FileInfo>) -> () {
    info!("Starting scanning folder: {}", folder);

    let cb = |entry: &DirEntry| {
        if let Ok(metadata) = entry.metadata() {
            let file_info = FileInfo {
                file: entry.path().display().to_string(), 
                size: metadata.len(),
                date: metadata.created().unwrap_or(SystemTime::UNIX_EPOCH)
            };
            match sender.send(file_info) {
                Ok(_) => (),
                Err(error) => info!("Problem sending message: {}", error),
            }
        }
    };

    match visit_dirs(Path::new(folder), &cb) {
        Ok(_) => info!("Scanning finished"),
        Err(error) => println!("{}", error),
    }
}
