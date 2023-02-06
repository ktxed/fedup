use std::fs::{self, DirEntry};
use std::io;
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
    }
    Ok(())
}

pub fn scan(folder: &String) -> () {
    println!("Starting scanning folder {}", folder);

    let cb = |entry: &DirEntry| {
        println!("{}", entry.path().display());
    };

    match visit_dirs(Path::new(folder), &cb) {
        Ok(_) => println!("Scanning finished"),
        Err(_) => println!("Error scanning"),
    }
}
