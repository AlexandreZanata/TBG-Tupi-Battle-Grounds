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
    /// Room server WebSocket URL.
    pub server: String,
    pub role: Role,
    pub lang: Lang,
    /// Skip the menu and start a match at once.
    pub play: bool,
    /// Pre-simulate this many seconds of an offline match (review captures).
    pub warp_secs: u32,
    pub seed: u64,
    /// Save one screenshot to this path a few seconds in, then exit.
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
        Self {
            online: has("--online"),
            server: value("--online")
                .filter(|v| v.starts_with("ws"))
                .unwrap_or_else(|| DEFAULT_SERVER.into()),
            role: value("--role")
                .and_then(|r| parse_role(&r))
                .unwrap_or(Role::Support),
            lang: value("--lang").map_or(Lang::Pt, |l| if l == "en" { Lang::En } else { Lang::Pt }),
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
