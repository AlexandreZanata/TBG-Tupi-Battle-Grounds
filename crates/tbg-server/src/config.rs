//! Server settings from environment variables (docs/phases/P4).

use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct Config {
    pub admin: crate::admin::settings::Settings,
    pub moderation: crate::moderation::cache::Cache,
    pub mail: crate::mail::settings::Settings,
    pub mail_capture: Option<crate::mail::transport::MailCapture>,
    pub bootstrap_email: crate::mail::settings::Private<String>,
    pub feed_ms: u64,
    /// Fixed loopback release-slot ports aggregated by the P73 presence snapshot.
    pub presence_ports: [u16; 2],
    /// P73 staged rollout: migrate explicitly, then collect, owner API, public opt-in.
    pub history_migrate: bool,
    pub history_collect: bool,
    pub history_api: bool,
    pub history_public: bool,
    /// Programmatic synthetic test clock; never configurable over HTTP or environment.
    pub auth_clock: crate::auth::Clock,
    /// Exact reviewed browser origins; never derive cookie authority from Host.
    pub auth_origins: Vec<String>,
    pub addr: String,
    pub max_connections: usize,
    pub max_per_ip: usize,
    pub reconnect_secs: u64,
    /// P71 (BR-M03, Proposed): connection-pause budget per player per match.
    pub pause_secs: u64,
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
    /// P61: optional ranked statistics collection (`TBG_RANKED_STATS`).
    pub stats: crate::ranked::policy::StatsMode,
    /// P66: fixed release slot (`TBG_SLOT`, `a`–`z`); `None` keeps the
    /// single-process behavior with no release directory.
    pub slot: Option<String>,
    /// P66: this process's generation id (`TBG_GENERATION`); every
    /// replacement build gets a new one.
    pub generation: String,
    /// P66: immutable server build label (`TBG_SERVER_BUILD`).
    pub server_build: String,
    /// P66: the fixed proxy path of this slot (`TBG_WS_ENDPOINT`).
    pub ws_endpoint: String,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            admin: crate::admin::settings::Settings::default(),
            moderation: crate::moderation::cache::Cache::default(),
            mail: crate::mail::settings::Settings::default(),
            mail_capture: None,
            bootstrap_email: crate::mail::settings::Private::default(),
            feed_ms: 1000,
            presence_ports: [8080, 8082],
            history_migrate: false,
            history_collect: false,
            history_api: false,
            history_public: false,
            auth_clock: crate::auth::Clock::default(),
            auth_origins: vec![
                "https://app.tupibg.com".into(),
                "https://api.tupibg.com".into(),
            ],
            addr: "0.0.0.0:8080".into(),
            max_connections: 2_000,
            max_per_ip: 8,
            reconnect_secs: 30,
            pause_secs: 120,
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
            stats: crate::ranked::policy::StatsMode::default(),
            slot: None,
            generation: default_build(),
            server_build: default_build(),
            ws_endpoint: "/ws".into(),
        }
    }
}

/// `<package version>+<source commit>` (commit from the build env).
fn default_build() -> String {
    format!(
        "{}+{}",
        env!("CARGO_PKG_VERSION"),
        option_env!("TBG_SOURCE_COMMIT").unwrap_or("dev")
    )
}

/// A slot name: one lowercase ASCII letter.
fn slot_name(raw: &str) -> Option<String> {
    let ok = raw.len() == 1 && raw.bytes().all(|b| b.is_ascii_lowercase());
    ok.then(|| raw.to_string())
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
            admin: crate::admin::settings::Settings::from_env(),
            moderation: crate::moderation::cache::Cache::default(),
            mail: crate::mail::settings::Settings::from_env(),
            mail_capture: None,
            bootstrap_email: crate::mail::settings::Private(
                std::env::var("TBG_ADMIN_BOOTSTRAP_EMAIL").unwrap_or_default(),
            ),
            feed_ms: num("TBG_FEED_MS", 1000).clamp(100, 10000),
            presence_ports: [
                num("TBG_PRESENCE_PORT_A", 8080).clamp(1, 65535) as u16,
                num("TBG_PRESENCE_PORT_B", 8082).clamp(1, 65535) as u16,
            ],
            history_migrate: std::env::var("TBG_HISTORY_MIGRATE").as_deref() == Ok("1"),
            history_collect: std::env::var("TBG_HISTORY_COLLECT").as_deref() == Ok("1"),
            history_api: std::env::var("TBG_HISTORY_API").as_deref() == Ok("1"),
            history_public: std::env::var("TBG_HISTORY_PUBLIC").as_deref() == Ok("1"),
            auth_clock: d.auth_clock,
            auth_origins: std::env::var("TBG_AUTH_ORIGINS")
                .ok()
                .map(|v| v.split(',').map(|s| s.trim().to_owned()).collect())
                .unwrap_or(d.auth_origins),
            addr: std::env::var("TBG_ADDR").unwrap_or(d.addr),
            max_connections: num("TBG_MAX_CONNECTIONS", d.max_connections as u64) as usize,
            max_per_ip: num("TBG_MAX_PER_IP", d.max_per_ip as u64) as usize,
            reconnect_secs: num("TBG_RECONNECT_SECS", d.reconnect_secs),
            pause_secs: num("TBG_PAUSE_SECS", d.pause_secs),
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
            stats: d.stats,
            slot: std::env::var("TBG_SLOT").ok().and_then(|s| slot_name(&s)),
            generation: std::env::var("TBG_GENERATION").unwrap_or(d.generation),
            server_build: std::env::var("TBG_SERVER_BUILD").unwrap_or(d.server_build),
            ws_endpoint: std::env::var("TBG_WS_ENDPOINT").unwrap_or(d.ws_endpoint),
        }
    }
}
