use std::{path::PathBuf, process::Command};

use crate::files::extract_temp_file;

pub fn run(dir: PathBuf, args: Vec<String>) {
    let asar = match extract_temp_file("asar.exe", include_bytes!("../bin/asar.exe")) {
        Ok(f) => f,
        Err(err) => {
            crate::err(format!("failed to extract asar: {}", err).to_string());
            return;
        }
    };

    let cmd = Command::new(asar).current_dir(dir).args(args).spawn();

    if cmd.is_err() {
        match cmd.unwrap_err().kind() {
            std::io::ErrorKind::NotFound => crate::err("failed to extract asar".to_string()),
            _ => crate::err("failed run asar command".to_string()),
        }
    } else {
        cmd.unwrap().wait().unwrap();
    }
}
