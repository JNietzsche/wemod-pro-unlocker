use std::{collections::HashMap, env::var, path::PathBuf, process::Command};
use windirs::{known_folder_path, FolderId};

fn get_asar_dirs() -> Vec<String> {
    let mut result = vec![];

    result.push(
        known_folder_path(FolderId::ProgramFiles)
            .unwrap()
            .join("nodejs"),
    );
    result.push(
        known_folder_path(FolderId::ProgramFiles)
            .unwrap()
            .join("nodejs")
            .join("node_modules")
            .join(".bin"),
    );
    result.push(
        known_folder_path(FolderId::RoamingAppData)
            .unwrap()
            .join("npm"),
    );

    result
        .iter_mut()
        .map(|path| path.to_str().unwrap().to_string())
        .collect::<Vec<String>>()
}

pub fn run(prog_dir: PathBuf, dir: PathBuf, args: Vec<String>, opts: &HashMap<String, String>) {
    let cmd = Command::new(opts.get("asar-bin").unwrap_or(&"asar.cmd".to_string()))
        .env(
            "PATH",
            get_asar_dirs().join(";")
                + ";"
                + var("PATH").unwrap().as_str()
                + ";"
                + prog_dir.to_str().unwrap(),
        )
        .current_dir(dir)
        .args(args)
        .spawn();

    if cmd.is_err() {
        match cmd.unwrap_err().kind() {
            std::io::ErrorKind::NotFound => crate::err(
                "asar is not installed. you can install it using 'npm i -g @electron/asar'"
                    .to_string(),
            ),
            _ => crate::err("failed run asar command".to_string()),
        }
    } else {
        cmd.unwrap().wait().unwrap();
    }
}
