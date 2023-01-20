use colored::Colorize;
use simpleargs::SimpleArgs;
use std::{
    cmp::Ordering,
    collections::HashMap,
    env,
    fs::{self, DirEntry},
    io::ErrorKind::NotFound,
    path::PathBuf,
    process::{exit, Command},
};
use version_compare::{compare as compare_versions, Cmp};
use windirs::{known_folder_path, FolderId};

const VERSION: &str = env!("CARGO_PKG_VERSION");
const IS_BETA_CHANNEL: bool = false;

fn get_wemod_folder() -> PathBuf {
    let local_app_data =
        known_folder_path(FolderId::LocalAppData).expect("Local app data could not be found.");

    return local_app_data.join("WeMod");
}

fn get_version_from_dir_entry(entry: &DirEntry) -> String {
    return entry.file_name().to_str().unwrap().replacen("app-", "", 1);
}

fn get_version_from_path(path: PathBuf) -> String {
    return path
        .file_name()
        .unwrap()
        .to_str()
        .unwrap()
        .replacen("app-", "", 1);
}

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

fn sort_app_versions(a: &DirEntry, b: &DirEntry) -> Ordering {
    return match compare_versions(
        get_version_from_dir_entry(a.clone()),
        get_version_from_dir_entry(b.clone()),
    )
    .unwrap_or(Cmp::Eq)
    {
        Cmp::Gt => Ordering::Greater,
        Cmp::Lt => Ordering::Less,
        _ => Ordering::Equal,
    };
}

fn run_asar(prog_dir: PathBuf, dir: PathBuf, args: Vec<String>, opts: &HashMap<String, String>) {
    let cmd = Command::new(opts.get("asar-bin").unwrap_or(&"asar.cmd".to_string()))
        .env(
            "PATH",
            get_asar_dirs().join(";")
                + ";"
                + env::var("PATH").unwrap().as_str()
                + ";"
                + prog_dir.to_str().unwrap(),
        )
        .current_dir(dir)
        .args(args)
        .spawn();

    if cmd.is_err() {
        if let NotFound = cmd.unwrap_err().kind() {
            err(
                "asar is not installed. you can install it using 'npm i -g @electron/asar'"
                    .to_string(),
            )
        } else {
            err("failed run asar command".to_string())
        }
    } else {
        cmd.unwrap().wait().unwrap();
    }
}

