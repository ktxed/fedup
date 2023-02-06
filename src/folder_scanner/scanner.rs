use std::fs::{self, DirEntry};
use std::io::{self, ErrorKind};
use std::path::Path;

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

pub fn scan(folder: &String) -> () {
    println!("Starting scanning folder: {}", folder);

    let cb = |entry: &DirEntry| {
        println!("{}", entry.path().display());
        if let Ok(metadata) = entry.metadata() {
            println!("\tfilesize: {}", metadata.len());
        }
    };

    match visit_dirs(Path::new(folder), &cb) {
        Ok(_) => println!("Scanning finished"),
        Err(error) => println!("{}", error),
    }
}
