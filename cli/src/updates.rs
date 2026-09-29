use colored::Colorize;
use std::{
    env::current_exe,
    process::{exit, Command},
};
use version_compare::Cmp;

use crate::files::extract_temp_file;

fn get_latest_release() -> Option<serde_json::Value> {
    let request_url = "https://api.github.com/repos/Allain-afk/wemod-pro-unlocker/releases/latest";
    let request = minreq::Request::new(minreq::Method::Get, request_url)
        .with_header("User-Agent", "wmpu-cli");

    match request.send() {
        Ok(response) => {
            if !(200..=299).contains(&response.status_code) {
                return None;
            }
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

            Some(json_response)
        }
        Err(err) => {
            println!("failed to check for updates: {}", err);
            None
        }
    }
}

fn update() {
    let updater = include_bytes!("../bin/wemod-pro-unlocker-updater.exe");

    match extract_temp_file("wemod-pro-unlocker-updater.exe", updater) {
        Err(err) => println!("failed to create updater: {}", err),
        Ok(file) => {
            Command::new(file.canonicalize().unwrap().to_str().unwrap())
                .arg(current_exe().unwrap())
                .spawn()
                .expect("failed to start updater");
        }
    };
}

pub fn check(flags: &Vec<String>) {
    if !(flags.contains(&"no-update".to_string()) || flags.contains(&"offline".to_string())) {
        let latest_release = get_latest_release();

        if let Some(release) = latest_release {
            if let Some(tag_name) = release["tag_name"].as_str() {
                match version_compare::compare(tag_name.replace("v", ""), crate::VERSION) {
                    Ok(result) => {
                        if result == Cmp::Gt {
                            println!(
                                "{}",
                                "UPDATE AVAILABLE: There is a new update available."
                                    .on_bright_blue()
                                    .white()
                                    .bold()
                            );
                            update();
                            exit(0);
                        }
                    }
                    Err(err) => println!("failed to check for updates: {:?}", err),
                }
            }
        }
    }
}
