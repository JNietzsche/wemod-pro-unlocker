use sha2::{Digest, Sha256};
use std::{
    fs,
    io::Read,
    path::{Path, PathBuf},
};

const SENTINEL: &[u8] = b"dL7pKGdnNz796PbbjQWNKmHXBZaB9tsX";
const FUSE_EMBEDDED_ASAR_INTEGRITY_VALIDATION: usize = 4;
const FUSE_DISABLE: u8 = b'0';
const FUSE_ENABLE: u8 = b'1';

pub fn compute_sha256(path: &Path) -> std::io::Result<String> {
    let mut file = fs::File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 65536];
    loop {
        let n = file.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        hasher.update(&buffer[..n]);
    }
    let result = hasher.finalize();
    Ok(result.iter().map(|b| format!("{:02x}", b)).collect())
}

pub fn update_embedded_asar_hash(app_dir: &Path, asar_path: &Path) {
    let hash = match compute_sha256(asar_path) {
        Ok(h) => h,
        Err(e) => {
            println!("failed to compute asar sha256: {}", e);
            return;
        }
    };

    let candidates = ["Wand.exe", "WeMod.exe"];
    let needle = b"\"alg\":\"SHA256\",\"value\":\"";

    for name in candidates.iter() {
        let exe_path = app_dir.join(name);
        if !exe_path.exists() {
            continue;
        }

        let mut data = match fs::read(&exe_path) {
            Ok(d) => d,
            Err(e) => {
                println!("failed to read {}: {}", name, e);
                continue;
            }
        };

        if let Some(pos) = data.windows(needle.len()).position(|w| w == needle) {
            let start = pos + needle.len();
            if start + 64 <= data.len() && (start + 64 == data.len() || data[start + 64] == b'"') {
                data[start..start + 64].copy_from_slice(hash.as_bytes());
                if let Err(e) = fs::write(&exe_path, &data) {
                    println!("failed to update embedded asar hash in {}: {}", name, e);
                } else {
                    println!("Updated embedded asar hash in {}.", name);
                }
            }
        }
    }
}

pub fn patch_auxiliary_service(resource_dir: &Path) {
    let subdirs = ["app.asar.unpacked", "app.asar.unpacked.old"];
    let names = ["WandAuxiliaryService.exe", "WeModAuxiliaryService.exe"];

    for subdir in subdirs.iter() {
        let aux_dir = resource_dir
            .join(subdir)
            .join("static")
            .join("unpacked")
            .join("auxiliary");
        if !aux_dir.exists() {
            continue;
        }

        for name in names.iter() {
            let exe_path = aux_dir.join(name);
            if !exe_path.exists() {
                continue;
            }

            let mut data = match fs::read(&exe_path) {
                Ok(d) => d,
                Err(e) => {
                    println!("failed to read {}: {}", name, e);
                    continue;
                }
            };

            // Search for W::0 IL sequence:
            // 0x02, 0x28, ??, ??, ??, 0x06, 0x0A, 0xDD, 0x08, 0x00, 0x00, 0x00, 0x26, 0x16, 0x0A, 0xDD
            let tail = [0x0A, 0xDD, 0x08, 0x00, 0x00, 0x00, 0x26, 0x16, 0x0A, 0xDD];
            let mut patched = false;

            for i in 0..data.len().saturating_sub(16) {
                if data[i] == 0x02
                    && data[i + 1] == 0x28
                    && data[i + 5] == 0x06
                    && &data[i + 6..i + 16] == &tail
                {
                    // Replace `ldarg.0; call <token>;` with `ldc.i4.1; nop; nop; nop; nop; nop;`
                    data[i] = 0x17;
                    data[i + 1] = 0x00;
                    data[i + 2] = 0x00;
                    data[i + 3] = 0x00;
                    data[i + 4] = 0x00;
                    data[i + 5] = 0x00;
                    patched = true;
                    break;
                }
            }

            if patched {
                if let Err(e) = fs::write(&exe_path, &data) {
                    println!("failed to write patched auxiliary service {}: {}", name, e);
                } else {
                    println!("Patched client integrity check in {}/{}.", subdir, name);
                }
            }
        }
    }
}

