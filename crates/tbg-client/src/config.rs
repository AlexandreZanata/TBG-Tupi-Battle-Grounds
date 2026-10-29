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

fn hex16(token: &[u8; 16]) -> String {
    token.iter().map(|b| format!("{b:02x}")).collect()
}

fn parse_hex16(hex: &str) -> Option<[u8; 16]> {
    if hex.len() != 32 {
        return None;
    }
    let mut out = [0u8; 16];
    for (i, b) in out.iter_mut().enumerate() {
        *b = u8::from_str_radix(hex.get(i * 2..i * 2 + 2)?, 16).ok()?;
    }
    Some(out)
}

/// P39: hex encoding used by the `?challenge=` link.
pub fn invite_token_hex(token: &[u8; 16]) -> String {
    hex16(token)
}

/// P39: parse the 32-hex-character invite token from a URL.
pub fn parse_invite_token(hex: &str) -> Option<[u8; 16]> {
    parse_hex16(hex)
}

/// Session token kept across page reloads so a player can resume a match.
/// Per tab (`sessionStorage`): a shared token made two open tabs keep
/// taking over each other's session.
pub fn load_token() -> Option<[u8; 16]> {
    #[cfg(target_arch = "wasm32")]
    {
        parse_hex16(&token_storage()?.get_item("tbg_token").ok()??)
    }
    #[cfg(not(target_arch = "wasm32"))]
    None
}

pub fn save_token(token: &[u8; 16]) {
    #[cfg(target_arch = "wasm32")]
    if let Some(s) = token_storage() {
        let _ = s.set_item("tbg_token", &hex16(token));
    }
    #[cfg(not(target_arch = "wasm32"))]
    let _ = token;
}

#[cfg(target_arch = "wasm32")]
fn storage() -> Option<web_sys::Storage> {
    web_sys::window()?.local_storage().ok()?
}

#[cfg(target_arch = "wasm32")]
fn token_storage() -> Option<web_sys::Storage> {
    web_sys::window()?.session_storage().ok()?
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

/// P39: shareable link that auto-joins a Quick Challenge.
pub fn challenge_link(token: &[u8; 16]) -> String {
    let hex = invite_token_hex(token);
    #[cfg(target_arch = "wasm32")]
    if let Some(loc) = web_sys::window().map(|w| w.location()) {
        let origin = loc.origin().unwrap_or_default();
        let path = loc.pathname().unwrap_or_default();
        return format!("{origin}{path}?challenge={hex}");
    }
    format!("?challenge={hex}")
}

/// P39: native share sheet when available; copying the link otherwise.
pub fn share_text(title: &str, url: &str) {
    #[cfg(target_arch = "wasm32")]
    if let Some(window) = web_sys::window() {
        let navigator = window.navigator();
        let data = web_sys::ShareData::new();
        data.set_title(title);
        data.set_text(title);
        data.set_url(url);
        if web_sys::Navigator::can_share_with_data(&navigator, &data) {
            let _ = web_sys::Navigator::share_with_data(&navigator, &data);
            return;
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    let _ = title;
    copy_text(url);
}

/// P39: remove one query parameter without disturbing the others.
pub fn clear_url_param(name: &str) {
    #[cfg(target_arch = "wasm32")]
    if let Some(window) = web_sys::window() {
        let loc = window.location();
        let search = loc.search().unwrap_or_default();
        let rest: Vec<&str> = search
            .trim_start_matches('?')
            .split('&')
            .filter(|pair| !pair.is_empty() && pair.split('=').next() != Some(name))
            .collect();
        let query = if rest.is_empty() {
            String::new()
        } else {
            format!("?{}", rest.join("&"))
        };
        if let Ok(history) = window.history() {
            let url = format!(
                "{}{}{}",
                loc.origin().unwrap_or_default(),
                loc.pathname().unwrap_or_default(),
                query
            );
            let _ = history.replace_state_with_url(&wasm_bindgen::JsValue::NULL, "", Some(&url));
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    let _ = name;
}

/// P13: open a same-site page (`/ranking/`) in a new tab.
pub fn open_url(path: &str) {
    #[cfg(target_arch = "wasm32")]
    if let Some(w) = web_sys::window() {
        let _ = w.open_with_url_and_target(path, "_blank");
    }
    #[cfg(not(target_arch = "wasm32"))]
    let _ = path;
}

pub fn copy_text(text: &str) {
    #[cfg(target_arch = "wasm32")]
    if let Some(w) = web_sys::window() {
        let _ = w.navigator().clipboard().write_text(text);
    }
    #[cfg(not(target_arch = "wasm32"))]
    let _ = text;
}

/// P11: the player's own account key (`None` on native). Anyone with it
/// can play as the account; it lives only in `localStorage`.
pub fn load_account_key() -> Option<String> {
    #[cfg(target_arch = "wasm32")]
    {
        storage()?.get_item("tbg_account_key").ok()?
    }
    #[cfg(not(target_arch = "wasm32"))]
    None
}

pub fn save_account_key(key: &str) {
    #[cfg(target_arch = "wasm32")]
    if let Some(s) = storage() {
        let _ = s.set_item("tbg_account_key", key);
    }
    #[cfg(not(target_arch = "wasm32"))]
    let _ = key;
}

pub fn delete_account_key() {
    #[cfg(target_arch = "wasm32")]
    if let Some(s) = storage() {
        let _ = s.remove_item("tbg_account_key");
    }
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

/// Presentation-only arena inspection for P16-A captures, without the menu or orbit.
pub fn arena_preview() -> bool {
    static PREVIEW: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *PREVIEW.get_or_init(|| url_param("arena").is_some())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invite_token_hex_round_trip() {
        let token = [0xAB; 16];
        assert_eq!(invite_token_hex(&token), "ab".repeat(16));
        assert_eq!(parse_invite_token(&invite_token_hex(&token)), Some(token));
    }

    #[test]
    fn invite_token_accepts_uppercase_hex() {
        assert_eq!(parse_invite_token(&"AB".repeat(16)), Some([0xAB; 16]));
    }

    #[test]
    fn invite_token_rejects_bad_length_and_chars() {
        assert_eq!(parse_invite_token("ab"), None);
        assert_eq!(parse_invite_token(&"zz".repeat(16)), None);
        assert_eq!(parse_invite_token(&"a".repeat(31)), None);
    }

    #[test]
    fn native_challenge_link_contains_the_token() {
        assert_eq!(
            challenge_link(&[0x0F; 16]),
            format!("?challenge={}", "0f".repeat(16))
        );
    }
}
