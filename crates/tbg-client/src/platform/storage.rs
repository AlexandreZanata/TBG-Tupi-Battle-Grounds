//! Browser-compatible preferences and private, atomic native JSON persistence.

#[derive(Clone, Copy)]
pub enum Scope {
    Local,
    Session,
}

#[cfg(target_arch = "wasm32")]
fn browser(scope: Scope) -> Option<web_sys::Storage> {
    let window = web_sys::window()?;
    match scope {
        Scope::Local => window.local_storage().ok()?,
        Scope::Session => window.session_storage().ok()?,
    }
}

pub fn get(scope: Scope, key: &str) -> Option<String> {
    #[cfg(target_arch = "wasm32")]
    {
        browser(scope)?.get_item(key).ok()?
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        native::get(scope, key)
    }
}

pub fn set(scope: Scope, key: &str, value: &str) -> bool {
    #[cfg(target_arch = "wasm32")]
    {
        browser(scope).is_some_and(|s| s.set_item(key, value).is_ok())
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        native::update(scope, key, Some(value))
    }
}

pub fn remove(scope: Scope, key: &str) {
    #[cfg(target_arch = "wasm32")]
    if let Some(s) = browser(scope) {
        let _ = s.remove_item(key);
    }
    #[cfg(not(target_arch = "wasm32"))]
    native::update(scope, key, None);
}

#[cfg(not(target_arch = "wasm32"))]
pub fn data_dir() -> Option<std::path::PathBuf> {
    if let Some(path) = std::env::var_os("TBG_DATA_DIR") {
        return Some(path.into());
    }
    Some(
        directories::ProjectDirs::from("", "", "tupi-battle-grounds")?
            .data_dir()
            .to_path_buf(),
    )
}

#[cfg(not(target_arch = "wasm32"))]
mod native {
    use super::{data_dir, Scope};
    use std::{
        collections::BTreeMap,
        fs,
        io::{self, Write},
        path::Path,
        sync::{Mutex, OnceLock},
    };
    type Values = BTreeMap<String, String>;
    static SESSION: OnceLock<Mutex<Values>> = OnceLock::new();
    static LOCAL: Mutex<()> = Mutex::new(());

    pub(super) fn get(scope: Scope, key: &str) -> Option<String> {
        match scope {
            Scope::Session => SESSION
                .get_or_init(Mutex::default)
                .lock()
                .ok()?
                .get(key)
                .cloned(),
            Scope::Local => {
                let _guard = LOCAL.lock().ok()?;
                read(&data_dir()?.join("preferences.json"))
                    .get(key)
                    .cloned()
            }
        }
    }

    pub(super) fn update(scope: Scope, key: &str, value: Option<&str>) -> bool {
        match scope {
            Scope::Session => {
                let Ok(mut values) = SESSION.get_or_init(Mutex::default).lock() else {
                    return false;
                };
                change(&mut values, key, value);
                true
            }
            Scope::Local => {
                let Ok(_guard) = LOCAL.lock() else {
                    return false;
                };
                let Some(dir) = data_dir() else { return false };
                let path = dir.join("preferences.json");
                let mut values = read(&path);
                change(&mut values, key, value);
                if let Err(error) = write(&path, &values) {
                    bevy::log::warn!("Could not save preferences: {error}");
                    return false;
                }
                true
            }
        }
    }

    fn change(values: &mut Values, key: &str, value: Option<&str>) {
        if let Some(value) = value {
            values.insert(key.into(), value.into());
        } else {
            values.remove(key);
        }
    }

    fn read(path: &Path) -> Values {
        if fs::metadata(path).map_or(true, |m| m.len() > 65_536) {
            return Values::new();
        }
        fs::read(path)
            .ok()
            .and_then(|bytes| serde_json::from_slice(&bytes).ok())
            .unwrap_or_default()
    }

    fn write(path: &Path, values: &Values) -> io::Result<()> {
        let parent = path
            .parent()
            .ok_or_else(|| io::Error::other("No data directory"))?;
        fs::create_dir_all(parent)?;
        let temp = path.with_extension("json.tmp");
        let mut options = fs::OpenOptions::new();
        options.create(true).truncate(true).write(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
            fs::set_permissions(parent, fs::Permissions::from_mode(0o700))?;
            options.mode(0o600);
        }
        let mut file = options.open(&temp)?;
        file.write_all(&serde_json::to_vec(values).map_err(io::Error::other)?)?;
        file.sync_all()?;
        fs::rename(temp, path)
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        #[test]
        fn private_atomic_preferences_recover_from_corruption() {
            let dir = std::env::temp_dir().join(format!("tbg-storage-{}", std::process::id()));
            let path = dir.join("preferences.json");
            let values = Values::from([("key".into(), "credential".into())]);
            write(&path, &values).unwrap();
            assert_eq!(read(&path), values);
            assert!(!path.with_extension("json.tmp").exists());
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                assert_eq!(
                    fs::metadata(&path).unwrap().permissions().mode() & 0o777,
                    0o600
                );
            }
            fs::write(&path, "broken JSON").unwrap();
            assert!(read(&path).is_empty());
            write(&path, &values).unwrap();
            let mut deleted = read(&path);
            change(&mut deleted, "key", None);
            write(&path, &deleted).unwrap();
            assert!(read(&path).is_empty());
            fs::remove_dir_all(dir).unwrap();
        }
    }
}
