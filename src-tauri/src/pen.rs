//! tiptoi® Stift-Erkennung und Dateisystem-Analyse unter Linux.
//! Portiert von der Python-Referenzimplementierung (pen.py):
//! Scannt Mountpoints, liest `.tiptoi.log` und `.gme`-Header aus.

use serde::Serialize;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Serialize, Clone, Debug)]
pub struct InstalledProduct {
    pub filename: String,
    pub path: String,
    pub gme_id: u32,
    pub version: String,
    pub size_bytes: u64,
}

#[derive(Serialize, Clone, Debug)]
pub struct PenInfo {
    pub mount_path: String,
    pub serial_number: String,
    pub firmware_descriptor: String,
    pub language: String,
    pub mcu: String,
    pub generation: String,
    pub friendly_generation: String,
    pub installed_products: Vec<InstalledProduct>,
    pub total_space: u64,
    pub free_space: u64,
    pub used_space: u64,
}

/// Liest GME-ID (Offset 20, u32 LE) und Version (Offset 82, 8 Byte ASCII) aus einer `.gme`-Datei.
fn parse_gme_header(path: &Path) -> std::io::Result<(u32, String)> {
    let mut f = fs::File::open(path)?;
    let mut header = [0u8; 128];
    let read = f.read(&mut header)?;
    if read < 90 {
        return Ok((0, String::new()));
    }
    let gme_id = u32::from_le_bytes([header[20], header[21], header[22], header[23]]);
    let version = String::from_utf8_lossy(&header[82..90])
        .trim_matches(['\0', ' '])
        .to_string();
    Ok((gme_id, version))
}

/// Parst `.tiptoi.log`: durch 0x00/0xFF getrennte ASCII-Wörter (Seriennummer, Firmware, Sprache, MCU).
struct LogInfo {
    serial_number: String,
    firmware_descriptor: String,
    language: String,
    mcu: String,
    generation: String,
}

fn parse_log_file(log_path: &Path) -> std::io::Result<LogInfo> {
    let raw = fs::read(log_path)?;
    let mut words: Vec<String> = Vec::new();
    let mut cur: Vec<u8> = Vec::new();

    for &b in &raw {
        if b == 0x00 || b == 0xFF {
            if !cur.is_empty() {
                words.push(String::from_utf8_lossy(&cur).trim().to_string());
                cur.clear();
            }
        } else if (32..=126).contains(&b) {
            cur.push(b);
        }
    }
    if !cur.is_empty() {
        words.push(String::from_utf8_lossy(&cur).trim().to_string());
    }

    let serial = words.first().cloned().unwrap_or_else(|| "Unknown".into());
    let firmware = words.get(1).cloned().unwrap_or_else(|| "Unknown".into());
    let language = words.get(2).cloned().unwrap_or_else(|| "GERMAN".into());
    let mcu = words.get(3).cloned().unwrap_or_default();

    let fw_upper = firmware.to_uppercase();
    let generation = if firmware.starts_with("v00") || words.len() <= 2 {
        "REV1"
    } else if fw_upper.contains("3202MT") {
        "REV3"
    } else if fw_upper.contains("3202") {
        "REV2"
    } else if mcu.contains("3205") || fw_upper.contains("7GE") {
        "REV14"
    } else if fw_upper.contains("4N") {
        "REV7"
    } else if fw_upper.contains("4E") {
        "REV8"
    } else if fw_upper.contains("EMMC") {
        "REV6"
    } else if fw_upper.contains("2G") {
        "REV5"
    } else {
        "REV4"
    }
    .to_string();

    Ok(LogInfo {
        serial_number: serial,
        firmware_descriptor: firmware,
        language,
        mcu,
        generation,
    })
}

fn friendly_generation(generation: &str, firmware: &str, mcu: &str) -> String {
    let fw = firmware.to_uppercase();
    if generation.contains("REV1") || fw.starts_with("V00") {
        "Generation 1 (ohne Player)".into()
    } else if generation.contains("REV2") || generation.contains("REV3") || fw.contains("3202") {
        "Generation 2 (mit Player)".into()
    } else if mcu.contains("3205") || generation.contains("REV14") || fw.contains("WLAN") || fw.contains("7GE")
    {
        "Generation 4 (tiptoi® WLAN)".into()
    } else if mcu.contains("3203") || generation.contains("REV4") || generation.contains("REV5") {
        "Generation 3 / 4".into()
    } else {
        format!("tiptoi® ({generation})")
    }
}

fn scan_installed_products(mount_path: &Path) -> Vec<InstalledProduct> {
    let mut products = Vec::new();
    if !mount_path.exists() {
        return products;
    }

    for entry in walkdir(mount_path) {
        let Some(name) = entry.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        if !name.to_lowercase().ends_with(".gme") || name.starts_with("._") {
            continue;
        }
        let Ok(metadata) = fs::metadata(&entry) else {
            continue;
        };
        let (gme_id, version) = parse_gme_header(&entry).unwrap_or((0, String::new()));
        products.push(InstalledProduct {
            filename: name.to_string(),
            path: entry.to_string_lossy().to_string(),
            gme_id,
            version,
            size_bytes: metadata.len(),
        });
    }
    products
}

