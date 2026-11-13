use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
    process::Command,
    thread,
    time::Duration,
};

const TARGET_PROCESSES: &[&str] = &[
    "Wand.exe",
    "WeMod.exe",
    "WandAuxiliaryService.exe",
    "WeModAuxiliaryService.exe",
    "GameLauncher.exe",
];

const GENERIC_EXE_NAMES: &[&str] = &["update.exe", "squirrel.exe", "createdump.exe"];

const MAX_KILL_ATTEMPTS: usize = 10;
const RETRY_DELAY: Duration = Duration::from_millis(200);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessInfo {
    pub pid: u32,
    pub parent_pid: u32,
    pub exe_name: String,
    pub exe_path: Option<PathBuf>,
}

pub fn kill_wemod_processes(search_dirs: &[&Path]) -> Result<(), String> {
    let discovered_exes = collect_exe_names(search_dirs);
    let mut last_error: Option<String> = None;

    for _ in 0..MAX_KILL_ATTEMPTS {
        let targets = find_matching_processes(search_dirs, &discovered_exes);
        if targets.is_empty() {
            return Ok(());
        }

        for proc_info in &targets {
            if let Err(err) = terminate_process_tree(proc_info.pid) {
                last_error = Some(err);
            }
        }

        thread::sleep(RETRY_DELAY);
    }

    let remaining = find_matching_processes(search_dirs, &discovered_exes);
    if remaining.is_empty() {
        return Ok(());
    }

    let details = remaining
        .iter()
        .map(|p| format!("{} (PID {})", p.exe_name, p.pid))
        .collect::<Vec<_>>()
        .join(", ");

    let reason = last_error
        .map(|e| format!(" ({})", e))
        .unwrap_or_default();

    Err(format!(
        "Failed to stop running WeMod/Wand process(es): {}{}. Close Wand/WeMod manually or run this tool as Administrator.",
        details, reason
    ))
}

fn find_matching_processes(
    search_dirs: &[&Path],
    discovered_exes: &HashSet<String>,
) -> Vec<ProcessInfo> {
    let current_pid = std::process::id();
    enumerate_processes()
        .into_iter()
        .filter(|proc_info| {
            proc_info.pid != current_pid
                && should_target_process(proc_info, search_dirs, discovered_exes)
        })
        .collect()
}

fn should_target_process(
    proc_info: &ProcessInfo,
    search_dirs: &[&Path],
    discovered_exes: &HashSet<String>,
) -> bool {
    let lower_name = proc_info.exe_name.to_ascii_lowercase();
    if TARGET_PROCESSES
        .iter()
        .any(|target| target.eq_ignore_ascii_case(&lower_name))
    {
        return true;
    }

    match &proc_info.exe_path {
        Some(exe_path) => search_dirs.iter().any(|dir| is_path_in_dir(exe_path, dir)),
        None => discovered_exes.contains(&lower_name),
    }
}

fn collect_exe_names(search_dirs: &[&Path]) -> HashSet<String> {
    let mut names = HashSet::new();
    for dir in search_dirs {
        collect_exe_names_in_dir(dir, 0, &mut names);
    }
    names
}

fn collect_exe_names_in_dir(dir: &Path, depth: usize, out: &mut HashSet<String>) {
    if depth > 2 {
        return;
    }
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let Ok(file_type) = entry.file_type() else {
            continue;
        };
        if file_type.is_dir() {
            collect_exe_names_in_dir(&path, depth + 1, out);
        } else if file_type.is_file() {
            let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
                continue;
            };
            let lower = name.to_ascii_lowercase();
            if lower.ends_with(".exe") && !GENERIC_EXE_NAMES.contains(&lower.as_str()) {
                out.insert(lower);
            }
        }
    }
}

