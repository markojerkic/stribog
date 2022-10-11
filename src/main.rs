use std::{env, fs};

fn main() {
    let current_dir = env::current_dir().unwrap();
    println!(
        "Entries modified in the last 24 hours in {:?}:",
        current_dir
    );

    for entry_res in fs::read_dir(current_dir).unwrap() {
        let entry = entry_res.unwrap();
        let path = entry.path();

        let metadata = fs::metadata(&path).unwrap();
        let last_modified = metadata.modified().unwrap().elapsed().unwrap().as_secs();

        if last_modified < 5 * 24 * 3600 && metadata.is_file() {
            println!(
                "Last modified: {:?} seconds, is read only: {:?}, size: {:?} bytes, filename: {:?}",
                last_modified,
                metadata.permissions().readonly(),
                metadata.len(),
                path.file_name().ok_or("No filename")
            );
        }
    }
}
