use std::{env::temp_dir, fs, path::PathBuf};

pub fn extract_temp_file(name: &str, bytes: &'static [u8]) -> std::io::Result<PathBuf> {
    let file = temp_dir().join(name);

    if file.exists() && file.is_file() {
        fs::remove_file(&file).unwrap_or_default();
    } else if file.is_dir() {
        fs::remove_dir_all(&file).unwrap_or_default();
    }

    match fs::write(&file, &bytes) {
        Err(err) => Err(err),
        Ok(_) => Ok(file),
    }
}
