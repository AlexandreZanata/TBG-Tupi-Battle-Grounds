//! Platform services shared by the browser and native Bevy interface.

pub mod http;
pub mod storage;
#[cfg(not(target_arch = "wasm32"))]
pub mod window;

#[cfg(not(target_arch = "wasm32"))]
fn clipboard() -> &'static std::sync::Mutex<Option<arboard::Clipboard>> {
    static CLIPBOARD: std::sync::OnceLock<std::sync::Mutex<Option<arboard::Clipboard>>> =
        std::sync::OnceLock::new();
    CLIPBOARD.get_or_init(|| std::sync::Mutex::new(arboard::Clipboard::new().ok()))
}

#[cfg(not(target_arch = "wasm32"))]
pub fn copy(text: &str) -> bool {
    clipboard()
        .lock()
        .ok()
        .and_then(|mut c| c.as_mut()?.set_text(text).ok())
        .is_some()
}

#[cfg(not(target_arch = "wasm32"))]
pub fn paste() -> Option<String> {
    clipboard().lock().ok()?.as_mut()?.get_text().ok()
}

/// Resolve a public page against the configured authoritative server.
pub fn page_url(path: &str) -> Option<String> {
    if path.starts_with("https://") || path.starts_with("http://") {
        return Some(path.to_string());
    }
    Some(format!(
        "{}{}",
        http::origin(&crate::config::server_url())?,
        path
    ))
}

#[cfg(not(target_arch = "wasm32"))]
pub fn open_page(path: &str) {
    if let Some(url) = page_url(path) {
        if let Err(error) = open::that(url) {
            bevy::log::warn!("Could not open browser: {error}");
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub mod credentials;

/// An explicit owner-requested backup export; never used for automatic session persistence.
#[cfg(not(target_arch = "wasm32"))]
pub fn export_recovery_key(name: &str, key: &str) -> bool {
    use std::io::Write;
    let Some(dir) = storage::data_dir() else {
        return false;
    };
    let safe: String = name
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '_')
        .collect();
    if safe.is_empty() || std::fs::create_dir_all(&dir).is_err() {
        return false;
    }
    let mut options = std::fs::OpenOptions::new();
    options.create(true).truncate(true).write(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    options
        .open(dir.join(format!("{safe}-tbg-key.txt")))
        .and_then(|mut file| file.write_all(key.as_bytes()).and_then(|_| file.sync_all()))
        .is_ok()
}