fn find_app_bundle_file(extracted_resource_dir: PathBuf) -> Option<PathBuf> {
    let ls_result = fs::read_dir(extracted_resource_dir);

    if ls_result.is_err() {
        err(format!(
            "failed to find app bundle file: {}",
            ls_result.unwrap_err()
        ));
        return None;
    }

    for file_result in ls_result.unwrap() {
        if file_result.is_err() {
            println!(
                "error while finding app bundle file: {}",
                file_result.unwrap_err()
            );
            continue;
        }
        let file = file_result.unwrap();
        let file_name_os = &file.file_name();
        let file_name = file_name_os.to_str().unwrap();

        if !(file_name.starts_with("app-") && file_name.ends_with(".js")) {
            continue;
        }

        let contents_result = fs::read_to_string(&file.path());

        if contents_result.is_err() {
            println!(
                "error while reading possible app bundle file: {}",
                contents_result.unwrap_err()
            );
            continue;
        }

        if contents_result
            .unwrap()
            .contains(r#""application/json"===e.headers.get("Content-Type")"#)
        {
            return Some(file.path());
        }
    }

    None
}

fn patch_vendor_bundle(extracted_resource_dir: PathBuf) -> Option<PathBuf> {
    let ls_result = fs::read_dir(extracted_resource_dir);

    if ls_result.is_err() {
        err(format!(
            "failed to find vendor bundle file: {}",
            ls_result.unwrap_err()
        ));
        return None;
    }

    for file_result in ls_result.unwrap() {
        if file_result.is_err() {
            println!(
                "error while finding vendor bundle file: {}",
                file_result.unwrap_err()
            );
            continue;
        }
        let file = file_result.unwrap();
        let file_name_os = &file.file_name();
        let file_name = file_name_os.to_str().unwrap();

        if !(file_name.starts_with("vendors-") && file_name.ends_with(".js")) {
            continue;
        }

        let contents_result = fs::read_to_string(&file.path());

        if contents_result.is_err() {
            println!(
                "error while reading possible vendor bundle file: {}",
                contents_result.unwrap_err()
            );
            continue;
        }

        let mut contents = contents_result.unwrap();

        let vendor_bundle_patch = include_str!("vendorPatch.js")
            .to_string()
            .replace("/*{%version%}*/", VERSION);

        contents.insert_str(0, &vendor_bundle_patch);

        let write_result = fs::write(file.path(), contents);

        if write_result.is_err() {
            println!(
                "error while writing vendor bundle file: {}",
                write_result.unwrap_err()
            );
            continue;
        }

        break;
    }

    None
}

fn get_latest_app_dir(wemod_dir: PathBuf) -> std::io::Result<PathBuf> {
    let mut versions = fs::read_dir(wemod_dir)?
        .map(|result| result.expect("failed to get wemod folder content"))
        .filter(|entry| {
            entry.metadata().expect("failed to get metadata").is_dir()
                && entry
                    .file_name()
                    .to_str()
                    .expect("failed to get folder name")
                    .starts_with("app-")
        })
        .collect::<Vec<DirEntry>>();

    versions.sort_by(|a, b| sort_app_versions(a, b));

    Ok(versions
        .last()
        .expect(
            "failed to find latest WeMod version. you can manually specify it with --wemod-version",
        )
        .path())
}

fn patch(extracted_resource_dir: PathBuf, opts: &HashMap<String, String>) -> std::io::Result<()> {
    let app_bundle_option = find_app_bundle_file(extracted_resource_dir.clone());

    if app_bundle_option.is_none() {
        err("no app bundle file found.".to_string());
    }

    let app_bundle = PathBuf::from(app_bundle_option.unwrap());

    if !app_bundle.exists() || !app_bundle.is_file() {
        err("app bundle file not found. Please open an issue on the GitHub page.".to_string());
    }

    println!("Patching app bundle...");

    let app_bundle_contents = fs::read_to_string(&app_bundle).expect("failed to read app bundle");

    let app_bundle_patch = include_str!("fetchIntercept.js").to_string().replace(
        "/*{%account%}*/",
        if opts.contains_key("account") {
            opts.get("account").unwrap()
        } else {
            ""
        },
    );
    let app_bundle_original_code =
        "return\"application/json\"===e.headers.get(\"Content-Type\")?await e.json():await e.text()";

    if !app_bundle_contents.contains(app_bundle_original_code) {
        err("failed to patch app bundle. WeMod may have changed their program".to_string());
    }

    let app_bundle_contents_patched =
        app_bundle_contents.replace(app_bundle_original_code, app_bundle_patch.as_str());

    fs::write(&app_bundle, app_bundle_contents_patched)?;

    println!("Done.");

    println!("Patching vendor bundle...");

    patch_vendor_bundle(extracted_resource_dir.clone());

    println!("Done.");

    let index_js = extracted_resource_dir.join("index.js");

    if !index_js.exists() || !index_js.is_file() {
        err("index.js not found. your WeMod version may not be supported.".to_string())
    }

    println!("Patching index.js...");

    let index_js_contents =
        fs::read_to_string(&index_js)?.replace("d.devMode", "process.argv.includes('-dev')");

    fs::write(index_js, index_js_contents)?;

    println!("Done.");

    Ok(())
}

fn err(msg: String) {
    println!("{}", msg.red());
    exit(1);
}

fn main() -> std::io::Result<()> {
    if env::consts::OS != "windows" {
        err(format!("Your OS ({}) is not supported.", env::consts::OS))
    }

    let latest_release_result = gh_updater::ReleaseFinderConfig::new("wmpu-cli")
        .with_repository("wemod-pro-unlocker")
        .with_author("bennett-sh")
        .find_release();

    if latest_release_result.is_err() {
        println!("failed to check for updates");
    } else {
        let latest_release = latest_release_result.unwrap();
        let latest_in_current_channel_result = match IS_BETA_CHANNEL {
            true => &latest_release.1,
            false => &latest_release.0,
        };

        if latest_in_current_channel_result.is_some() {
            let latest_in_current_channel = latest_in_current_channel_result.as_ref().unwrap();
            let tag_name = latest_in_current_channel.get_release_tag();

            if compare_versions(tag_name.replace("v", ""), VERSION).unwrap_or(Cmp::Eq) == Cmp::Gt {
                println!("{}\n{}", "UPDATE AVAILABLE: There is a new update available, which you are advised to install to ensure compatibility with further WeMod updates.".on_bright_blue().white().bold(), "You can download it via cargo or from the GitHub page.".white())
            }
        }
    }

    println!("WeMod Pro Unlocker v{}", VERSION);
    println!("If the patcher does not work anymore, please make sure to update it to the latest version.");

    let (_cmds, flags, opts) = SimpleArgs::new(env::args().collect()).parse();

    if flags.contains(&"v".to_string()) {
        println!("{}", VERSION);
        exit(0);
    }

    let wemod_folder = if opts.contains_key("wemod-dir") {
        PathBuf::from(opts.get("wemod-dir").unwrap())
    } else {
        get_wemod_folder()
    };

    if !wemod_folder.exists() {
        err(
            "WeMod is not installed/not found (specify custom install location with --wemod-dir)."
                .to_string(),
        );
    }

    let wemod_version_folder = if opts.contains_key("wemod-version") {
        let folder = wemod_folder.join(PathBuf::from(
            "app-".to_string()
                + PathBuf::from(opts.get("wemod-version").unwrap())
                    .file_name()
                    .unwrap()
                    .to_str()
                    .unwrap(),
        ));

        if !folder.exists() {
            err("WeMod version specified does not exist.".to_string())
        }

        wemod_folder.join(folder)
    } else {
        get_latest_app_dir(wemod_folder).expect(
            "failed to find latest WeMod version. you can manually specify it with --wemod-version",
        )
    };

    let resource_dir = wemod_version_folder.join("resources");

    let asar_folder = if opts.contains_key("asar") {
        let folder = PathBuf::from(opts.get("asar").unwrap());

        if !folder.exists() {
            err("asar path specified does not exist.".to_string());
        }

        folder
    } else {
        PathBuf::from(".")
    };

    println!(
        "Attempting to patch WeMod version {}...",
        get_version_from_path(wemod_version_folder)
    );

    println!("Extracting resources...");

    if resource_dir.join("app.asar.old").exists() {
        fs::copy(
            resource_dir.join("app.asar.old"),
            resource_dir.join("app.asar"),
        )?;
    } else {
        fs::copy(
            resource_dir.join("app.asar"),
            resource_dir.join("app.asar.old"),
        )?;
    }

    run_asar(
        asar_folder.clone(),
        resource_dir.clone(),
        vec![
            "extract".to_string(),
            "app.asar".to_string(),
            "app".to_string(),
        ],
        &opts,
    );

    println!("Done.");

    let extracted_resource_dir = resource_dir.join("app");

    patch(extracted_resource_dir.clone(), &opts)?;

    println!("Repacking resources...");

    run_asar(
        asar_folder.clone(),
        resource_dir,
        vec![
            "pack".to_string(),
            "app".to_string(),
            "app.asar".to_string(),
        ],
        &opts,
    );

    println!("Done.");
    println!("Cleaning up...");

    fs::remove_dir_all(extracted_resource_dir).expect("failed to remove extracted resources.");

    println!("Done.");

    Ok(())
}
