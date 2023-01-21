use std::{
    env::{current_exe, temp_dir},
    fs,
    process::Command,
};

pub fn get_latest_release() -> Option<serde_json::Value> {
    let request_url = "https://api.github.com/repos/bennett-sh/wemod-pro-unlocker/releases/latest";
    let request = minreq::Request::new(minreq::Method::Get, request_url)
        .with_header("User-Agent", "wmpu-cli");

    match request.send() {
        Ok(response) => {
            let text_response = match response.as_str() {
                Ok(text) => text,
                Err(err) => {
                    println!("failed to check for updates: {}", err);
                    return None;
                }
            };
            let json_response: serde_json::Value = match serde_json::from_str(text_response) {
                Ok(json) => json,
                Err(err) => {
                    println!("failed to check for updates: {}", err);
                    return None;
                }
            };

            return Some(json_response);
        }
        Err(err) => println!("failed to check for updates: {}", err),
    }

    None
}

pub fn update() {
    let updater = include_bytes!("../bin/wemod-pro-unlocker-updater.exe");
    let updater_file = temp_dir().join("wemod-pro-unlocker-updater.exe");

    if updater_file.exists() && updater_file.is_file() {
        fs::remove_file(&updater_file).unwrap_or_default();
    } else if updater_file.is_dir() {
        fs::remove_dir_all(&updater_file).unwrap_or_default();
    }

    match fs::write(&updater_file, &updater) {
        Err(err) => println!("failed to create updater: {}", err),
        Ok(_) => {
            Command::new(&updater_file.canonicalize().unwrap().to_str().unwrap())
                .arg(current_exe().unwrap())
                .spawn()
                .expect("failed to start updater");
        }
    };
}
