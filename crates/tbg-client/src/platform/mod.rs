//! Platform services shared by the browser and native Bevy interface.

pub mod http;

/// P66 §7: what this client reports with an admission request: its
/// installed build and the bounded outcome of the last update (the
/// launcher passes both; a direct or browser run reports its version).
pub struct UpdateReport {
    pub build: String,
    pub outcome: String,
}

/// URL-safe short slug (`[A-Za-z0-9._:-]`, at most 64 characters).
fn slug(raw: &str) -> String {
    raw.chars()
        .take(64)
        .map(|c| {
            if c.is_ascii_alphanumeric() || "._:-".contains(c) {
                c
            } else {
                '_'
            }
        })
        .collect()
}

pub fn update_report() -> UpdateReport {
    #[cfg(not(target_arch = "wasm32"))]
    let (build, outcome) = (
        std::env::var("TBG_INSTALLED_BUILD").unwrap_or_else(|_| env!("CARGO_PKG_VERSION").into()),
        std::env::var("TBG_UPDATE_OUTCOME").unwrap_or_else(|_| "none".into()),
    );
    #[cfg(target_arch = "wasm32")]
    let (build, outcome) = (
        format!("web-{}", env!("CARGO_PKG_VERSION")),
        "none".to_string(),
    );
    UpdateReport {
        build: slug(&build),
        outcome: slug(&outcome),
    }
}
#[cfg(not(target_arch = "wasm32"))]
pub mod credentials;
#[cfg(not(target_arch = "wasm32"))]
pub mod resume;
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
