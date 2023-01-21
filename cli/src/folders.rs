use crate::versions;
use std::{
    fs::{self, DirEntry},
    path::PathBuf,
};
use windirs::{known_folder_path, FolderId};

pub fn get_wemod_folder() -> PathBuf {
    let local_app_data =
        known_folder_path(FolderId::LocalAppData).expect("Local app data could not be found.");

    return local_app_data.join("WeMod");
}

pub fn get_latest_app_dir(wemod_dir: PathBuf) -> std::io::Result<PathBuf> {
    let mut versions = fs::read_dir(wemod_dir)?
        .map(|result| result.expect("failed to get wemod folder content"))
        .filter(|entry| {
            entry.metadata().expect("failed to get metadata").is_dir()
                && entry
                    .file_name()
                    .to_str()
                    .expect("failed to get folder name")
                    .starts_with("app-")
        })
        .collect::<Vec<DirEntry>>();

    versions.sort_by(|a, b| versions::sort_app_versions(a, b));

    Ok(versions
        .last()
        .expect(
            "failed to find latest WeMod version. you can manually specify it with --wemod-version",
        )
        .path())
}
