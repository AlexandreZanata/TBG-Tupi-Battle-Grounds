//! Durable file primitives (P66 §5.1): a document is written to a private
//! temp file, flushed, renamed over the target and the directory entry is
//! flushed, so power loss leaves either the old or the new bytes, never a
//! torn file. Reads are size-bounded; local state is untrusted input.

use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::Path;

/// Owner-only file mode on Unix.
pub fn private_options() -> OpenOptions {
    let mut options = OpenOptions::new();
    options.create(true).truncate(true).write(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    options
}

/// Create `dir` (and parents) owner-only.
pub fn private_dir(dir: &Path) -> io::Result<()> {
    fs::create_dir_all(dir)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(dir, fs::Permissions::from_mode(0o700))?;
    }
    Ok(())
}

/// Flush a directory entry (rename/create durability). A no-op where the
/// platform cannot open directories (Windows flushes on file close).
pub fn sync_dir(dir: &Path) -> io::Result<()> {
    #[cfg(unix)]
    File::open(dir)?.sync_all()?;
    #[cfg(not(unix))]
    let _ = dir;
    Ok(())
}

/// Atomically replace `path` with `bytes`.
pub fn write_atomic(path: &Path, bytes: &[u8]) -> io::Result<()> {
    let dir = path
        .parent()
        .ok_or_else(|| io::Error::other("no parent directory"))?;
    fs::create_dir_all(dir)?;
    let name = path
        .file_name()
        .ok_or_else(|| io::Error::other("no file name"))?;
    let temp = dir.join(format!(".{}.tmp", name.to_string_lossy()));
    {
        let mut file = private_options().open(&temp)?;
        file.write_all(bytes)?;
        file.sync_all()?;
    }
    fs::rename(&temp, path)?;
    sync_dir(dir)
}

/// Read at most `limit` bytes; a larger file is an error, not a truncation.
pub fn read_bounded(path: &Path, limit: u64) -> io::Result<Vec<u8>> {
    let file = File::open(path)?;
    if file.metadata()?.len() > limit {
        return Err(io::Error::other("file exceeds its size limit"));
    }
    let mut out = Vec::new();
    file.take(limit + 1).read_to_end(&mut out)?;
    if out.len() as u64 > limit {
        return Err(io::Error::other("file exceeds its size limit"));
    }
    Ok(out)
}

/// Bounded JSON read into `T`.
pub fn read_json<T: serde::de::DeserializeOwned>(path: &Path, limit: u64) -> io::Result<T> {
    let bytes = read_bounded(path, limit)?;
    serde_json::from_slice(&bytes).map_err(io::Error::other)
}

/// Atomic JSON write.
pub fn write_json<T: serde::Serialize>(path: &Path, value: &T) -> io::Result<()> {
    let bytes = serde_json::to_vec_pretty(value).map_err(io::Error::other)?;
    write_atomic(path, &bytes)
}

/// Lowercase hex of a byte slice.
pub fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
