use crate::versions;
use std::{
    fs::{self, DirEntry},
    path::{Path, PathBuf},
};
use windirs::{known_folder_path, FolderId};

pub fn get_wemod_folder() -> PathBuf {
    let local_app_data =
        known_folder_path(FolderId::LocalAppData).expect("Local app data could not be found.");

    get_wemod_folder_in(&local_app_data)
}

fn get_wemod_folder_in(local_app_data: &Path) -> PathBuf {
    for folder_name in ["Wand", "WeMod"] {
        let candidate = local_app_data.join(folder_name);
        if has_installed_version(&candidate) {
            return candidate;
        }
    }

    local_app_data.join("WeMod")
}

fn has_installed_version(folder: &Path) -> bool {
    match fs::read_dir(folder) {
        Ok(entries) => entries.filter_map(Result::ok).any(|entry| {
            entry
                .file_type()
                .map_or(false, |file_type| file_type.is_dir())
                && entry.file_name().to_string_lossy().starts_with("app-")
        }),
        Err(_) => false,
    }
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

#[cfg(test)]
mod tests {
    use super::get_wemod_folder_in;
    use std::{
        fs,
        path::{Path, PathBuf},
        time::{SystemTime, UNIX_EPOCH},
    };

    struct TestDirectory(PathBuf);

    impl TestDirectory {
        fn new() -> Self {
            let unique = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let path = std::env::temp_dir().join(format!(
                "wemod-pro-unlocker-folder-test-{}-{unique}",
                std::process::id()
            ));
            fs::create_dir_all(&path).unwrap();
            Self(path)
        }

        fn path(&self) -> &Path {
            &self.0
        }
    }

    impl Drop for TestDirectory {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn chooses_wand_when_wand_and_wemod_are_both_installed() {
        let local_app_data = TestDirectory::new();
        fs::create_dir_all(local_app_data.path().join("Wand").join("app-12.41.1")).unwrap();
        fs::create_dir_all(local_app_data.path().join("WeMod").join("app-9.12.0")).unwrap();

        let folder = get_wemod_folder_in(local_app_data.path());

        assert_eq!(folder, local_app_data.path().join("Wand"));
    }

    #[test]
    fn falls_back_to_wemod_when_wand_has_no_installed_version() {
        let local_app_data = TestDirectory::new();
        fs::create_dir_all(local_app_data.path().join("Wand").join("logs")).unwrap();
        fs::create_dir_all(local_app_data.path().join("WeMod").join("app-9.12.0")).unwrap();

        let folder = get_wemod_folder_in(local_app_data.path());

        assert_eq!(folder, local_app_data.path().join("WeMod"));
    }
}
