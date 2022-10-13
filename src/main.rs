use std::{fs::File, io::Read, io::Write, path::Path};

use error_chain::error_chain;
use std::thread;

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

    #[arg(short, long, default_value_t = false)]
    use_cache: bool,
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

fn walk_dir(
    dir: &str,
    forbidden: &Vec<String>,
    mut max_depth: i32,
    mut cache_file: &File,
) -> std::result::Result<(), ()> {
    if writeln!(cache_file, "{}", dir).is_err() {
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
        if walk_dir(&dir, &forbidden, max_depth, cache_file).is_err() {
            return Err(());
        };
    }

    Ok(())
}

fn walk_dir_stdout(
    dir: &str,
    forbidden: &Vec<String>,
    mut max_depth: i32,
) -> std::result::Result<(), ()> {
    println!("{}", dir);
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
        if walk_dir_stdout(&dir, &forbidden, max_depth).is_err() {
            return Err(());
        };
    }

    Ok(())
}

fn write_cache(args: Args, cache_file: &File) -> std::result::Result<(), ()> {
    for root in args.root.into_iter() {
        if walk_dir(&root, &args.forbidden, args.max_depth, cache_file).is_err() {
            return Ok(());
        }
    }

    Ok(())
}
fn write_std(args: Args) -> std::result::Result<(), ()> {
    for root in args.root.into_iter() {
        if walk_dir_stdout(&root, &args.forbidden, args.max_depth).is_err() {
            return Ok(());
        }
    }
    Ok(())
}

fn read_cache() -> std::result::Result<(), ()> {
    if !Path::new("/root/.stribog").exists() {
        return Err(());
    }

    let cache = File::open("/root/.stribog");
    if cache.is_err() {
        return Err(());
    }

    let mut cache_file = cache.unwrap();
    let mut cached_entries = String::new();
    if cache_file.read_to_string(&mut cached_entries).is_err() {
        return Err(());
    }

    println!("{}", cached_entries);
    Ok(())
}

fn write_cache_async(args: Args) -> std::result::Result<(), ()> {
    let handle = thread::spawn(|| {
        let cache = File::create("/root/.stribog");
        if cache.is_err() {
            return;
        }
        let cache_file = cache.unwrap();
        if write_cache(args, &cache_file).is_err() {
            panic!("Error writting cache");
        }
    });

    handle.join().unwrap();

    Ok(())
}

fn main() -> std::result::Result<(), ()> {
    let args = Args::parse();

    if args.use_cache {
        if read_cache().is_err() {
            return Err(());
        }
        if write_cache_async(args).is_err() {
            return Err(());
        }
    } else {
        if write_std(args).is_err() {
            return Err(());
        }
    }

    Ok(())
}
