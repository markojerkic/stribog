use std::panic;

use error_chain::error_chain;

use clap::Parser;
use walkdir::WalkDir;

/// A simple file tree traverer
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// List of root directories to search
    #[arg(short, long, num_args = 1.., required=true)]
    root: Vec<String>,

    /// List of forbiden directory names.
    /// Any dir which name starts with any of the entries will be skipped and not walked into
    #[arg(short, long)]
    forbidden: Vec<String>,

    /// Max depth to walk
    #[arg(short, long, default_value_t = i32::MAX)]
    max_depth: i32,
}

error_chain! {
    foreign_links {
        WalkDir(walkdir::Error);
        Io(std::io::Error);
        SystemTime(std::time::SystemTimeError);
    }
}

fn is_whitelisted(file_name: &str, forbidden: &Vec<String>) -> bool {
    for forbidden_entry in forbidden {
        let forbidden_name = (*forbidden_entry).as_str();
        let starts_with = file_name.starts_with(forbidden_name);
        if starts_with {
            return false;
        }
    }

    true
}

fn walk_dir(dir: &str, forbidden: &Vec<String>, mut max_depth: i32) -> std::result::Result<(), ()> {
    if panic::catch_unwind(|| println!("{}", dir)).is_err() {
        return Err(());
    }
    if max_depth <= 0 {
        return Ok(());
    }
    max_depth -= 1;

    let walker = WalkDir::new(dir);

    let dirs = walker
        .max_depth(1)
        .into_iter()
        .filter(|entry| entry.is_ok())
        .map(|entry| entry.unwrap())
        .filter(|entry| entry.file_type().is_dir())
        .filter(|entry| {
            is_whitelisted(
                entry.file_name().to_string_lossy().to_string().as_str(),
                forbidden,
            )
        })
        .map(|entry| entry.path().display().to_string())
        .filter(|entry| entry.ne(dir) && entry.ne("/"));
    if dir.len() == 0 {
        return Ok(());
    }
    for dir in dirs {
        if walk_dir(&dir, &forbidden, max_depth).is_err() {
            return Err(());
        };
    }

    Ok(())
}

fn main() -> std::result::Result<(), ()> {
    let args = Args::parse();
    if args.root.len() <= 0 {
        panic!("Must pass at least one root dir");
    }

    for root in args.root.into_iter() {
        if walk_dir(&root, &args.forbidden, args.max_depth).is_err() {
            return Ok(());
        }
    }
    Ok(())
}
