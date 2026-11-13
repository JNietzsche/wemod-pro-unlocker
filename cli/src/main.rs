use colored::Colorize;
use simpleargs::SimpleArgs;
use std::{
    collections::HashMap,
    env, fs,
    io::Write,
    path::PathBuf,
    process::exit,
    sync::atomic::{AtomicU8, Ordering},
};

use crate::patches::patch_index_js;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

mod asar;
mod files;
mod folders;
mod fuse;
mod patches;
mod processes;
mod updates;
mod versions;

const USAGE: &str = "\
Usage: wemod-pro-unlocker.exe [OPTIONS]

Options:
  --wemod-dir <path>         Wand/WeMod install directory
                             (default: %LOCALAPPDATA%\\Wand, then %LOCALAPPDATA%\\WeMod)
  --wemod-version <version>  Version to patch, e.g. 12.41.1 (default: latest installed)
  --account <json>           Custom account metadata override
  --creator                  Enable creator mode
  --no-update, --offline     Skip checking for unlocker updates
  --pause                    Always wait for Enter before exiting
  --no-pause                 Never wait for Enter before exiting (for scripts)
  -v, --version              Print the unlocker version and exit
  -h, --help                 Print this help and exit";

/// Bare `--flag` switches that take no value. `simpleargs` treats every `--name`
/// as an option that consumes the following argument, so these are handled
/// separately and removed before the remaining args are parsed.
const BARE_SWITCHES: &[&str] = &["--help", "--pause", "--no-pause", "--version"];

/// Boolean flags that may be written as `--flag` or `-flag`. The double-dash
/// form is rewritten to `-flag` so `simpleargs` sees it as a flag instead of an
/// option that swallows the next argument.
const BOOLEAN_FLAGS: &[&str] = &["creator", "no-update", "offline"];

const PAUSE_AUTO: u8 = 0;
const PAUSE_ALWAYS: u8 = 1;
const PAUSE_NEVER: u8 = 2;

static PAUSE_MODE: AtomicU8 = AtomicU8::new(PAUSE_AUTO);

#[derive(Debug, Default, PartialEq, Eq)]
struct CliSwitches {
    help: bool,
    version: bool,
    pause: bool,
    no_pause: bool,
}

/// Splits out the bare switches (`--help`, `--pause`, ...) and returns the
/// remaining arguments for `simpleargs`. Single-dash forms (`-h`, `-pause`, ...)
/// are recognised too.
fn split_switches(args: Vec<String>) -> (CliSwitches, Vec<String>) {
    let mut switches = CliSwitches::default();
    let mut rest = Vec::with_capacity(args.len());

    for (i, arg) in args.into_iter().enumerate() {
        if i == 0 {
            rest.push(arg);
            continue;
        }
        let name = arg.trim_start_matches('-').trim_start_matches('/');
        let is_dash = arg.starts_with('-') || arg.starts_with('/');
        if arg.starts_with("--") && BOOLEAN_FLAGS.contains(&name) {
            rest.push(format!("-{}", name));
            continue;
        }
        let is_bare_double = BARE_SWITCHES.contains(&arg.as_str());
        let is_single = is_dash && !arg.starts_with("--");

        if is_bare_double || is_single {
            match name {
                "h" | "help" | "?" => switches.help = true,
                "v" | "version" => switches.version = true,
                "pause" => switches.pause = true,
                "no-pause" => switches.no_pause = true,
                _ => {
                    rest.push(arg);
                    continue;
                }
            }
            continue;
        }
        rest.push(arg);
    }

    (switches, rest)
}

/// Returns the number of processes attached to this console, or `None` if it
/// cannot be determined. A count of 1 means this process owns a console window
/// that was created for it (e.g. it was double-clicked in Explorer) and the
/// window will close as soon as the process exits.
#[cfg(windows)]
fn console_process_count() -> Option<u32> {
    #[link(name = "kernel32")]
    extern "system" {
        fn GetConsoleProcessList(lp_process_list: *mut u32, dw_process_count: u32) -> u32;
    }

    let mut pids = [0u32; 4];
    let count = unsafe { GetConsoleProcessList(pids.as_mut_ptr(), pids.len() as u32) };
    if count == 0 {
        None
    } else {
        Some(count)
    }
}

#[cfg(not(windows))]
fn console_process_count() -> Option<u32> {
    None
}

fn should_pause(mode: u8, console_processes: Option<u32>) -> bool {
    match mode {
        PAUSE_ALWAYS => true,
        PAUSE_NEVER => false,
        _ => console_processes == Some(1),
    }
}

/// Keeps a freshly spawned console window open so the user can read the output.
/// Does nothing when running inside an existing terminal (unless `--pause` was
/// passed) or when `--no-pause` was passed.
pub fn pause_before_exit() {
    let _ = std::io::stdout().flush();
    if !should_pause(PAUSE_MODE.load(Ordering::Relaxed), console_process_count()) {
        return;
    }
    print!("\nPress Enter to exit...");
    let _ = std::io::stdout().flush();
    let _ = std::io::stdin().read_line(&mut String::new());
}

