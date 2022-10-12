use error_chain::error_chain;

use walkdir::WalkDir;

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

fn walk_dir(dir: String, forbidden: Vec<String>) -> Result<()> {
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
                &forbidden,
            )
        })
        .map(|entry| entry.path().display().to_string());

    for dir in dirs {
        println!("{}", dir);
    }

    Ok(())
}

fn main() -> Result<()> {
    let mut forbidden: Vec<String> = Vec::new();
    forbidden.push(".".to_owned());
    forbidden.push("mnt".to_owned());
    let res = walk_dir("/".to_owned(), forbidden);
    res
}
