use std::{fs, path::PathBuf};

const SENTINEL: &[u8] = b"dL7pKGdnNz796PbbjQWNKmHXBZaB9tsX";
const FUSE_EMBEDDED_ASAR_INTEGRITY_VALIDATION: usize = 4;
const FUSE_DISABLE: u8 = b'0';
const FUSE_ENABLE: u8 = b'1';

pub fn disable_asar_integrity(app_dir: PathBuf) {
    let candidates = ["Wand.exe", "WeMod.exe"];
    let mut exe_path: Option<PathBuf> = None;
    for name in candidates.iter() {
        let p = app_dir.join(name);
        if p.exists() {
            exe_path = Some(p);
            break;
        }
    }

    let exe_path = match exe_path {
        Some(p) => p,
        None => {
            println!("could not find Wand.exe or WeMod.exe to patch integrity fuse.");
            return;
        }
    };

    let mut data = match fs::read(&exe_path) {
        Ok(d) => d,
        Err(e) => {
            println!("failed to read electron binary: {}", e);
            return;
        }
    };

    let off = match data.windows(SENTINEL.len()).position(|w| w == SENTINEL) {
        Some(o) => o,
        None => {
            println!("electron fuse sentinel not found; skipping integrity fuse flip.");
            return;
        }
    };

    // layout: [sentinel][version:1][length:1][fuse0..fuseN]
    let fuse_base = off + SENTINEL.len() + 2;
    let target = fuse_base + FUSE_EMBEDDED_ASAR_INTEGRITY_VALIDATION;

    if target >= data.len() {
        println!("fuse index out of bounds; skipping.");
        return;
    }

    match data[target] {
        FUSE_DISABLE => return,
        FUSE_ENABLE => {
            data[target] = FUSE_DISABLE;
            if let Err(e) = fs::write(&exe_path, &data) {
                println!("failed to write patched electron binary: {}", e);
            }
        }
        other => {
            println!(
                "unexpected fuse value 0x{:02x} at offset {}; skipping.",
                other, target
            );
        }
    }
}