/// Exits immediately without pausing. Used when another process (the updater)
/// needs this executable to terminate right away.
pub fn exit_now(code: i32) -> ! {
    let _ = std::io::stdout().flush();
    exit(code)
}

fn patch(
    extracted_resource_dir: PathBuf,
    opts: &HashMap<String, String>,
    creator: bool,
) -> std::io::Result<()> {
    println!("Enabling pro...");

    patches::patch_pro_mode(extracted_resource_dir.clone(), &opts);

    println!("Done.");

    if creator {
        println!("Enabling creator mode...");
        patches::patch_creator_mode(extracted_resource_dir.clone());
        println!("Done.");
    }

    println!("Patching vendor bundle...");

    patches::patch_vendor_bundle(extracted_resource_dir.clone());

    println!("Done.");

    println!("Patching index.js...");

    patch_index_js(extracted_resource_dir.clone());

    println!("Done.");

    Ok(())
}

pub fn err(msg: String) {
    fail(msg)
}

/// Prints an error, keeps the console open if needed, and exits with code 1.
pub fn fail(msg: String) -> ! {
    println!("{}", msg.red());
    pause_before_exit();
    exit(1)
}

fn main() {
    let (switches, args) = split_switches(env::args().collect());

    let mode = if switches.no_pause {
        PAUSE_NEVER
    } else if switches.pause {
        PAUSE_ALWAYS
    } else {
        PAUSE_AUTO
    };
    PAUSE_MODE.store(mode, Ordering::Relaxed);

    // Keep the window open for unexpected panics too (release builds abort on panic).
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        default_hook(info);
        pause_before_exit();
    }));

    if switches.help {
        println!("WeMod Pro Unlocker v{}\n\n{}", VERSION, USAGE);
        pause_before_exit();
        return;
    }

    if switches.version {
        println!("{}", VERSION);
        pause_before_exit();
        return;
    }

    if let Err(e) = run(args) {
        fail(format!("Error: {}", e));
    }

    pause_before_exit();
}

