use crate::files::extract_temp_file;
use std::{env::current_exe, process::Command};

mod files;

fn main() {
    println!("The crates.io version of this program is currently not supported due to crates.io's max size limit.");
    println!(
        "WMPU is now being updated to the GitHub release version. Please rerun it afterwards."
    );

    match extract_temp_file(
        "wemod-pro-unlocker-updater.exe",
        include_bytes!("../bin/wemod-pro-unlocker-updater.exe"),
    ) {
        Err(err) => println!("failed to create updater: {}", err),
        Ok(file) => {
            Command::new(file.canonicalize().unwrap().to_str().unwrap())
                .arg(current_exe().unwrap())
                .spawn()
                .expect("failed to start updater");
        }
    };
}