fn normalize_windows_path(path: &Path) -> String {
    let raw = path.to_string_lossy();
    let stripped = raw
        .strip_prefix(r"\\?\")
        .or_else(|| raw.strip_prefix("//?/"))
        .unwrap_or(&raw);
    let mut normalized = stripped.replace('/', "\\").to_ascii_lowercase();
    while normalized.ends_with('\\') && normalized.len() > 3 {
        normalized.pop();
    }
    normalized
}

fn is_path_in_dir(file_path: &Path, dir: &Path) -> bool {
    let norm_file = normalize_windows_path(file_path);
    let norm_dir = normalize_windows_path(dir);
    if norm_dir.is_empty() {
        return false;
    }
    if norm_file == norm_dir {
        return true;
    }
    let Some(rest) = norm_file.strip_prefix(&norm_dir) else {
        return false;
    };
    norm_dir.ends_with('\\') || rest.starts_with('\\')
}

fn terminate_process_tree(pid: u32) -> Result<(), String> {
    match terminate_tree_direct(pid) {
        Ok(()) => Ok(()),
        Err(direct_err) => {
            let pid_str = pid.to_string();
            match Command::new("taskkill")
                .args(["/F", "/T", "/PID", &pid_str])
                .output()
            {
                Ok(out) if out.status.success() => Ok(()),
                Ok(out) => {
                    let taskkill_err = format_command_error(&out.stderr, &out.stdout);
                    Err(format!("{}; {}", direct_err, taskkill_err))
                }
                Err(_) => Err(direct_err),
            }
        }
    }
}

fn terminate_tree_direct(root_pid: u32) -> Result<(), String> {
    let all_procs = enumerate_processes();
    let mut to_kill = vec![root_pid];
    let mut idx = 0;
    while idx < to_kill.len() {
        let parent = to_kill[idx];
        idx += 1;
        for proc_info in &all_procs {
            if proc_info.parent_pid == parent && !to_kill.contains(&proc_info.pid) {
                to_kill.push(proc_info.pid);
            }
        }
    }

    // Terminate children before parent so child processes don't outlive/respawn.
    let mut first_err: Option<String> = None;
    for &pid in to_kill.iter().rev() {
        if let Err(err) = terminate_process_direct(pid) {
            if first_err.is_none() {
                first_err = Some(err);
            }
        }
    }

    match first_err {
        None => Ok(()),
        Some(err) => Err(err),
    }
}

fn format_command_error(stderr: &[u8], stdout: &[u8]) -> String {
    let raw = if !stderr.is_empty() {
        String::from_utf8_lossy(stderr)
    } else {
        String::from_utf8_lossy(stdout)
    };
    let collapsed = raw
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .collect::<Vec<_>>()
        .join(" ");
    if collapsed.is_empty() {
        "taskkill failed with non-zero exit code".to_string()
    } else {
        collapsed
    }
}

#[cfg(windows)]
fn enumerate_processes() -> Vec<ProcessInfo> {
    use std::ffi::OsString;
    use std::os::windows::ffi::OsStringExt;

    const TH32CS_SNAPPROCESS: u32 = 0x0000_0002;
    const INVALID_HANDLE_VALUE: isize = -1;
    const PROCESS_QUERY_LIMITED_INFORMATION: u32 = 0x1000;

    #[repr(C)]
    struct ProcessEntry32W {
        dw_size: u32,
        cnt_usage: u32,
        th32_process_id: u32,
        th32_default_heap_id: usize,
        th32_module_id: u32,
        cnt_threads: u32,
        th32_parent_process_id: u32,
        pc_pri_class_base: i32,
        dw_flags: u32,
        sz_exe_file: [u16; 260],
    }

    #[link(name = "kernel32")]
    extern "system" {
        fn CreateToolhelp32Snapshot(dw_flags: u32, th32_process_id: u32) -> isize;
        fn Process32FirstW(h_snapshot: isize, lppe: *mut ProcessEntry32W) -> i32;
        fn Process32NextW(h_snapshot: isize, lppe: *mut ProcessEntry32W) -> i32;
        fn OpenProcess(dw_desired_access: u32, b_inherit_handle: i32, dw_process_id: u32) -> isize;
        fn QueryFullProcessImageNameW(
            h_process: isize,
            dw_flags: u32,
            lp_exe_name: *mut u16,
            lpdw_size: *mut u32,
        ) -> i32;
        fn CloseHandle(h_object: isize) -> i32;
    }

    let snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) };
    if snapshot == INVALID_HANDLE_VALUE || snapshot == 0 {
        return Vec::new();
    }

    let mut result = Vec::new();
    let mut entry = ProcessEntry32W {
        dw_size: std::mem::size_of::<ProcessEntry32W>() as u32,
        cnt_usage: 0,
        th32_process_id: 0,
        th32_default_heap_id: 0,
        th32_module_id: 0,
        cnt_threads: 0,
        th32_parent_process_id: 0,
        pc_pri_class_base: 0,
        dw_flags: 0,
        sz_exe_file: [0; 260],
    };

    let mut has_entry = unsafe { Process32FirstW(snapshot, &mut entry) } != 0;
    while has_entry {
        let len = entry
            .sz_exe_file
            .iter()
            .position(|&c| c == 0)
            .unwrap_or(entry.sz_exe_file.len());
        let exe_name = OsString::from_wide(&entry.sz_exe_file[..len])
            .to_string_lossy()
            .into_owned();

        let exe_path = if entry.th32_process_id != 0 {
            let handle = unsafe {
                OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, entry.th32_process_id)
            };
            if handle != 0 {
                let mut buf = vec![0u16; 1024];
                let mut buf_len = buf.len() as u32;
                let ok = unsafe {
                    QueryFullProcessImageNameW(handle, 0, buf.as_mut_ptr(), &mut buf_len)
                };
                unsafe {
                    CloseHandle(handle);
                }
                if ok != 0 && buf_len > 0 {
                    Some(PathBuf::from(OsString::from_wide(&buf[..buf_len as usize])))
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };

        result.push(ProcessInfo {
            pid: entry.th32_process_id,
            parent_pid: entry.th32_parent_process_id,
            exe_name,
            exe_path,
        });

        has_entry = unsafe { Process32NextW(snapshot, &mut entry) } != 0;
    }

    unsafe {
        CloseHandle(snapshot);
    }

    result
}