fn run(args: Vec<String>) -> std::io::Result<()> {
    if env::consts::OS != "windows" {
        fail(format!("Your OS ({}) is not supported.", env::consts::OS))
    }

    let (_cmds, flags, opts) = SimpleArgs::new(args).parse();

    println!("WeMod Pro Unlocker v{}", VERSION);

    updates::check(&flags);

    let wemod_folder = match opts.get("wemod-dir") {
        Some(dir) => PathBuf::from(dir),
        None => folders::get_wemod_folder().unwrap_or_else(|e| fail(e)),
    };

    if !wemod_folder.exists() {
        fail(
            "WeMod is not installed/not found (specify custom install location with --wemod-dir)."
                .to_string(),
        );
    }

    let wemod_version_folder = if let Some(version) = opts.get("wemod-version") {
        let version = version.trim().trim_start_matches("app-");
        if version.is_empty() || version.contains(['/', '\\']) || version.contains("..") {
            fail(format!("Invalid --wemod-version value: {:?}", version));
        }
        let folder = wemod_folder.join(format!("app-{}", version));

        if !folder.is_dir() {
            fail(format!(
                "WeMod version specified does not exist: {}",
                folder.display()
            ))
        }

        folder
    } else {
        match folders::get_latest_app_dir(&wemod_folder) {
            Ok(dir) => dir,
            Err(e) => fail(format!(
                "{}. You can manually specify a version with --wemod-version.",
                e
            )),
        }
    };

    let resource_dir = wemod_version_folder.join("resources");
    let wemod_version_folder_for_fuse = wemod_version_folder.clone();

    println!(
        "Attempting to patch WeMod v{}...",
        versions::get_version_from_path(&wemod_version_folder)
    );

    println!("Stopping running WeMod/Wand processes...");
    if let Err(reason) =
        processes::kill_wemod_processes(&[&wemod_folder, &wemod_version_folder_for_fuse])
    {
        fail(reason);
    }
    println!("Done.");

    println!("Extracting resources...");

    let unpacked_dir = resource_dir.join("app.asar.unpacked");
    let unpacked_old = resource_dir.join("app.asar.unpacked.old");

    if resource_dir.join("app.asar.old").exists() {
        fs::copy(
            resource_dir.join("app.asar.old"),
            resource_dir.join("app.asar"),
        )?;
        if unpacked_old.exists() {
            let _ = fs::remove_dir_all(&unpacked_dir);
            let _ = files::copy_dir_all(&unpacked_old, &unpacked_dir);
        }
    } else {
        fs::copy(
            resource_dir.join("app.asar"),
            resource_dir.join("app.asar.old"),
        )?;
        if unpacked_dir.exists() && !unpacked_old.exists() {
            let _ = files::copy_dir_all(&unpacked_dir, &unpacked_old);
        }
    }

    asar::run(
        resource_dir.clone(),
        vec![
            "extract".to_string(),
            "app.asar".to_string(),
            "app".to_string(),
        ],
    );

    println!("Done.");

    let extracted_resource_dir = resource_dir.join("app");

    // Copy original unpacked files into the extracted `app` directory so that
    // when `asar pack ... --unpack-dir static/unpacked` runs, the unpacked binaries
    // (such as WeModAuxiliaryService.exe and TrainerLib) exist in app/static/unpacked
    // and are repacked properly into app.asar.unpacked instead of being wiped out!
    if unpacked_dir.exists() {
        let _ = files::copy_dir_all(&unpacked_dir, &extracted_resource_dir);
    } else if unpacked_old.exists() {
        let _ = files::copy_dir_all(&unpacked_old, &extracted_resource_dir);
    } else {
        println!(
            "{}",
            "Notice: app.asar.unpacked not found. If game mod loading fails, reinstall WeMod from wand.com/wemod.com to restore auxiliary service binaries."
                .yellow()
        );
    }

    let creator = flags.iter().any(|f| f == "creator") || opts.contains_key("creator");
    patch(extracted_resource_dir.clone(), &opts, creator)?;

    println!("Repacking resources...");

    asar::run(
        resource_dir.clone(),
        vec![
            "pack".to_string(),
            "app".to_string(),
            "app.asar".to_string(),
            "--unpack-dir".to_string(),
            "static/unpacked".to_string(),
        ],
    );

    println!("Done.");

    // Preserve any original unpacked files if missing after pack
    if unpacked_old.exists() && unpacked_dir.exists() {
        let _ = files::copy_dir_all(&unpacked_old, &unpacked_dir);
    }

    println!("Updating embedded asar integrity hash...");
    fuse::update_embedded_asar_hash(&wemod_version_folder_for_fuse, &resource_dir.join("app.asar"));
    println!("Done.");

    println!("Patching auxiliary service integrity checks...");
    fuse::patch_auxiliary_service(&resource_dir);
    println!("Done.");

    println!("Disabling asar integrity check...");
    fuse::disable_asar_integrity(wemod_version_folder_for_fuse);
    println!("Done.");

    println!("Cleaning up...");

    match fs::remove_dir_all(&extracted_resource_dir) {
        Ok(()) => println!("Done."),
        Err(e) => println!(
            "{}",
            format!(
                "Warning: failed to remove temporary folder {} ({}). Patching succeeded; you can delete it manually.",
                extracted_resource_dir.display(),
                e
            )
            .yellow()
        ),
    }

    println!("{}", "WeMod was patched successfully.".green());

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(list: &[&str]) -> Vec<String> {
        std::iter::once("wemod-pro-unlocker.exe")
            .chain(list.iter().copied())
            .map(String::from)
            .collect()
    }

    #[test]
    fn split_switches_extracts_bare_flags_without_eating_options() {
        let (sw, rest) = split_switches(args(&["--pause", "--wemod-dir", "D:\\Wand", "-offline"]));
        assert!(sw.pause);
        assert!(!sw.no_pause && !sw.help && !sw.version);

        let (_, flags, opts) = SimpleArgs::new(rest).parse();
        assert_eq!(opts.get("wemod-dir").map(String::as_str), Some("D:\\Wand"));
        assert!(flags.contains(&"offline".to_string()));
    }

    #[test]
    fn split_switches_recognises_help_version_and_no_pause() {
        let (sw, rest) = split_switches(args(&["-h", "--no-pause", "-v"]));
        assert_eq!(
            sw,
            CliSwitches {
                help: true,
                version: true,
                pause: false,
                no_pause: true
            }
        );
        assert_eq!(rest, args(&[]));

        let (sw, _) = split_switches(args(&["--help", "--version"]));
        assert!(sw.help && sw.version);
    }

    #[test]
    fn split_switches_keeps_unrelated_args() {
        let (sw, rest) = split_switches(args(&["-creator", "--wemod-version", "12.41.1"]));
        assert_eq!(sw, CliSwitches::default());
        assert_eq!(rest, args(&["-creator", "--wemod-version", "12.41.1"]));
    }

    #[test]
    fn double_dash_boolean_flags_do_not_swallow_next_arg() {
        let (_, rest) = split_switches(args(&["--creator", "--offline", "--wemod-dir", r"D:\Wand"]));
        let (_, flags, opts) = SimpleArgs::new(rest).parse();
        assert!(flags.contains(&"creator".to_string()));
        assert!(flags.contains(&"offline".to_string()));
        assert_eq!(opts.get("wemod-dir").map(String::as_str), Some(r"D:\Wand"));
    }

    #[test]
    fn pauses_only_for_owned_console_unless_forced() {
        assert!(should_pause(PAUSE_AUTO, Some(1)));
        assert!(!should_pause(PAUSE_AUTO, Some(2)));
        assert!(!should_pause(PAUSE_AUTO, None));
        assert!(should_pause(PAUSE_ALWAYS, Some(3)));
        assert!(should_pause(PAUSE_ALWAYS, None));
        assert!(!should_pause(PAUSE_NEVER, Some(1)));
    }
}
