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
            static_dir: std::env::var("TBG_STATIC_DIR").ok().map(PathBuf::from),
            db: Some(
                std::env::var("TBG_DB")
                    .map_or_else(|_| PathBuf::from("tbg.sqlite3"), PathBuf::from),
            ),
        }
    }
}