#[cfg(not(windows))]
fn enumerate_processes() -> Vec<ProcessInfo> {
    Vec::new()
}

#[cfg(windows)]
fn terminate_process_direct(pid: u32) -> Result<(), String> {
    const PROCESS_TERMINATE: u32 = 0x0001;

    #[link(name = "kernel32")]
    extern "system" {
        fn OpenProcess(dw_desired_access: u32, b_inherit_handle: i32, dw_process_id: u32) -> isize;
        fn TerminateProcess(h_process: isize, u_exit_code: u32) -> i32;
        fn CloseHandle(h_object: isize) -> i32;
    }

    let handle = unsafe { OpenProcess(PROCESS_TERMINATE, 0, pid) };
    if handle == 0 {
        return Err(std::io::Error::last_os_error().to_string());
    }
    let ok = unsafe { TerminateProcess(handle, 1) };
    let err = if ok == 0 {
        Some(std::io::Error::last_os_error().to_string())
    } else {
        None
    };
    unsafe {
        CloseHandle(handle);
    }
    match err {
        None => Ok(()),
        Some(e) => Err(e),
    }
}

#[cfg(not(windows))]
fn terminate_process_direct(_pid: u32) -> Result<(), String> {
    Err("not supported on non-Windows".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn test_is_path_in_dir_handles_case_prefixes_and_unc() {
        let dir = Path::new(r"\\?\C:\Users\Test\AppData\Local\Wand");
        assert!(is_path_in_dir(
            Path::new(r"c:\users\test\appdata\local\wand\app-12.72.0\Wand.exe"),
            dir
        ));
        assert!(is_path_in_dir(
            Path::new(r"C:\Users\Test\AppData\Local\Wand\Update.exe"),
            dir
        ));
        // Must not match a sibling folder whose name merely starts with "Wand"
        assert!(!is_path_in_dir(
            Path::new(r"C:\Users\Test\AppData\Local\Wand2\Wand.exe"),
            dir
        ));
    }

    #[test]
    fn test_should_target_process_matches_path_known_and_discovered_exes() {
        let wand_dir = Path::new(r"C:\Users\Test\AppData\Local\Wand");
        let dirs = [wand_dir];
        let mut discovered = HashSet::new();
        discovered.insert("wandhelper.exe".to_string());

        // Matched by directory even for generic names like Update.exe
        let in_dir = ProcessInfo {
            pid: 100,
            parent_pid: 1,
            exe_name: "Update.exe".to_string(),
            exe_path: Some(PathBuf::from(r"C:\Users\Test\AppData\Local\Wand\Update.exe")),
        };
        assert!(should_target_process(&in_dir, &dirs, &discovered));

        // Unrelated Update.exe outside Wand directory must not be targeted
        let other_update = ProcessInfo {
            pid: 101,
            parent_pid: 1,
            exe_name: "Update.exe".to_string(),
            exe_path: Some(PathBuf::from(r"C:\Users\Test\AppData\Local\Discord\Update.exe")),
        };
        assert!(!should_target_process(&other_update, &dirs, &discovered));

        // Elevated target process whose full path couldn't be queried is still matched by name
        let elevated_wand = ProcessInfo {
            pid: 102,
            parent_pid: 1,
            exe_name: "wand.exe".to_string(),
            exe_path: None,
        };
        assert!(should_target_process(&elevated_wand, &dirs, &discovered));

        // Discovered non-generic exe from install dir matched when path cannot be read
        let discovered_helper = ProcessInfo {
            pid: 103,
            parent_pid: 1,
            exe_name: "WandHelper.exe".to_string(),
            exe_path: None,
        };
        assert!(should_target_process(&discovered_helper, &dirs, &discovered));

        // Same filename in an unrelated directory with a readable path must not be matched
        let unrelated_helper = ProcessInfo {
            pid: 104,
            parent_pid: 1,
            exe_name: "WandHelper.exe".to_string(),
            exe_path: Some(PathBuf::from(r"C:\OtherApp\WandHelper.exe")),
        };
        assert!(!should_target_process(&unrelated_helper, &dirs, &discovered));
    }

    #[cfg(windows)]
    #[test]
    fn test_terminate_process_tree_stops_running_child() {
        use std::process::Stdio;

        let mut child = Command::new("powershell")
            .args(["-NoProfile", "-Command", "Start-Sleep -Seconds 30"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("failed to spawn sleeping child");

        let pid = child.id();
        assert!(
            child.try_wait().unwrap().is_none(),
            "child process exited prematurely before termination test"
        );

        let result = terminate_process_tree(pid);
        let status = child.wait().expect("failed to wait on child");

        assert!(result.is_ok(), "terminate_process_tree failed: {:?}", result);
        assert!(!status.success());
    }

    #[test]
    fn test_collect_exe_names_ignores_generic_update_exe() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!("wmpu-proc-test-{unique}"));
        let app_dir = root.join("app-12.72.0");
        fs::create_dir_all(&app_dir).unwrap();
        fs::write(root.join("Update.exe"), b"").unwrap();
        fs::write(app_dir.join("Wand.exe"), b"").unwrap();
        fs::write(app_dir.join("CustomHelper.exe"), b"").unwrap();

        let names = collect_exe_names(&[&root]);
        assert!(names.contains("wand.exe"));
        assert!(names.contains("customhelper.exe"));
        assert!(!names.contains("update.exe"));

        let _ = fs::remove_dir_all(&root);
    }

    #[cfg(windows)]
    #[test]
    fn test_enumerate_processes_finds_current_process() {
        let pid = std::process::id();
        let procs = enumerate_processes();
        let current = procs.into_iter().find(|p| p.pid == pid);
        assert!(current.is_some(), "expected current PID {} in snapshot", pid);
        let current = current.unwrap();
        assert!(!current.exe_name.is_empty());
        assert!(current.exe_path.is_some());
    }
}
