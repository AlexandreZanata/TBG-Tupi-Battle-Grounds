//! Launch options from the command line (native) with safe defaults (browser).

use bevy::prelude::*;
use tbg_moba_core::Role;

use crate::i18n::Lang;

/// Default local room address of `tbg-moba-server`.
pub const DEFAULT_SERVER: &str = "ws://127.0.0.1:7878/moba/ws";

#[derive(Resource, Clone, Debug)]
pub struct Launch {
    /// Play on a room server instead of offline practice vs 9 bots.
    pub online: bool,
    /// The 1v1 Practice launcher opens a bot-only menu.
    pub practice_only: bool,
    /// Room server WebSocket URL.
    pub server: String,
    pub role: Role,
    pub lang: Lang,
    /// Skip the menu and start a match at once.
    pub play: bool,
    /// Pre-simulate this many seconds of an offline match (review captures).
    pub warp_secs: u32,
    pub seed: u64,
    /// Save one screenshot to this path a few seconds in, then exit (desktop only).
    #[cfg_attr(target_arch = "wasm32", allow(dead_code))]
    pub capture: Option<String>,
    /// Start zoomed out over the whole valley (review captures).
    pub overview: bool,
}

impl Launch {
    pub fn from_args() -> Self {
        let args: Vec<String> = std::env::args().skip(1).collect();
        let value = |flag: &str| {
            args.iter()
                .position(|a| a == flag)
                .and_then(|i| args.get(i + 1))
                .cloned()
        };
        let has = |flag: &str| args.iter().any(|a| a == flag);
        let practice_only = cfg!(target_arch = "wasm32") || has("--practice");
        Self {
            online: !practice_only && has("--online"),
            practice_only,
            server: value("--online")
                .filter(|v| v.starts_with("ws"))
                .unwrap_or_else(|| DEFAULT_SERVER.into()),
            role: value("--role")
                .and_then(|r| parse_role(&r))
                .unwrap_or(Role::Support),
            lang: language(value("--lang").as_deref()),
            play: has("--play") || has("--capture"),
            warp_secs: value("--warp")
                .and_then(|v| v.parse().ok())
                .unwrap_or(0)
                .min(25 * 60),
            seed: value("--seed")
                .and_then(|v| v.parse().ok())
                .unwrap_or_else(clock_seed),
            capture: value("--capture"),
            overview: has("--overview"),
        }
    }
}

fn language(native: Option<&str>) -> Lang {
    #[cfg(target_arch = "wasm32")]
    let browser = web_sys::window()
        .and_then(|window| window.location().search().ok())
        .unwrap_or_default();
    #[cfg(target_arch = "wasm32")]
    let native = browser
        .trim_start_matches('?')
        .split('&')
        .find_map(|part| part.strip_prefix("lang="))
        .or(native);
    match native {
        Some("en") => Lang::En,
        _ => Lang::Pt,
    }
}

/// Return to the originating 1v1 preview. The native companion closes its window.
#[cfg(target_arch = "wasm32")]
pub fn return_to_lobby(lang: Lang) {
    if let Some(window) = web_sys::window() {
        let code = if lang == Lang::En { "en" } else { "pt-BR" };
        let _ = window
            .location()
            .assign(&format!("../?practice5v5=1&lang={code}"));
    }
}

pub fn parse_role(text: &str) -> Option<Role> {
    Role::ALL
        .into_iter()
        .find(|r| format!("{r:?}").eq_ignore_ascii_case(text))
}

/// Offline practice only needs variety, not secrecy.
fn clock_seed() -> u64 {
    #[cfg(not(target_arch = "wasm32"))]
    {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(1, |d| d.as_nanos() as u64)
    }
    #[cfg(target_arch = "wasm32")]
    {
        1
    }
}

/// `--name value` (or bare `--name`) from the command line; the shared 1v1
/// art reads its options through this name.
pub fn url_param(name: &str) -> Option<String> {
    let flag = format!("--{name}");
    let args: Vec<String> = std::env::args().skip(1).collect();
    let i = args.iter().position(|a| *a == flag)?;
    Some(
        args.get(i + 1)
            .filter(|v| !v.starts_with("--"))
            .cloned()
            .unwrap_or_default(),
    )
}

/// `--quality low` trims motion detail in the shared 1v1 animation.
pub fn low_motion() -> bool {
    static LOW: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *LOW.get_or_init(|| url_param("quality").as_deref() == Some("low"))
}
