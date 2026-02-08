use std::collections::HashMap;
use std::fs;
use std::fs::DirEntry;
use std::io;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

pub fn walk_dir(root: &Path, cb: &mut dyn FnMut(&DirEntry)) -> io::Result<()> {
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();

        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            if name.starts_with("._") || name == ".DS_Store" {
                continue;
            }
        }

        cb(&entry);

        if path.is_dir() {
            walk_dir(&path, cb)?;
        }
    }
    Ok(())
}

#[derive(Debug)]
pub struct FileMeta {
    pub is_dir: bool,
    pub size: u64,
    pub modified: SystemTime,
}

pub fn sync(source: &Path, destination: &Path) -> io::Result<()> {
    let source = source.canonicalize()?;
    let destination = destination.canonicalize()?;

    let mut source_hash: HashMap<PathBuf, FileMeta> = HashMap::new();
    let mut destination_hash: HashMap<PathBuf, FileMeta> = HashMap::new();

    walk_dir(&source, &mut |entry| {
        let relative = match entry.path().strip_prefix(&source) {
            Ok(p) => p.to_path_buf(),
            Err(_) => return,
        };

        let meta = match entry.metadata() {
            Ok(m) => m,
            Err(_) => return,
        };

        source_hash.insert(
            relative,
            FileMeta {
                is_dir: meta.is_dir(),
                size: meta.len(),
                modified: meta.modified().unwrap_or(SystemTime::UNIX_EPOCH),
            },
        );
    })?;

    walk_dir(&destination, &mut |entry| {
        let relative = match entry.path().strip_prefix(&destination) {
            Ok(p) => p.to_path_buf(),
            Err(_) => return,
        };

        let meta = match entry.metadata() {
            Ok(m) => m,
            Err(_) => return,
        };

        destination_hash.insert(
            relative,
            FileMeta {
                is_dir: meta.is_dir(),
                size: meta.len(),
                modified: meta.modified().unwrap_or(SystemTime::UNIX_EPOCH),
            },
        );
    })?;

    for (path, src_meta) in &source_hash {
        if !destination_hash.contains_key(path) {
            if src_meta.is_dir {
                fs::create_dir_all(destination.join(path))?;
                println!("CREATED DIR:  {:?}", path);
            } else {
                let src_file = source.join(path);
                let dst_file = destination.join(path);

                if let Some(parent) = dst_file.parent() {
                    fs::create_dir_all(parent)?;
                }

                fs::copy(src_file, dst_file)?;
                println!("COPIED FILE:  {:?}", path);
            }
        }
    }

    for (path, meta) in &destination_hash {
        if !source_hash.contains_key(path) && !meta.is_dir {
            fs::remove_file(destination.join(path))?;
            println!("DELETED FILE: {:?}", path);
        }
    }

    let mut dirs_to_delete: Vec<&PathBuf> = destination_hash
        .iter()
        .filter(|(p, m)| !source_hash.contains_key(*p) && m.is_dir)
        .map(|(p, _)| p)
        .collect();

    dirs_to_delete.sort_by(|a, b| b.components().count().cmp(&a.components().count()));

    for dir in dirs_to_delete {
        fs::remove_dir(destination.join(dir))?;
        println!("DELETED DIR:  {:?}", dir);
    }

    println!(" Sync completed successfully");
    Ok(())
}
