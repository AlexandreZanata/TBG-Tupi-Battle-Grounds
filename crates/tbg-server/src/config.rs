//! Server settings from environment variables (docs/phases/P4).

use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct Config {
    pub addr: String,
    pub max_connections: usize,
    pub max_per_ip: usize,
    pub reconnect_secs: u64,
    /// Simulation tick length; 50 ms in production. Tests shorten it.
    pub tick_ms: u64,
    pub static_dir: Option<PathBuf>,
    /// SQLite file for match records (`TBG_DB`); `None` disables records.
    pub db: Option<PathBuf>,
    /// Tokio worker threads (`TBG_WORKERS`); 0 (default) means all cores.
    pub workers: usize,
    /// P11: registers allowed per IP per hour (`TBG_REGISTER_PER_IP_HOUR`).
    pub register_per_ip_hour: u64,
    /// P11: logins allowed per IP per minute (`TBG_LOGIN_PER_IP_MIN`).
    pub login_per_ip_min: u64,
    /// P13: `/api/*` requests allowed per IP per minute (`TBG_API_PER_IP_MIN`).
    pub api_per_ip_min: u64,
    /// P39: Quick Challenge invitation lifetime.
    pub challenge_ttl_secs: u64,
    /// P39: rematch offer lifetime after a challenge match.
    pub rematch_ttl_secs: u64,
    /// P39: `JoinChallenge` attempts allowed per IP per minute.
    pub invite_per_ip_min: u64,
    /// P47 (BR-LV09, O-3): spectators allowed in one listed match.
    pub spectators_per_match: usize,
    /// P47 (BR-LV09, O-3): spectators allowed on the whole server.
    pub max_spectators: usize,
    /// P16-C: tests receive every finished match's replay here, to prove a
    /// headless re-run gives the same state (D-33). Never set from the
    /// environment: the seed stays on the server (BR-S05).
    pub replays: Option<tokio::sync::mpsc::UnboundedSender<tbg_core::replay::Replay>>,
    /// P60: shadow integrity policy (`TBG_RANKED_*`, validated in `main`).
    pub ranked: crate::ranked::policy::RankedPolicy,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            addr: "0.0.0.0:8080".into(),
            max_connections: 2_000,
            max_per_ip: 8,
            reconnect_secs: 30,
            tick_ms: 50,
            static_dir: None,
            db: None,
            workers: 0,
            register_per_ip_hour: 3,
            login_per_ip_min: 10,
            api_per_ip_min: 120,
            challenge_ttl_secs: 600,
            rematch_ttl_secs: 120,
            invite_per_ip_min: 30,
            spectators_per_match: 200,
            max_spectators: 2_000,
            replays: None,
            ranked: crate::ranked::policy::RankedPolicy::default(),
        }
    }
}

impl Config {
    pub fn from_env() -> Config {
        let d = Config::default();
        let num = |key: &str, default: u64| {
            std::env::var(key)
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(default)
        };
        Config {
            addr: std::env::var("TBG_ADDR").unwrap_or(d.addr),
            max_connections: num("TBG_MAX_CONNECTIONS", d.max_connections as u64) as usize,
            max_per_ip: num("TBG_MAX_PER_IP", d.max_per_ip as u64) as usize,
            reconnect_secs: num("TBG_RECONNECT_SECS", d.reconnect_secs),
            tick_ms: num("TBG_TICK_MS", d.tick_ms).max(1),
            workers: num("TBG_WORKERS", 0) as usize,
            register_per_ip_hour: num("TBG_REGISTER_PER_IP_HOUR", d.register_per_ip_hour),
            login_per_ip_min: num("TBG_LOGIN_PER_IP_MIN", d.login_per_ip_min),
            api_per_ip_min: num("TBG_API_PER_IP_MIN", d.api_per_ip_min),
            challenge_ttl_secs: d.challenge_ttl_secs,
            rematch_ttl_secs: d.rematch_ttl_secs,
            invite_per_ip_min: d.invite_per_ip_min,
            spectators_per_match: num("TBG_SPECTATORS_PER_MATCH", d.spectators_per_match as u64)
                as usize,
            max_spectators: num("TBG_MAX_SPECTATORS", d.max_spectators as u64) as usize,
            static_dir: std::env::var("TBG_STATIC_DIR").ok().map(PathBuf::from),
            db: Some(
                std::env::var("TBG_DB")
                    .map_or_else(|_| PathBuf::from("tbg.sqlite3"), PathBuf::from),
            ),
            replays: None,
            ranked: d.ranked,
        }
    }
}
