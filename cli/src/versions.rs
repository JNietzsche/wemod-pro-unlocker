use std::{cmp::Ordering, path::Path};
use version_compare::{self, Cmp};

/// Strips the `app-` prefix from a Squirrel app folder name (`app-12.41.1` -> `12.41.1`).
pub fn version_from_dir_name(name: &str) -> &str {
    name.strip_prefix("app-").unwrap_or(name)
}

pub fn get_version_from_path(path: &Path) -> String {
    path.file_name()
        .map(|name| version_from_dir_name(&name.to_string_lossy()).to_string())
        .unwrap_or_else(|| path.display().to_string())
}

/// Compares two app folder names (e.g. `app-9.9.0` and `app-10.0.0`) by their
/// semantic version rather than lexically.
pub fn compare_app_dir_names(a: &str, b: &str) -> Ordering {
    match version_compare::compare(version_from_dir_name(a), version_from_dir_name(b))
        .unwrap_or(Cmp::Eq)
    {
        Cmp::Gt => Ordering::Greater,
        Cmp::Lt => Ordering::Less,
        _ => Ordering::Equal,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn sorted(mut names: Vec<&str>) -> Vec<&str> {
        names.sort_by(|a, b| compare_app_dir_names(a, b));
        names
    }

    #[test]
    fn sorts_by_minor_and_patch_numerically() {
        assert_eq!(
            sorted(vec!["app-8.10.0", "app-8.2.0", "app-8.9.12", "app-8.9.3"]),
            vec!["app-8.2.0", "app-8.9.3", "app-8.9.12", "app-8.10.0"]
        );
    }

    #[test]
    fn sorts_two_digit_major_versions_after_single_digit() {
        assert_eq!(
            sorted(vec!["app-12.41.1", "app-9.12.0", "app-10.0.0", "app-12.5.0"]),
            vec!["app-9.12.0", "app-10.0.0", "app-12.5.0", "app-12.41.1"]
        );
    }

    #[test]
    fn compares_equal_and_ordered_versions() {
        assert_eq!(compare_app_dir_names("app-1.2.3", "app-1.2.3"), Ordering::Equal);
        assert_eq!(compare_app_dir_names("app-1.2.10", "app-1.2.9"), Ordering::Greater);
        assert_eq!(compare_app_dir_names("app-1.0.0", "app-1.0.1"), Ordering::Less);
    }

    #[test]
    fn extracts_version_from_path() {
        assert_eq!(
            get_version_from_path(&PathBuf::from(r"C:\Users\x\AppData\Local\Wand\app-12.41.1")),
            "12.41.1"
        );
        assert_eq!(version_from_dir_name("app-8.3.6"), "8.3.6");
        assert_eq!(version_from_dir_name("8.3.6"), "8.3.6");
    }
}
