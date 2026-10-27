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
            .filter_map(|pair| pair.split_once('='))
            .find(|(k, _)| *k == name)
            .map(|(_, v)| v.to_string())
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        std::env::var(format!("TBG_{}", name.to_uppercase())).ok()
    }
}
