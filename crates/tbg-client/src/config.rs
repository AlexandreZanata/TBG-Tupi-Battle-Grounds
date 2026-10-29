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
/// Per tab (`sessionStorage`): a shared token made two open tabs keep
/// taking over each other's session.
pub fn load_token() -> Option<[u8; 16]> {
    #[cfg(target_arch = "wasm32")]
    {
        let hex = token_storage()?.get_item("tbg_token").ok()??;
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
    if let Some(s) = token_storage() {
        let hex: String = token.iter().map(|b| format!("{b:02x}")).collect();
        let _ = s.set_item("tbg_token", &hex);
    }
    #[cfg(not(target_arch = "wasm32"))]
    let _ = token;
}

/// D-40: the Arcana preset the lobby editor saved under `tbg.arcanas`
/// (`{"schema":1,"ranks":[...]}`); the default when absent, partial or
/// invalid, so every match always gets a complete preset.
pub fn load_arcanas() -> tbg_core::ArcanaPreset {
    #[cfg(target_arch = "wasm32")]
    if let Some(raw) = storage().and_then(|s| s.get_item(ARCANAS_KEY).ok().flatten()) {
        if let Some(p) = parse_arcanas(&raw) {
            return p;
        }
    }
    tbg_core::ArcanaPreset::DEFAULT
}

/// D-40: keep the server's selection (e.g. an account's saved preset).
pub fn save_arcanas(preset: tbg_core::ArcanaPreset) {
    #[cfg(target_arch = "wasm32")]
    if let Some(s) = storage() {
        let ranks: Vec<String> = preset.ranks.iter().map(u8::to_string).collect();
        let _ = s.set_item(
            ARCANAS_KEY,
            &format!("{{\"schema\":1,\"ranks\":[{}]}}", ranks.join(",")),
        );
    }
    #[cfg(not(target_arch = "wasm32"))]
    let _ = preset;
}

#[cfg(target_arch = "wasm32")]
const ARCANAS_KEY: &str = "tbg.arcanas";

/// The six numbers after `"ranks":[`, as a complete preset.
#[cfg(any(target_arch = "wasm32", test))]
pub fn parse_arcanas(raw: &str) -> Option<tbg_core::ArcanaPreset> {
    if raw.len() > 1024 {
        return None;
    }
    let list = raw.split("\"ranks\"").nth(1)?;
    let inner = list.split('[').nth(1)?.split(']').next()?;
    let ranks: Vec<u8> = inner
        .split(',')
        .map(|n| n.trim().parse().ok())
        .collect::<Option<_>>()?;
    tbg_core::ArcanaPreset::complete(ranks.try_into().ok()?).ok()
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
mod arcana_tests {
    use super::parse_arcanas;
    use tbg_core::ArcanaPreset;

    #[test]
    fn the_editor_document_round_trips_and_bad_input_is_refused() {
        // The exact document `editor.js` saves (D-40).
        let doc = r#"{"schema":1,"ranks":[2,2,2,0,0,0]}"#;
        assert_eq!(
            parse_arcanas(doc).map(|p| p.ranks),
            Some([2, 2, 2, 0, 0, 0])
        );
        // Partial, oversized, malformed: none becomes a preset.
        for bad in [
            r#"{"schema":1,"ranks":[1,0,0,0,0,0]}"#,
            r#"{"schema":1,"ranks":[3,3,0,0,0,0]}"#,
            r#"{"schema":1,"ranks":[2,2,2]}"#,
            r#"{"schema":1,"ranks":"x"}"#,
            "",
        ] {
            assert_eq!(parse_arcanas(bad), None, "{bad}");
        }
        assert_eq!(parse_arcanas(&"9".repeat(2000)), None);
        assert!(ArcanaPreset::complete([2, 0, 2, 0, 2, 0]).is_ok());
    }
}