pub fn disable_asar_integrity(app_dir: PathBuf) {
    let candidates = ["Wand.exe", "WeMod.exe"];
    let mut checked_any = false;
    let mut patched_any = false;

    for name in candidates.iter() {
        let exe_path = app_dir.join(name);
        if !exe_path.exists() {
            continue;
        }
        checked_any = true;

        let mut data = match fs::read(&exe_path) {
            Ok(d) => d,
            Err(e) => {
                println!("failed to read electron binary {}: {}", name, e);
                continue;
            }
        };

        let off = match data.windows(SENTINEL.len()).position(|w| w == SENTINEL) {
            Some(o) => o,
            None => {
                // Not an Electron binary (e.g. launcher stub), skip to next candidate
                continue;
            }
        };

        // layout: [sentinel][version:1][length:1][fuse0..fuseN]
        let fuse_base = off + SENTINEL.len() + 2;
        let target = fuse_base + FUSE_EMBEDDED_ASAR_INTEGRITY_VALIDATION;

        if target >= data.len() {
            println!("fuse index out of bounds in {}; skipping.", name);
            continue;
        }

        match data[target] {
            FUSE_DISABLE => {
                patched_any = true;
            }
            FUSE_ENABLE => {
                data[target] = FUSE_DISABLE;
                if let Err(e) = fs::write(&exe_path, &data) {
                    println!("failed to write patched electron binary {}: {}", name, e);
                } else {
                    println!("Disabled asar integrity check in {}.", name);
                    patched_any = true;
                }
            }
            other => {
                println!(
                    "unexpected fuse value 0x{:02x} at offset {} in {}; skipping.",
                    other, target, name
                );
            }
        }
    }

    if !checked_any {
        println!("could not find Wand.exe or WeMod.exe to patch integrity fuse.");
    } else if !patched_any {
        println!("electron fuse sentinel not found or already disabled; skipping integrity fuse flip.");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fuse_sentinel_flip() {
        let mut mock_data = Vec::new();
        mock_data.extend_from_slice(b"HEADER_PADDING_DATA");
        mock_data.extend_from_slice(SENTINEL);
        mock_data.push(0x01); // fuse wire format version
        mock_data.push(0x10); // number of fuses
        for _ in 0..16 {
            mock_data.push(FUSE_ENABLE);
        }
        mock_data.extend_from_slice(b"FOOTER_PADDING_DATA");

        let temp_dir = std::env::temp_dir().join(format!("wmpu-fuse-test-{}", std::process::id()));
        fs::create_dir_all(&temp_dir).unwrap();
        let test_exe = temp_dir.join("Wand.exe");
        fs::write(&test_exe, &mock_data).unwrap();

        disable_asar_integrity(temp_dir.clone());

        let read_back = fs::read(&test_exe).unwrap();
        let off = read_back.windows(SENTINEL.len()).position(|w| w == SENTINEL).unwrap();
        let target = off + SENTINEL.len() + 2 + FUSE_EMBEDDED_ASAR_INTEGRITY_VALIDATION;
        assert_eq!(read_back[target], FUSE_DISABLE);

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_update_embedded_asar_hash() {
        let temp_dir = std::env::temp_dir().join(format!("wmpu-hash-test-{}", std::process::id()));
        fs::create_dir_all(&temp_dir).unwrap();

        let asar_file = temp_dir.join("app.asar");
        fs::write(&asar_file, b"sample asar data").unwrap();
        let expected_hash = compute_sha256(&asar_file).unwrap();

        let mut exe_data = Vec::new();
        exe_data.extend_from_slice(b"PREAMBLE");
        exe_data.extend_from_slice(b"\"alg\":\"SHA256\",\"value\":\"0000000000000000000000000000000000000000000000000000000000000000\"");
        exe_data.extend_from_slice(b"POSTAMBLE");

        let exe_file = temp_dir.join("Wand.exe");
        fs::write(&exe_file, &exe_data).unwrap();

        update_embedded_asar_hash(&temp_dir, &asar_file);

        let read_exe = fs::read(&exe_file).unwrap();
        let needle = b"\"alg\":\"SHA256\",\"value\":\"";
        let pos = read_exe.windows(needle.len()).position(|w| w == needle).unwrap();
        let hash_start = pos + needle.len();
        let updated_hash = std::str::from_utf8(&read_exe[hash_start..hash_start + 64]).unwrap();

        assert_eq!(updated_hash, expected_hash);

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_patch_auxiliary_service() {
        let temp_dir = std::env::temp_dir().join(format!("wmpu-aux-test-{}", std::process::id()));
        let aux_dir = temp_dir.join("app.asar.unpacked").join("static").join("unpacked").join("auxiliary");
        fs::create_dir_all(&aux_dir).unwrap();

        let mut exe_data = Vec::new();
        exe_data.extend_from_slice(&[0x90, 0x90]);
        // Pattern: 0x02, 0x28, 0xC4, 0x00, 0x00, 0x06, 0x0A, 0xDD, 0x08, 0x00, 0x00, 0x00, 0x26, 0x16, 0x0A, 0xDD
        exe_data.extend_from_slice(&[
            0x02, 0x28, 0xC4, 0x00, 0x00, 0x06, 0x0A, 0xDD, 0x08, 0x00, 0x00, 0x00, 0x26, 0x16, 0x0A, 0xDD,
        ]);
        exe_data.extend_from_slice(&[0x06, 0x2A]);

        let aux_exe = aux_dir.join("WandAuxiliaryService.exe");
        fs::write(&aux_exe, &exe_data).unwrap();

        patch_auxiliary_service(&temp_dir);

        let read_aux = fs::read(&aux_exe).unwrap();
        // Check that first 6 bytes became [0x17, 0x00, 0x00, 0x00, 0x00, 0x00]
        assert_eq!(&read_aux[2..8], &[0x17, 0x00, 0x00, 0x00, 0x00, 0x00]);

        let _ = fs::remove_dir_all(&temp_dir);
    }
}



