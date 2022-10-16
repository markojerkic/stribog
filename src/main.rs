use error_chain::error_chain;
use std::{fs::File, io::Read, io::Write, path::Path};
use string_builder::Builder;

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

    /// Opt in to use cached valus. Always a little bit behind, but should be faster
    #[arg(short, long, default_value_t = false)]
    use_cache: bool,

    /// If selected, nothing will be printed to stdout. The result of directory tree walking will be written in the cache file.
    #[arg(short, long, default_value_t = false)]
    cache_only: bool,

    /// If selected, deamon service will not be run
    #[arg(short, long, default_value_t = false)]
    no_deamon: bool,

    /// Set if used on linux
    #[arg(short, long, default_value_t = false)]
    is_linux: bool,
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
    builder: &mut Builder,
) -> std::result::Result<(), String> {
    (*builder).append(dir);
    (*builder).append("\n");

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
        match walk_dir(&dir, &forbidden, max_depth, builder) {
            Ok(ok) => ok,
            Err(err) => return Err(err),
        };
    }

    Ok(())
}

fn walk_dir_stdout(
    dir: &str,
    forbidden: &Vec<String>,
    mut max_depth: i32,
) -> std::result::Result<(), String> {
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
        match walk_dir_stdout(&dir, &forbidden, max_depth) {
            Ok(ok) => ok,
            Err(err) => return Err(err),
        };
    }

    Ok(())
}

fn write_cache(args: Args, builder: &mut Builder) -> std::result::Result<(), String> {
    for root in args.root.into_iter() {
        match walk_dir(&root, &args.forbidden, args.max_depth, builder) {
            Ok(ok) => ok,
            Err(err) => return Err(err),
        }
    }

    Ok(())
}
fn write_std(args: Args) -> std::result::Result<(), String> {
    for root in args.root.into_iter() {
        match walk_dir_stdout(&root, &args.forbidden, args.max_depth) {
            Ok(ok) => ok,
            Err(err) => return Err(err),
        }
    }
    Ok(())
}

fn get_cache_file_name(is_linux: bool) -> String {
    if is_linux {
        return "/root/dev/.stribog".to_owned();
    } else {
        return "C:\\Dev\\.stribog".to_owned();
    }
}

fn read_cache(is_linux: bool) -> std::result::Result<(), String> {
    let cache_file_name = get_cache_file_name(is_linux);
    if !Path::new(&cache_file_name).exists() {
        return Err("Error checking if file exists".to_owned());
    }

    let cache = File::open(cache_file_name);
    if cache.is_err() {
        return Err(cache.unwrap_err().to_string());
    }

    let mut cache_file = cache.unwrap();
    let mut cached_entries = String::new();
    match cache_file.read_to_string(&mut cached_entries) {
        Ok(res) => res,
        Err(err) => return Err(err.to_string()),
    };

    println!("{}", cached_entries);
    Ok(())
}

fn write_cache_deamon(args: Args) -> std::result::Result<(), String> {
    let is_linux = args.is_linux;
    let mut builder = Builder::default();
    match write_cache(args, &mut builder) {
        Ok(_ok) => _ok,
        Err(err) => return Err(err),
    }
    let cache_file_name = &get_cache_file_name(is_linux);

    let cache_file = match File::create(cache_file_name) {
        Ok(ok) => ok,
        Err(err) => return Err(err.to_string()),
    };

    writeln!(
        &cache_file,
        "{}",
        builder.string().expect("Building string failed")
    )
    .expect("Writting to file failed");

    Ok(())
}

#[cfg(not(windows))]
fn deamon(args: Args) -> std::result::Result<(), String> {
    use daemonize::Daemonize;
    let daemonize = Daemonize::new()
        .umask(0o777)
        .privileged_action(|| write_cache_deamon(args));
    match daemonize.start() {
        Ok(_ok) => return Ok(()),
        Err(err) => return Err(err.to_string()),
    }
}

#[cfg(windows)]
fn deamon(args: Args) -> std::result::Result<(), String> {
    use std::thread;

    return thread::spawn(|| write_cache_deamon(args))
        .join()
        .expect("join failed");
}

fn main() -> std::result::Result<(), String> {
    let args = Args::parse();

    if args.use_cache {
        let cn = &get_cache_file_name(args.is_linux);
        if !Path::new(&cn).exists() {
            let _cache = match File::create(cn) {
                Ok(ok) => ok,
                Err(err) => return Err(err.to_string()),
            };
        }

        let is_linux = args.is_linux;

        if !args.cache_only {
            match read_cache(is_linux) {
                Ok(ok) => ok,
                Err(err) => return Err(err),
            }
        }
        if !args.no_deamon {
            match deamon(args) {
                Ok(ok) => ok,
                Err(err) => return Err(err),
            }
        }
    } else {
        match write_std(args) {
            Ok(ok) => ok,
            Err(err) => return Err(err),
        }
    }

    Ok(())
}
