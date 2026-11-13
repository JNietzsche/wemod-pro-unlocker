use crate::versions;
use std::{
    fs,
    path::{Path, PathBuf},
};
use windirs::{known_folder_path, FolderId};

pub fn get_wemod_folder() -> Result<PathBuf, String> {
    let local_app_data = known_folder_path(FolderId::LocalAppData)
        .map_err(|e| format!("Local app data folder could not be found: {:?}", e))?;

    Ok(get_wemod_folder_in(&local_app_data))
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

/// An `app-*` folder counts as an installed version only if it contains the app
/// bundle (`resources/app.asar`, or the `app.asar.old` backup left by a previous
/// patch). Stale or partially removed `app-*` folders are ignored.
fn is_valid_app_dir(dir: &Path) -> bool {
    let resources = dir.join("resources");
    resources.join("app.asar").is_file() || resources.join("app.asar.old").is_file()
}

fn list_app_dirs(folder: &Path) -> std::io::Result<Vec<PathBuf>> {
    Ok(fs::read_dir(folder)?
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().map_or(false, |t| t.is_dir()))
        .filter(|entry| entry.file_name().to_string_lossy().starts_with("app-"))
        .map(|entry| entry.path())
        .collect())
}

fn has_installed_version(folder: &Path) -> bool {
    list_app_dirs(folder)
        .map(|dirs| dirs.iter().any(|dir| is_valid_app_dir(dir)))
        .unwrap_or(false)
}

pub fn get_latest_app_dir(wemod_dir: &Path) -> Result<PathBuf, String> {
    let all = list_app_dirs(wemod_dir)
        .map_err(|e| format!("Failed to read {}: {}", wemod_dir.display(), e))?;

    if all.is_empty() {
        return Err(format!(
            "No WeMod version folders (app-*) found in {}",
            wemod_dir.display()
        ));
    }

    let mut valid: Vec<PathBuf> = all.into_iter().filter(|dir| is_valid_app_dir(dir)).collect();

    valid.sort_by(|a, b| {
        versions::compare_app_dir_names(
            &a.file_name().unwrap_or_default().to_string_lossy(),
            &b.file_name().unwrap_or_default().to_string_lossy(),
        )
    });

    valid.pop().ok_or_else(|| {
        format!(
            "No complete WeMod installation found in {} (app-* folders are missing resources\\app.asar). Try reinstalling Wand/WeMod",
            wemod_dir.display()
        )
    })
}

#[cfg(test)]
mod tests {
    use super::{get_latest_app_dir, get_wemod_folder_in};
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

    /// Creates a complete app folder containing `resources/app.asar`.
    fn make_app(root: &Path, name: &str) -> PathBuf {
        let dir = root.join(name);
        fs::create_dir_all(dir.join("resources")).unwrap();
        fs::write(dir.join("resources").join("app.asar"), b"asar").unwrap();
        dir
    }

    #[test]
    fn chooses_wand_when_wand_and_wemod_are_both_installed() {
        let local_app_data = TestDirectory::new();
        make_app(&local_app_data.path().join("Wand"), "app-12.41.1");
        make_app(&local_app_data.path().join("WeMod"), "app-9.12.0");

        let folder = get_wemod_folder_in(local_app_data.path());

        assert_eq!(folder, local_app_data.path().join("Wand"));
    }

    #[test]
    fn falls_back_to_wemod_when_wand_has_no_installed_version() {
        let local_app_data = TestDirectory::new();
        fs::create_dir_all(local_app_data.path().join("Wand").join("logs")).unwrap();
        make_app(&local_app_data.path().join("WeMod"), "app-9.12.0");

        let folder = get_wemod_folder_in(local_app_data.path());

        assert_eq!(folder, local_app_data.path().join("WeMod"));
    }

    #[test]
    fn ignores_wand_with_only_empty_app_folders() {
        let local_app_data = TestDirectory::new();
        fs::create_dir_all(local_app_data.path().join("Wand").join("app-12.41.1")).unwrap();
        make_app(&local_app_data.path().join("WeMod"), "app-9.12.0");

        let folder = get_wemod_folder_in(local_app_data.path());

        assert_eq!(folder, local_app_data.path().join("WeMod"));
    }

    #[test]
    fn latest_app_dir_picks_highest_complete_version() {
        let root = TestDirectory::new();
        make_app(root.path(), "app-9.12.0");
        let expected = make_app(root.path(), "app-12.5.0");
        make_app(root.path(), "app-10.0.0");
        // Newer but incomplete (left over from an interrupted update)
        fs::create_dir_all(root.path().join("app-12.41.1").join("resources")).unwrap();
        // Unrelated folders and files are ignored
        fs::create_dir_all(root.path().join("packages")).unwrap();
        fs::write(root.path().join("app-99.0.0"), b"not a dir").unwrap();

        assert_eq!(get_latest_app_dir(root.path()), Ok(expected));
    }

    #[test]
    fn latest_app_dir_accepts_previously_patched_backup() {
        let root = TestDirectory::new();
        let dir = root.path().join("app-12.41.1");
        fs::create_dir_all(dir.join("resources")).unwrap();
        fs::write(dir.join("resources").join("app.asar.old"), b"asar").unwrap();

        assert_eq!(get_latest_app_dir(root.path()), Ok(dir));
    }

    #[test]
    fn latest_app_dir_errors_when_folder_missing() {
        let root = TestDirectory::new();
        let missing = root.path().join("does-not-exist");

        let err = get_latest_app_dir(&missing).unwrap_err();
        assert!(err.contains("Failed to read"), "{err}");
    }

    #[test]
    fn latest_app_dir_errors_when_no_app_folders() {
        let root = TestDirectory::new();
        fs::create_dir_all(root.path().join("packages")).unwrap();

        let err = get_latest_app_dir(root.path()).unwrap_err();
        assert!(err.contains("No WeMod version folders"), "{err}");
    }

    #[test]
    fn latest_app_dir_errors_when_all_app_folders_incomplete() {
        let root = TestDirectory::new();
        fs::create_dir_all(root.path().join("app-12.41.1")).unwrap();
        fs::create_dir_all(root.path().join("app-12.40.0").join("resources")).unwrap();

        let err = get_latest_app_dir(root.path()).unwrap_err();
        assert!(err.contains("No complete WeMod installation"), "{err}");
    }
}
