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

/// P47: the match a `?watch=<16 hex>` link names; `None` for anything else.
pub fn watch_param() -> Option<u64> {
    parse_watch_id(&url_param("watch")?)
}

fn parse_watch_id(hex: &str) -> Option<u64> {
    (hex.len() == 16).then(|| u64::from_str_radix(hex, 16).ok())?
}

/// P47: the watch a reload of this tab resumes (`sessionStorage`).
pub fn load_watch() -> Option<u64> {
    #[cfg(target_arch = "wasm32")]
    {
        parse_watch_id(&token_storage()?.get_item("tbg_watch").ok()??)
    }
    #[cfg(not(target_arch = "wasm32"))]
    None
}

pub fn save_watch(id: u64) {
    #[cfg(target_arch = "wasm32")]
    if let Some(s) = token_storage() {
        let _ = s.set_item("tbg_watch", &format!("{id:016x}"));
    }
    #[cfg(not(target_arch = "wasm32"))]
    let _ = id;
}

pub fn clear_watch() {
    #[cfg(target_arch = "wasm32")]
    if let Some(s) = token_storage() {
        let _ = s.remove_item("tbg_watch");
    }
}

pub fn save_token(token: &[u8; 16]) {
    #[cfg(target_arch = "wasm32")]
    if let Some(s) = token_storage() {
        let _ = s.set_item("tbg_token", &hex16(token));
    }
    #[cfg(not(target_arch = "wasm32"))]
    let _ = token;
}

/// D-40/P40: the Arcana preset the lobby editor saved under `tbg.arcanas`
/// (`{"schema":2,"ranks":[...]}`); a schema-1 six-node document is migrated,
/// and the default is used when absent, partial or invalid, so every match
/// always gets a complete preset.
pub fn load_arcanas() -> tbg_core::ArcanaPreset {
    #[cfg(target_arch = "wasm32")]
    if let Some(raw) = storage().and_then(|s| s.get_item(ARCANAS_KEY).ok().flatten()) {
        if let Some(p) = parse_arcanas(&raw) {
            // Persist the migrated/completed schema so the next load is direct.
            save_arcanas(p);
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
            &format!("{{\"schema\":2,\"ranks\":[{}]}}", ranks.join(",")),
        );
    }
    #[cfg(not(target_arch = "wasm32"))]
    let _ = preset;
}

#[cfg(target_arch = "wasm32")]
const ARCANAS_KEY: &str = "tbg.arcanas";

/// D-46: the Stronghold kind the lobby picker saved under `tbg.stronghold`
/// (the kind's name, e.g. `Artillery`); Bastion when absent or invalid
/// (BR-SK01).
pub fn load_stronghold() -> tbg_core::StrongholdKind {
    #[cfg(target_arch = "wasm32")]
    if let Some(raw) = storage().and_then(|s| s.get_item(STRONGHOLD_KEY).ok().flatten()) {
        if let Some(kind) = parse_stronghold(&raw) {
            return kind;
        }
    }
    tbg_core::StrongholdKind::DEFAULT
}

/// D-46: keep the server's selection (e.g. an account's saved kind).
pub fn save_stronghold(kind: tbg_core::StrongholdKind) {
    #[cfg(target_arch = "wasm32")]
    if let Some(s) = storage() {
        let _ = s.set_item(STRONGHOLD_KEY, &format!("{kind:?}"));
    }
    #[cfg(not(target_arch = "wasm32"))]
    let _ = kind;
}

#[cfg(target_arch = "wasm32")]
const STRONGHOLD_KEY: &str = "tbg.stronghold";

/// ARENAS-G: the arena Practice uses, chosen in the Arenas page (the page
/// writes the slug; Rust reads it). Online matches never read it.
#[cfg(target_arch = "wasm32")]
const ARENA_KEY: &str = "tbg.arena";

/// The Practice arena: the saved choice, else the neutral Forest.
pub fn load_arena() -> tbg_core::Arena {
    #[cfg(target_arch = "wasm32")]
    if let Some(raw) = storage().and_then(|s| s.get_item(ARENA_KEY).ok().flatten()) {
        if let Some(arena) = tbg_core::Arena::from_slug(raw.trim()) {
            return arena;
        }
    }
    tbg_core::Arena::Forest
}

