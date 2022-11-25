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

fn run_asar(prog_dir: PathBuf, dir: PathBuf, args: Vec<String>) {
    let cmd = Command::new("asar.cmd")
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

fn patch(extracted_resource_dir: PathBuf, opts: HashMap<String, String>) -> std::io::Result<()> {
    let app_bundle = extracted_resource_dir.join("output").join("app-bundle.js");

    if !app_bundle.exists() || !app_bundle.is_file() {
        err("app-bundle.js not found. Please open an issue on the GitHub page.".to_string());
    }

    println!("Patching app bundle...");

    let mut app_bundle_contents =
        fs::read_to_string(&app_bundle).expect("failed to read app bundle");

    let app_bundle_patch = include_str!("fetchIntercept.js").to_string().replace(
        "/*{%account%}*/",
        if opts.contains_key("account") {
            opts.get("account").unwrap()
        } else {
            ""
        },
    );
    let insert_app_bundle_patch_at =
        "if(e.headers.get(\"Content-Type\")===\"application/json\"){try{";

    app_bundle_contents.insert_str(
        app_bundle_contents
            .find(insert_app_bundle_patch_at)
            .expect("failed to patch app bundle. WeMod may have changed their program")
            + insert_app_bundle_patch_at.len(),
        app_bundle_patch.as_str(),
    );

    fs::write(&app_bundle, app_bundle_contents)?;

    println!("Done.");

    println!("Patching vendor bundle...");

    let vendor_bundle = extracted_resource_dir
        .join("output")
        .join("vendor-bundle.js");

    if !vendor_bundle.exists() || !vendor_bundle.is_file() {
        err("vendor-bundle.js not found. Please open an issue on the GitHub page.".to_string());
    }

    let mut vendor_bundle_contents =
        fs::read_to_string(&vendor_bundle).expect("failed to read vendor bundle");
    let vendor_bundle_patch = include_str!("vendorPatch.js")
        .to_string()
        .replace("/*{%version%}*/", VERSION);

    vendor_bundle_contents.insert_str(0, &vendor_bundle_patch);

    fs::write(&vendor_bundle, vendor_bundle_contents)?;

    println!("Done.");

    let index_js = extracted_resource_dir.join("index.js");

    if !index_js.exists() || !index_js.is_file() {
        err("index.js not found. your WeMod version may not be supported.".to_string())
    }

    println!("Patching index.js...");

    let index_js_contents = fs::read_to_string(&index_js)?
        .replace("g.devMode", "process.argv.includes('-dev')")
        .replace("_.devMode", "process.argv.includes('-dev')");

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

    let (_cmds, _flags, opts) = SimpleArgs::new(env::args().collect()).parse();

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

    let asar_bin = if opts.contains_key("asar-bin") {
        let folder = PathBuf::from(opts.get("asar-bin").unwrap());

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
        asar_bin.clone(),
        resource_dir.clone(),
        vec![
            "extract".to_string(),
            "app.asar".to_string(),
            "app".to_string(),
        ],
    );

    println!("Done.");

    let extracted_resource_dir = resource_dir.join("app");

    patch(extracted_resource_dir.clone(), opts)?;

    println!("Repacking resources...");

    run_asar(
        asar_bin.clone(),
        resource_dir,
        vec![
            "pack".to_string(),
            "app".to_string(),
            "app.asar".to_string(),
        ],
    );

    println!("Done.");
    println!("Cleaning up...");

    fs::remove_dir_all(extracted_resource_dir).expect("failed to remove extracted resources.");

    println!("Done.");

    Ok(())
}
