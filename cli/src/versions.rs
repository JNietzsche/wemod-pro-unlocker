use std::{cmp::Ordering, fs::DirEntry, path::PathBuf};
use version_compare::{self, Cmp};

fn get_version_from_dir_entry(entry: &DirEntry) -> String {
    return entry.file_name().to_str().unwrap().replacen("app-", "", 1);
}

pub fn get_version_from_path(path: PathBuf) -> String {
    return path
        .file_name()
        .unwrap()
        .to_str()
        .unwrap()
        .replacen("app-", "", 1);
}

pub fn sort_app_versions(a: &DirEntry, b: &DirEntry) -> Ordering {
    return match version_compare::compare(
        get_version_from_dir_entry(a.clone()),
        get_version_from_dir_entry(b.clone()),
    )
    .unwrap_or(Cmp::Eq)
    {
        Cmp::Gt => Ordering::Greater,
        Cmp::Lt => Ordering::Less,
        _ => Ordering::Equal,
    };
}
