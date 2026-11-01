//! Safe release extraction (P66 §5.1, U04): only regular files and
//! directories with plain relative names; no links, devices, absolute or
//! parent paths, Windows-reserved or case-colliding names; bounded entry
//! count and total size (decompression bombs). Extraction targets a fresh
//! staging directory that is never executable until it is complete.

use std::collections::HashSet;
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Component, Path, PathBuf};

use crate::limits::{ARCHIVE_ENTRIES, EXTRACTED_BYTES};

/// Files every release must contain.
pub fn required_files() -> [&'static str; 3] {
    let binary = if cfg!(windows) {
        "tbg-client.exe"
    } else {
        "tbg-client"
    };
    [binary, "build-info.json", "release-notes.json"]
}

const RESERVED: [&str; 22] = [
    "con", "prn", "aux", "nul", "com1", "com2", "com3", "com4", "com5", "com6", "com7", "com8",
    "com9", "lpt1", "lpt2", "lpt3", "lpt4", "lpt5", "lpt6", "lpt7", "lpt8", "lpt9",
];

/// A safe relative path inside the release, or the reason it is not.
pub fn safe_path(raw: &Path) -> Result<PathBuf, String> {
    let mut out = PathBuf::new();
    for part in raw.components() {
        let Component::Normal(name) = part else {
            if matches!(part, Component::CurDir) {
                continue;
            }
            return Err(format!("unsafe path component in {}", raw.display()));
        };
        let name = name.to_str().ok_or("non UTF-8 name")?;
        let stem = name.split('.').next().unwrap_or("").to_ascii_lowercase();
        let bad = name.is_empty()
            || name.len() > 200
            || name
                .chars()
                .any(|c| c.is_control() || "\\:*?\"<>|".contains(c))
            || name.ends_with('.')
            || name.ends_with(' ')
            || RESERVED.contains(&stem.as_str());
        if bad {
            return Err(format!("unsafe name {name:?}"));
        }
        out.push(name);
    }
    if out.as_os_str().is_empty() || out.components().count() > 16 {
        return Err("empty or too deep path".into());
    }
    Ok(out)
}

/// Extract `archive` (tar.gz) into the empty directory `dest`.
pub fn extract(archive: &Path, dest: &Path) -> Result<u64, String> {
    fs::create_dir_all(dest).map_err(|e| e.to_string())?;
    let file = File::open(archive).map_err(|e| e.to_string())?;
    let mut tar = tar::Archive::new(flate2::read::GzDecoder::new(file));
    let (mut total, mut count, mut seen) = (0u64, 0usize, HashSet::new());
    for entry in tar.entries().map_err(|e| e.to_string())? {
        let mut entry = entry.map_err(|e| e.to_string())?;
        count += 1;
        if count > ARCHIVE_ENTRIES {
            return Err("too many archive entries".into());
        }
        let raw = entry.path().map_err(|e| e.to_string())?.into_owned();
        // The archive's own root (`./`) names nothing to create.
        let root_dir = raw.components().all(|c| matches!(c, Component::CurDir));
        if root_dir && entry.header().entry_type() == tar::EntryType::Directory {
            continue;
        }
        let rel = safe_path(&raw)?;
        if !seen.insert(rel.to_string_lossy().to_lowercase()) {
            return Err(format!(
                "duplicate or case-colliding path {}",
                rel.display()
            ));
        }
        let target = dest.join(&rel);
        match entry.header().entry_type() {
            tar::EntryType::Directory => fs::create_dir_all(&target).map_err(|e| e.to_string())?,
            tar::EntryType::Regular => {
                let size = entry.header().size().map_err(|e| e.to_string())?;
                total = total.checked_add(size).ok_or("size overflow")?;
                if total > EXTRACTED_BYTES {
                    return Err("extracted size exceeds its limit".into());
                }
                let mode = entry.header().mode().unwrap_or(0o644);
                write_file(&mut entry, &target, size, mode & 0o100 != 0)?;
            }
            other => {
                return Err(format!(
                    "unsupported entry type {other:?} at {}",
                    rel.display()
                ))
            }
        }
    }
    for name in required_files() {
        if !dest.join(name).is_file() {
            return Err(format!("release is missing {name}"));
        }
    }
    Ok(total)
}

fn write_file(
    entry: &mut impl Read,
    target: &Path,
    size: u64,
    executable: bool,
) -> Result<(), String> {
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let mut options = fs::OpenOptions::new();
    options.create_new(true).write(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(if executable { 0o755 } else { 0o644 });
    }
    #[cfg(not(unix))]
    let _ = executable;
    let mut out = options
        .open(target)
        .map_err(|e| format!("{}: {e}", target.display()))?;
    let copied = io::copy(&mut entry.take(size), &mut out).map_err(|e| e.to_string())?;
    if copied != size {
        return Err("truncated archive entry".into());
    }
    out.flush()
        .and_then(|_| out.sync_all())
        .map_err(|e| e.to_string())
}
