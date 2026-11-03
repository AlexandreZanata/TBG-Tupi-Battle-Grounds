//! Only explicitly verified manual backups live in process memory; Android has no durable vault.

use std::{
    collections::BTreeMap,
    sync::{LazyLock, Mutex},
};

static RECOVERY: LazyLock<Mutex<BTreeMap<String, String>>> =
    LazyLock::new(|| Mutex::new(BTreeMap::new()));
const BACKUP_SUFFIX: &str = ":recovery-key";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    Missing,
    Unavailable,
    Invalid,
}

pub fn read(namespace: &str) -> Result<String, Error> {
    if !namespace.ends_with(BACKUP_SUFFIX) {
        return Err(Error::Missing);
    }
    RECOVERY
        .lock()
        .map_err(|_| Error::Unavailable)?
        .get(namespace)
        .cloned()
        .ok_or(Error::Missing)
}
pub fn read_within(namespace: &str, _limit: std::time::Duration) -> Result<String, Error> {
    read(namespace)
}
pub fn write(namespace: &str, value: &str) -> Result<(), Error> {
    // The auth coordinator verifies ownership before requesting this manual-key backup.
    // Refresh/access/resume credentials cannot acquire a persistence capability here.
    if !namespace.ends_with(BACKUP_SUFFIX) {
        return Err(Error::Unavailable);
    }
    if tbg_core::account::parse_key(value).is_none() {
        return Err(Error::Invalid);
    }
    RECOVERY
        .lock()
        .map_err(|_| Error::Unavailable)?
        .insert(namespace.to_owned(), value.to_owned());
    Ok(())
}
pub fn remove(namespace: &str) -> Result<(), Error> {
    let mut keys = RECOVERY.lock().map_err(|_| Error::Unavailable)?;
    keys.remove(namespace);
    keys.remove(&format!("{namespace}{BACKUP_SUFFIX}"));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn br_an08_manual_backup_is_process_only_and_logout_removes_it() {
        let base = "http://127.0.0.1:18069:BackupCase:synthetic-device";
        let namespace = format!("{base}{BACKUP_SUFFIX}");
        let key = tbg_core::account::format_key(
            "BackupCase",
            &tbg_core::account::encode_secret(&[7; 10]),
        );
        assert_eq!(read(&namespace), Err(Error::Missing));
        write(&namespace, &key).unwrap();
        assert_eq!(read(&namespace), Ok(key));
        remove(base).unwrap();
        assert_eq!(read(&namespace), Err(Error::Missing));
        assert_eq!(
            write(base, "synthetic-refresh-token"),
            Err(Error::Unavailable)
        );
        assert_eq!(write(&namespace, "invalid"), Err(Error::Invalid));
    }
}
