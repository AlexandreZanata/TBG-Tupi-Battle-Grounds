//! Private bounded atomic-file recovery journal, independent from Ranked settlement.

use crate::store::player_history::Envelope;
use std::{
    fs,
    io::{self, Read, Write},
    path::{Path, PathBuf},
};
pub const FILE_CAP: u64 = 32768;
pub const DIR_CAP: u64 = 32 * 1024 * 1024;

pub fn directory(db: &Path, slot: Option<&str>) -> PathBuf {
    db.parent()
        .unwrap_or(Path::new("."))
        .join(format!("history-pending-{}", slot.unwrap_or("single")))
}
fn name(e: &Envelope) -> String {
    use sha2::{Digest, Sha256};
    format!(
        "{}.json",
        crate::ranked::hex(&Sha256::digest(e.key.as_bytes()))
    )
}
fn sync(dir: &Path) -> io::Result<()> {
    #[cfg(unix)]
    fs::File::open(dir)?.sync_all()?;
    Ok(())
}
pub fn write(dir: &Path, e: &Envelope) -> io::Result<()> {
    fs::create_dir_all(dir)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(dir, fs::Permissions::from_mode(0o700))?;
    }
    let bytes = serde_json::to_vec(e)?;
    if bytes.len() as u64 > FILE_CAP {
        return Err(io::Error::other("history envelope exceeds 32 KiB"));
    }
    let used = fs::read_dir(dir)?.try_fold(0u64, |sum, entry| -> io::Result<_> {
        Ok(sum.saturating_add(entry?.metadata()?.len()))
    })?;
    if used.saturating_add(bytes.len() as u64) > DIR_CAP {
        return Err(io::Error::other("history journal exceeds 32 MiB"));
    }
    let path = dir.join(name(e));
    if path.exists() {
        let old = read(&path)?;
        if old == *e {
            return Ok(());
        }
        return Err(io::Error::other("conflicting history journal key"));
    }
    let temp = dir.join(format!(".tmp-{}", name(e)));
    let mut options = fs::OpenOptions::new();
    options.write(true).create(true).truncate(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options.open(&temp)?;
    file.write_all(&bytes)?;
    file.sync_all()?;
    fs::rename(temp, path)?;
    sync(dir)
}
pub fn remove(dir: &Path, e: &Envelope) -> io::Result<()> {
    fs::remove_file(dir.join(name(e)))?;
    sync(dir)
}
pub fn quarantine(dir: &Path, e: &Envelope) -> io::Result<()> {
    let path = dir.join(name(e));
    fs::rename(&path, path.with_extension("quarantine"))?;
    sync(dir)
}
fn read(path: &Path) -> io::Result<Envelope> {
    let f = fs::File::open(path)?;
    if f.metadata()?.len() > FILE_CAP {
        return Err(io::Error::other("oversized history journal"));
    }
    let mut bytes = Vec::new();
    f.take(FILE_CAP + 1).read_to_end(&mut bytes)?;
    if bytes.len() as u64 > FILE_CAP {
        return Err(io::Error::other("oversized history journal"));
    }
    serde_json::from_slice(&bytes).map_err(io::Error::other)
}
pub fn load(dir: &Path) -> io::Result<Vec<Envelope>> {
    let entries = match fs::read_dir(dir) {
        Ok(v) => v,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => return Err(e),
    };
    let mut result = Vec::new();
    let mut used = 0u64;
    for entry in entries {
        let entry = entry?;
        used = used.saturating_add(entry.metadata()?.len());
        if used > DIR_CAP || result.len() >= 1024 {
            return Err(io::Error::other("history recovery bound exceeded"));
        }
        if entry.path().extension().is_some_and(|e| e == "json") {
            result.push(read(&entry.path())?);
        }
    }
    Ok(result)
}
