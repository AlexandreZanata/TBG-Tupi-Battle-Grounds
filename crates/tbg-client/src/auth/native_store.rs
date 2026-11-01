//! Nonsecret per-origin metadata and an OS process lock, separate from credentials.

use super::types::{valid_hex, Failure};
use serde_json::{json, Value};
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};

pub struct Store {
    pub meta: Value,
    path: PathBuf,
    _lock: fs::File,
}

pub fn nonce(bytes: usize) -> Result<String, Failure> {
    let mut raw = vec![0; bytes];
    getrandom::fill(&mut raw).map_err(|_| Failure::Unavailable)?;
    Ok(raw.iter().map(|v| format!("{v:02x}")).collect())
}

fn origin_id(origin: &str) -> String {
    // Filename only, never an authentication primitive. Verify the full origin in metadata.
    let mut hash = 0xcbf29ce484222325_u64;
    for byte in origin.bytes() {
        hash = (hash ^ u64::from(byte)).wrapping_mul(0x100000001b3);
    }
    format!("{hash:016x}")
}

impl Store {
    pub fn open(origin: &str) -> Result<Self, Failure> {
        let base = directories::BaseDirs::new().ok_or(Failure::StorageUnavailable)?;
        let locks = base
            .runtime_dir()
            .unwrap_or_else(|| base.data_local_dir())
            .join("tbg-auth-locks");
        fs::create_dir_all(&locks).map_err(|_| Failure::StorageUnavailable)?;
        let id = origin_id(origin);
        let mut options = fs::OpenOptions::new();
        options.create(true).truncate(false).read(true).write(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let lock = options
            .open(locks.join(format!("{id}.lock")))
            .map_err(|_| Failure::StorageUnavailable)?;
        lock.try_lock().map_err(|_| Failure::Busy)?;
        let path = crate::platform::storage::data_dir()
            .ok_or(Failure::StorageUnavailable)?
            .join("auth")
            .join(format!("{id}.json"));
        let meta = match fs::read(&path) {
            Ok(bytes) if bytes.len() <= 4096 => {
                serde_json::from_slice(&bytes).map_err(|_| Failure::StorageUnavailable)?
            }
            Ok(_) => return Err(Failure::StorageUnavailable),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                json!({"schema":1,"origin":origin,"device":nonce(16)?,"remember":false,"logout_pending":false})
            }
            Err(_) => return Err(Failure::StorageUnavailable),
        };
        if meta.get("origin").and_then(Value::as_str) != Some(origin)
            || meta.get("schema").and_then(Value::as_u64) != Some(1)
            || !meta
                .get("device")
                .and_then(Value::as_str)
                .is_some_and(|s| valid_hex(s, 16))
        {
            return Err(Failure::StorageUnavailable);
        }
        let store = Self {
            meta,
            path,
            _lock: lock,
        };
        store.save()?;
        Ok(store)
    }
    pub fn device(&self) -> Result<&str, Failure> {
        self.meta
            .get("device")
            .and_then(Value::as_str)
            .ok_or(Failure::StorageUnavailable)
    }
    pub fn account(&self) -> Option<&str> {
        self.meta
            .get("account")
            .and_then(Value::as_str)
            .filter(|s| tbg_core::account::valid_name_shape(s))
    }
    pub fn save(&self) -> Result<(), Failure> {
        write(&self.path, &self.meta)
    }
    /// P66 resume record: per origin and installation (device), never per
    /// account; `~` cannot appear in an account name, so it never collides.
    pub fn resume_namespace(&self, origin: &str) -> Result<String, Failure> {
        let encoded: String = origin.bytes().map(|v| format!("{v:02x}")).collect();
        Ok(format!("{encoded}:~resume:{}", self.device()?))
    }
    pub fn namespace(&self, origin: &str, account: &str) -> Result<String, Failure> {
        if !tbg_core::account::valid_name_shape(account) {
            return Err(Failure::Incompatible);
        }
        let encoded: String = origin.bytes().map(|v| format!("{v:02x}")).collect();
        Ok(format!(
            "{encoded}:{}:{}",
            account.to_ascii_lowercase(),
            self.device()?
        ))
    }
}

fn write(path: &Path, value: &Value) -> Result<(), Failure> {
    let parent = path.parent().ok_or(Failure::StorageUnavailable)?;
    fs::create_dir_all(parent).map_err(|_| Failure::StorageUnavailable)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(parent, fs::Permissions::from_mode(0o700))
            .map_err(|_| Failure::StorageUnavailable)?;
    }
    let bytes = serde_json::to_vec(value).map_err(|_| Failure::StorageUnavailable)?;
    if bytes.len() > 4096 {
        return Err(Failure::StorageUnavailable);
    }
    let temp = path.with_extension(format!("json.{}.tmp", std::process::id()));
    let mut options = fs::OpenOptions::new();
    options.write(true).create(true).truncate(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options
        .open(&temp)
        .map_err(|_| Failure::StorageUnavailable)?;
    file.write_all(&bytes)
        .and_then(|_| file.sync_all())
        .and_then(|_| fs::rename(&temp, path))
        .map_err(|_| Failure::StorageUnavailable)?;
    if fs::read(path).map_err(|_| Failure::StorageUnavailable)? != bytes {
        return Err(Failure::StorageUnavailable);
    }
    Ok(())
}