/// Simple rekursive Verzeichnis-Traversierung (kein Abhängigkeits-Crate nötig).
fn walkdir(root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let Ok(read) = fs::read_dir(&dir) else {
            continue;
        };
        for entry in read.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else {
                out.push(path);
            }
        }
    }
    out
}

fn build_pen_info(mount_path: &Path, log: LogInfo) -> PenInfo {
    let (total_space, free_space) = statvfs_space(mount_path);
    let installed_products = scan_installed_products(mount_path);
    let friendly = friendly_generation(&log.generation, &log.firmware_descriptor, &log.mcu);

    PenInfo {
        mount_path: mount_path.to_string_lossy().to_string(),
        serial_number: log.serial_number,
        firmware_descriptor: log.firmware_descriptor,
        language: log.language,
        mcu: log.mcu,
        generation: log.generation,
        friendly_generation: friendly,
        installed_products,
        total_space,
        free_space,
        used_space: total_space.saturating_sub(free_space),
    }
}

#[cfg(unix)]
fn statvfs_space(path: &Path) -> (u64, u64) {
    use std::ffi::CString;
    use std::mem::MaybeUninit;

    let Ok(c_path) = CString::new(path.as_os_str().as_encoded_bytes()) else {
        return (0, 0);
    };

    unsafe {
        let mut stat: MaybeUninit<libc::statvfs> = MaybeUninit::uninit();
        if libc::statvfs(c_path.as_ptr(), stat.as_mut_ptr()) != 0 {
            return (0, 0);
        }
        let stat = stat.assume_init();
        let total = stat.f_blocks as u64 * stat.f_frsize as u64;
        let free = stat.f_bavail as u64 * stat.f_frsize as u64;
        (total, free)
    }
}

/// Liefert Kandidaten-Mountpunkte (USB-Laufwerke) analog zur Python-Implementierung.
fn candidate_mount_points() -> Vec<PathBuf> {
    let mut mounts = Vec::new();
    let user = std::env::var("USER").unwrap_or_else(|_| "user".into());

    for root in [
        format!("/run/media/{user}"),
        format!("/media/{user}"),
        "/media".to_string(),
        "/mnt".to_string(),
    ] {
        let root_path = Path::new(&root);
        if let Ok(entries) = fs::read_dir(root_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() && !mounts.contains(&path) {
                    mounts.push(path);
                }
            }
        }
    }

    if let Ok(contents) = fs::read_to_string("/proc/mounts") {
        for line in contents.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 3 {
                let mount_point = parts[1];
                let fstype = parts[2];
                if matches!(fstype, "vfat" | "msdos" | "exfat" | "fuse")
                    && !mount_point.starts_with("/boot")
                {
                    let path = PathBuf::from(mount_point);
                    if !mounts.contains(&path) {
                        mounts.push(path);
                    }
                }
            }
        }
    }

    mounts
}

/// Sucht nach einem angeschlossenen tiptoi-Stift über alle Kandidaten-Mountpunkte.
pub fn find_connected_pen(explicit_path: Option<&str>) -> Option<PenInfo> {
    let candidates: Vec<PathBuf> = match explicit_path {
        Some(p) => vec![PathBuf::from(p)],
        None => candidate_mount_points(),
    };

    for cand in candidates {
        if !cand.exists() {
            continue;
        }

        let log_file = cand.join(".tiptoi.log");
        if log_file.exists() {
            if let Ok(log) = parse_log_file(&log_file) {
                return Some(build_pen_info(&cand, log));
            }
        }

        // Fallback: Verzeichnis enthält .gme-Dateien, aber kein Log → generischer Stift.
        if let Ok(entries) = fs::read_dir(&cand) {
            let has_gme = entries.flatten().any(|e| {
                e.file_name()
                    .to_str()
                    .map(|n| n.to_lowercase().ends_with(".gme"))
                    .unwrap_or(false)
            });
            if has_gme {
                return Some(build_pen_info(
                    &cand,
                    LogInfo {
                        serial_number: "TIPTOI_PEN".into(),
                        firmware_descriptor: "REV4".into(),
                        language: "GERMAN".into(),
                        mcu: String::new(),
                        generation: "REV4".into(),
                    },
                ));
            }
        }
    }

    None
}

/// Hängt den Stift sicher aus (udisksctl → gio → umount als Fallback).
pub fn eject_pen(mount_path: &str) -> bool {
    let _ = Command::new("sync").status();

    if Command::new("udisksctl")
        .args(["unmount", "-p", mount_path])
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
    {
        return true;
    }

    if Command::new("gio")
        .args(["mount", "-u", mount_path])
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
    {
        return true;
    }

    Command::new("umount")
        .arg(mount_path)
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}
