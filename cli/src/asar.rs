use std::{path::PathBuf, process::Command};

use crate::files::extract_temp_file;

pub fn run(dir: PathBuf, args: Vec<String>) {
    let action = args.first().cloned().unwrap_or_default();

    let asar = match extract_temp_file("asar.exe", include_bytes!("../bin/asar.exe")) {
        Ok(f) => f,
        Err(err) => crate::fail(format!("failed to extract asar: {}", err)),
    };

    let status = match Command::new(&asar).current_dir(&dir).args(&args).status() {
        Ok(status) => status,
        Err(err) => crate::fail(format!(
            "failed to run asar {} ({}): {}",
            action,
            asar.display(),
            err
        )),
    };

    if !status.success() {
        let code = status
            .code()
            .map(|c| c.to_string())
            .unwrap_or_else(|| "unknown".to_string());
        crate::fail(format!(
            "asar {} failed with exit code {} in {}. Make sure Wand/WeMod is closed and the files are not in use, then try again.",
            action,
            code,
            dir.display()
        ));
    }
}
