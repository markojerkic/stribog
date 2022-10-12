use error_chain::error_chain;

use clap::Parser;
use walkdir::WalkDir;

/// A simple file tree traverer
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Root of search
    #[arg(short, long)]
    root: String,

    /// List of forbiden directory names.
    /// Any dir which name starts with any of the entryies will be skipped and not walked into
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

fn walk_dir(dir: &str, forbidden: &Vec<String>, mut max_depth: i32) -> Result<()> {
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
    let mut res = Ok(());
    if dir.len() == 0 {
        return Ok(());
    }
    for dir in dirs {
        res = walk_dir(&dir, &forbidden, max_depth);
    }

    res
}

fn main() -> Result<()> {
    let args = Args::parse();

    let res = walk_dir(&args.root, &args.forbidden, args.max_depth);
    res
}
