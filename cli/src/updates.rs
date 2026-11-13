use colored::Colorize;
use std::{
    env::current_exe,
    path::{Path, PathBuf},
    process::Command,
};
use version_compare::Cmp;

use crate::files::extract_temp_file;

fn get_latest_release() -> Option<serde_json::Value> {
    let request_url = "https://api.github.com/repos/Allain-afk/wemod-pro-unlocker/releases/latest";
    let request = minreq::Request::new(minreq::Method::Get, request_url)
        .with_header("User-Agent", "wmpu-cli")
        .with_timeout(15);

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

/// Removes the Win32 extended-length prefix that `canonicalize()` adds
/// (`\\?\C:\...` -> `C:\...`, `\\?\UNC\server\share` -> `\\server\share`).
pub fn strip_verbatim_prefix(path: &Path) -> PathBuf {
    let raw = path.to_string_lossy();
    if let Some(rest) = raw.strip_prefix(r"\\?\UNC\") {
        PathBuf::from(format!(r"\\{}", rest))
    } else if let Some(rest) = raw.strip_prefix(r"\\?\") {
        PathBuf::from(rest)
    } else {
        path.to_path_buf()
    }
}

/// Extracts the embedded updater and starts it. On success the caller must
/// exit immediately so the updater can replace this executable.
fn update() -> Result<(), String> {
    let updater = include_bytes!("../bin/wemod-pro-unlocker-updater.exe");

    let updater_path = extract_temp_file("wemod-pro-unlocker-updater.exe", updater)
        .map_err(|e| format!("failed to create updater: {}", e))?;
    let updater_path = strip_verbatim_prefix(&updater_path);

    let target = current_exe().map_err(|e| format!("failed to locate current executable: {}", e))?;
    let target = strip_verbatim_prefix(&target.canonicalize().unwrap_or(target));

    Command::new(&updater_path)
        .arg(&target)
        .spawn()
        .map_err(|e| format!("failed to start updater ({}): {}", updater_path.display(), e))?;

    Ok(())
}

pub fn check(flags: &[String]) {
    if flags.iter().any(|f| f == "no-update" || f == "offline") {
        return;
    }

    let Some(release) = get_latest_release() else {
        return;
    };
    let Some(tag_name) = release["tag_name"].as_str() else {
        return;
    };

    match version_compare::compare(tag_name.trim_start_matches('v'), crate::VERSION) {
        Ok(Cmp::Gt) => {
            println!(
                "{}",
                "UPDATE AVAILABLE: There is a new update available."
                    .on_bright_blue()
                    .white()
                    .bold()
            );
            match update() {
                // Exit right away (without "Press Enter") so the updater can overwrite this binary.
                Ok(()) => crate::exit_now(0),
                Err(e) => println!(
                    "{}",
                    format!("{}. Continuing with the current version.", e).yellow()
                ),
            }
        }
        Ok(_) => {}
        Err(err) => println!("failed to check for updates: {:?}", err),
    }
}

#[cfg(test)]
mod tests {
    use super::strip_verbatim_prefix;
    use std::path::{Path, PathBuf};

    #[test]
    fn strips_extended_length_prefixes() {
        assert_eq!(
            strip_verbatim_prefix(Path::new(r"\\?\C:\Users\x\wemod-pro-unlocker.exe")),
            PathBuf::from(r"C:\Users\x\wemod-pro-unlocker.exe")
        );
        assert_eq!(
            strip_verbatim_prefix(Path::new(r"\\?\UNC\server\share\wmpu.exe")),
            PathBuf::from(r"\\server\share\wmpu.exe")
        );
        assert_eq!(
            strip_verbatim_prefix(Path::new(r"C:\plain\path.exe")),
            PathBuf::from(r"C:\plain\path.exe")
        );
    }
}
