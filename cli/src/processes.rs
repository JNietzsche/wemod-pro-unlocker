use std::process::Command;

const TARGET_PROCESSES: &[&str] = &[
    "Wand.exe",
    "WeMod.exe",
    "WandAuxiliaryService.exe",
    "WeModAuxiliaryService.exe",
    "GameLauncher.exe",
];

pub fn kill_wemod_processes() {
    for name in TARGET_PROCESSES {
        let _ = Command::new("taskkill")
            .args(["/F", "/T", "/IM", name])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status();
    }
}
