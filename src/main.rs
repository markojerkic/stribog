use error_chain::error_chain;

use walkdir::WalkDir;

error_chain! {
    foreign_links {
        WalkDir(walkdir::Error);
        Io(std::io::Error);
        SystemTime(std::time::SystemTimeError);
    }
}

use walkdir::DirEntry;

fn is_hidden(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with("."))
        .unwrap_or(false)
}

fn list_dir(dir: &str, mut dir_vec: Vec<String>) -> Result<()> {
    let walker = WalkDir::new(dir);

    for entry in walker
        .into_iter()
        .filter(|entry| entry.is_ok())
        .map(|entry| entry.unwrap())
        .filter(|entry| is_hidden(entry))
    {
        let pntr = entry.path().display().to_string();
        dir_vec.push(pntr);
    }

    Ok(())
}

fn main() -> Result<()> {
    let home_walker = WalkDir::new("/")
        .follow_links(true)
        .max_depth(3)
        .follow_links(true);

    for entry in home_walker.into_iter() {
        if entry.is_ok() {
            println!("{}", entry?.path().display());
        }
    }

    Ok(())
}
