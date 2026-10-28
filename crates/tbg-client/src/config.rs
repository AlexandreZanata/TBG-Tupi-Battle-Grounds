//! Page config: URL params on wasm32, env vars on native.
//!
//! Single helper so later phases read `?room=`, `?server=`, `?lag=` the
//! same way.

/// Read parameter `name`: on wasm32 from the page query string
/// (`?name=value`), on native from `TBG_<NAME>` (uppercased).
pub fn url_param(name: &str) -> Option<String> {
    #[cfg(target_arch = "wasm32")]
    {
        let search = web_sys::window()?.location().search().ok()?;
        search
            .trim_start_matches('?')
            .split('&')
            // A bare flag (`?autoplay`) reads as an empty value.
            .map(|pair| pair.split_once('=').unwrap_or((pair, "")))
            .find(|(k, _)| *k == name)
            .map(|(_, v)| v.to_string())
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        std::env::var(format!("TBG_{}", name.to_uppercase())).ok()
    }
}

/// WebSocket URL for the ping socket:
/// 1. `?server=<url>` when present;
/// 2. on wasm with `https:` page → `wss://<host>/ws`;
/// 3. on wasm otherwise → `ws://<hostname>:8080/ws`;
/// 4. native → `ws://127.0.0.1:8080/ws`.
pub fn server_url() -> String {
    if let Some(url) = url_param("server") {
        return url;
    }
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(window) = web_sys::window() {
            let location = window.location();
            if location.protocol().as_deref() == Ok("https:") {
                if let Ok(host) = location.host() {
                    return format!("wss://{host}/ws");
                }
            } else if let Ok(hostname) = location.hostname() {
                return format!("ws://{hostname}:8080/ws");
            }
        }
        "ws://127.0.0.1:8080/ws".into()
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        "ws://127.0.0.1:8080/ws".into()
    }
}

/// Session token kept across page reloads so a player can resume a match.
pub fn load_token() -> Option<[u8; 16]> {
    #[cfg(target_arch = "wasm32")]
    {
        let hex = storage()?.get_item("tbg_token").ok()??;
        let mut out = [0u8; 16];
        for (i, b) in out.iter_mut().enumerate() {
            *b = u8::from_str_radix(hex.get(i * 2..i * 2 + 2)?, 16).ok()?;
        }
        Some(out)
    }
    #[cfg(not(target_arch = "wasm32"))]
    None
}

pub fn save_token(token: &[u8; 16]) {
    #[cfg(target_arch = "wasm32")]
    if let Some(s) = storage() {
        let hex: String = token.iter().map(|b| format!("{b:02x}")).collect();
        let _ = s.set_item("tbg_token", &hex);
    }
    #[cfg(not(target_arch = "wasm32"))]
    let _ = token;
}

#[cfg(target_arch = "wasm32")]
fn storage() -> Option<web_sys::Storage> {
    web_sys::window()?.local_storage().ok()?
}

/// Shareable link that auto-joins a private room.
pub fn room_link(code: &str) -> String {
    #[cfg(target_arch = "wasm32")]
    if let Some(loc) = web_sys::window().map(|w| w.location()) {
        let origin = loc.origin().unwrap_or_default();
        let path = loc.pathname().unwrap_or_default();
        return format!("{origin}{path}?room={code}");
    }
    format!("?room={code}")
}

pub fn copy_text(text: &str) {
    #[cfg(target_arch = "wasm32")]
    if let Some(w) = web_sys::window() {
        let _ = w.navigator().clipboard().write_text(text);
    }
    #[cfg(not(target_arch = "wasm32"))]
    let _ = text;
}

/// A boolean preference in `localStorage` (false when unavailable).
pub fn load_flag(key: &str) -> bool {
    #[cfg(target_arch = "wasm32")]
    {
        storage()
            .and_then(|s| s.get_item(key).ok().flatten())
            .is_some_and(|v| v == "1")
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = key;
        false
    }
}

pub fn save_flag(key: &str, value: bool) {
    #[cfg(target_arch = "wasm32")]
    if let Some(s) = storage() {
        let _ = s.set_item(key, if value { "1" } else { "0" });
    }
    #[cfg(not(target_arch = "wasm32"))]
    let _ = (key, value);
}
