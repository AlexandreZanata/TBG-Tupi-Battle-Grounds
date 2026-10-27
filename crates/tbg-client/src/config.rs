//! Page config: URL params on wasm32, env vars on native.
//!
//! Single helper so later phases read `?room=`, `?server=`, `?lag=` the
//! same way.

/// Read parameter `name`: on wasm32 from the page query string
/// (`?name=value`), on native from `TBG_<NAME>` (uppercased).
pub fn url_param(name: &str) -> Option<String> {    #[cfg(target_arch = "wasm32")]
    {
        let search = web_sys::window()?.location().search().ok()?;
        search
            .trim_start_matches('?')
            .split('&')
            .filter_map(|pair| pair.split_once('='))
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
        return "ws://127.0.0.1:8080/ws".into();
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        "ws://127.0.0.1:8080/ws".into()
    }
}
