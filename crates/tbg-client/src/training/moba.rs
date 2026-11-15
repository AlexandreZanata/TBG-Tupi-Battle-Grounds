//! Opens the optional 5v5 Practice companion from the desktop training panel.

use crate::i18n::Lang;

#[cfg(not(target_arch = "wasm32"))]
fn companion() -> Option<std::path::PathBuf> {
    let binary = format!("tbg-moba{}", std::env::consts::EXE_SUFFIX);
    std::env::var_os("TBG_MOBA_CLIENT")
        .map(std::path::PathBuf::from)
        .filter(|path| path.is_absolute() && path.is_file())
        .or_else(|| {
            let executable = std::env::current_exe().ok()?;
            let path = executable.with_file_name(binary);
            path.is_file().then_some(path)
        })
}

#[cfg(not(target_arch = "wasm32"))]
pub(super) fn available() -> bool {
    companion().is_some()
}

#[cfg(target_arch = "wasm32")]
pub(super) fn available() -> bool {
    crate::config::url_param("practice5v5").as_deref() == Some("1")
}

#[cfg(not(target_arch = "wasm32"))]
pub(super) fn launch(lang: Lang) -> bool {
    let Some(path) = companion() else {
        return false;
    };
    use std::sync::atomic::{AtomicBool, Ordering};

    static OPEN: AtomicBool = AtomicBool::new(false);
    if OPEN.swap(true, Ordering::Relaxed) {
        return true;
    }
    let Ok(mut child) = std::process::Command::new(path)
        .args(["--practice", "--lang", lang.code()])
        .spawn()
    else {
        OPEN.store(false, Ordering::Relaxed);
        return false;
    };
    // Reap the companion after the player closes it, without blocking the lobby.
    std::thread::spawn(move || {
        let _ = child.wait();
        OPEN.store(false, Ordering::Relaxed);
    });
    true
}

#[cfg(target_arch = "wasm32")]
pub(super) fn launch(lang: Lang) -> bool {
    web_sys::window().is_some_and(|window| {
        window
            .location()
            .assign(&format!("moba/?lang={}", lang.code()))
            .is_ok()
    })
}