/// A selectable kind by name; anything else (Classic, junk) is `None`.
#[cfg(any(target_arch = "wasm32", test))]
pub fn parse_stronghold(raw: &str) -> Option<tbg_core::StrongholdKind> {
    tbg_core::StrongholdKind::SELECTABLE
        .into_iter()
        .find(|k| format!("{k:?}") == raw.trim())
}

/// The numbers after `"ranks":[`, as a complete preset. Six values are a
/// schema-1 tree and migrate; fifteen are validated as the current schema.
#[cfg(any(target_arch = "wasm32", test))]
pub fn parse_arcanas(raw: &str) -> Option<tbg_core::ArcanaPreset> {
    if raw.len() > 4096 {
        return None;
    }
    let list = raw.split("\"ranks\"").nth(1)?;
    let inner = list.split('[').nth(1)?.split(']').next()?;
    let ranks: Vec<u8> = inner
        .split(',')
        .map(|n| n.trim().parse().ok())
        .collect::<Option<_>>()?;
    if ranks.len() == 6 {
        return Some(tbg_core::ArcanaPreset::migrate_legacy(
            ranks.try_into().ok()?,
        ));
    }
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

/// P39: shareable link that auto-joins a Quick Challenge.
pub fn challenge_link(token: &[u8; 16]) -> String {
    let hex = invite_token_hex(token);
    #[cfg(target_arch = "wasm32")]
    if let Some(loc) = web_sys::window().map(|w| w.location()) {
        let origin = loc.origin().unwrap_or_default();
        let path = loc.pathname().unwrap_or_default();
        let server = url_param("server").map_or_else(String::new, |s| format!("&server={s}"));
        return format!("{origin}{path}?challenge={hex}{server}");
    }
    format!("?challenge={hex}")
}

// Browser support varies, and dismissing the share sheet rejects its Promise.
// Keep both that rejection and clipboard denial inside the browser helper.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen(inline_js = r#"
export function tbgCopyText(text) {
    if (navigator.clipboard) navigator.clipboard.writeText(text).catch(() => {});
}
export function tbgShareText(title, url) {
    const data = { title, text: title, url };
    try {
        if (typeof navigator.share === 'function' &&
            (typeof navigator.canShare !== 'function' || navigator.canShare(data))) {
            navigator.share(data).catch(error => {
                if (error.name !== 'AbortError') tbgCopyText(url);
            });
            return;
        }
    } catch (_) { /* Unsupported data falls back to the clipboard. */ }
    tbgCopyText(url);
}
"#)]
extern "C" {
    #[wasm_bindgen::prelude::wasm_bindgen(js_name = tbgShareText)]
    fn browser_share_text(title: &str, url: &str);
    #[wasm_bindgen::prelude::wasm_bindgen(js_name = tbgCopyText)]
    fn browser_copy_text(text: &str);
}

/// P39: native share sheet when available; copying the link otherwise.
pub fn share_text(title: &str, url: &str) {
    #[cfg(target_arch = "wasm32")]
    browser_share_text(title, url);
    #[cfg(not(target_arch = "wasm32"))]
    let _ = (title, url);
}

#[cfg(any(target_arch = "wasm32", test))]
fn without_url_param(search: &str, name: &str) -> String {
    let rest: Vec<&str> = search
        .trim_start_matches('?')
        .split('&')
        .filter(|pair| !pair.is_empty() && pair.split('=').next() != Some(name))
        .collect();
    if rest.is_empty() {
        String::new()
    } else {
        format!("?{}", rest.join("&"))
    }
}

/// P39: remove one query parameter without disturbing the others.
pub fn clear_url_param(name: &str) {
    #[cfg(target_arch = "wasm32")]
    if let Some(window) = web_sys::window() {
        let loc = window.location();
        let search = loc.search().unwrap_or_default();
        let query = without_url_param(&search, name);
        if let Ok(history) = window.history() {
            let url = format!(
                "{}{}{}{}",
                loc.origin().unwrap_or_default(),
                loc.pathname().unwrap_or_default(),
                query,
                loc.hash().unwrap_or_default()
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
    browser_copy_text(text);
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
mod stronghold_tests {
    use super::parse_stronghold;
    use tbg_core::StrongholdKind;

    #[test]
    fn picker_names_parse_and_classic_never_does() {
        for kind in StrongholdKind::SELECTABLE {
            assert_eq!(parse_stronghold(&format!("{kind:?}")), Some(kind));
        }
        for bad in ["Classic", "", "artillery", "Bastion; drop", "4"] {
            assert_eq!(parse_stronghold(bad), None, "{bad}");
        }
    }
}

#[cfg(test)]
mod arcana_tests {
    use super::parse_arcanas;
    use tbg_core::ArcanaPreset;

    #[test]
    fn the_editor_document_round_trips_and_legacy_rows_migrate() {
        // The exact document `editor.js` saves (P40).
        let doc = r#"{"schema":2,"ranks":[2,2,2,2,2,2,0,0,0,0,0,0,0,0,0]}"#;
        assert_eq!(
            parse_arcanas(doc).map(|p| p.ranks),
            Some(ArcanaPreset::DEFAULT.ranks)
        );
        // A schema-1 six-node row preserves its ranks and gains the six
        // new points on the original nodes.
        let legacy = r#"{"schema":1,"ranks":[2,0,2,0,2,0]}"#;
        assert_eq!(
            parse_arcanas(legacy).map(|p| p.ranks),
            Some(ArcanaPreset::DEFAULT.ranks)
        );
        // Partial, oversized, malformed: none becomes a preset.
        for bad in [
            r#"{"schema":2,"ranks":[1,0,0,0,0,0,0,0,0,0,0,0,0,0,0]}"#,
            r#"{"schema":2,"ranks":[3,3,0,0,0,0,0,0,0,0,0,0,0,0,0]}"#,
            r#"{"schema":2,"ranks":[2,2,2]}"#,
            r#"{"schema":2,"ranks":"x"}"#,
            "",
        ] {
            assert_eq!(parse_arcanas(bad), None, "{bad}");
        }
        assert_eq!(parse_arcanas(&"9".repeat(5_000)), None);
        assert!(ArcanaPreset::complete(ArcanaPreset::DEFAULT.ranks).is_ok());
    }
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
    fn removing_a_consumed_invite_preserves_server_and_lag() {
        assert_eq!(
            without_url_param(
                "?server=ws://localhost:8080/ws&challenge=abc&lag=200",
                "challenge"
            ),
            "?server=ws://localhost:8080/ws&lag=200"
        );
        assert_eq!(without_url_param("?challenge=abc", "challenge"), "");
        assert_eq!(
            without_url_param("?challenge=abc&challenge=def&room=ABCDE", "challenge"),
            "?room=ABCDE"
        );
    }

    #[test]
    fn native_challenge_link_contains_the_token() {
        assert_eq!(
            challenge_link(&[0x0F; 16]),
            format!("?challenge={}", "0f".repeat(16))
        );
    }
}

/// Result motion obeys explicit preference first, then the browser's OS accessibility preference.
pub fn reduced_motion() -> bool {
    if let Some(value) = url_param("reduced_motion") {
        return !matches!(value.as_str(), "0" | "false" | "off");
    }
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(value) = storage().and_then(|s| s.get_item("tbg_reduced_motion").ok().flatten())
        {
            return value == "1";
        }
        return web_sys::window()
            .and_then(|w| {
                w.match_media("(prefers-reduced-motion: reduce)")
                    .ok()
                    .flatten()
            })
            .is_some_and(|m| m.matches());
    }
    #[cfg(not(target_arch = "wasm32"))]
    false
}

pub fn save_reduced_motion(value: bool) {
    save_flag("tbg_reduced_motion", value);
}

/// One tab-local cosmetic dismissal; no account key or session token is removed.
pub fn dismissed_result() -> Option<tbg_core::protocol::ResultId> {
    #[cfg(target_arch = "wasm32")]
    {
        parse_hex16(&token_storage()?.get_item("tbg_dismissed_result").ok()??)
    }
    #[cfg(not(target_arch = "wasm32"))]
    None
}
pub fn dismiss_result(id: tbg_core::protocol::ResultId) {
    #[cfg(target_arch = "wasm32")]
    if let Some(s) = token_storage() {
        let _ = s.set_item("tbg_dismissed_result", &hex16(&id));
    }
    #[cfg(not(target_arch = "wasm32"))]
    let _ = id;
}
pub fn clear_dismissed_result() {
    #[cfg(target_arch = "wasm32")]
    if let Some(s) = token_storage() {
        let _ = s.remove_item("tbg_dismissed_result");
    }
}
