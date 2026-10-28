//! Per-IP rate limits for account endpoints (BR-P01): fixed windows kept
//! as timestamp logs, pruned lazily on every check and swept clean every
//! minute. Memory-only: IPs are never stored on disk or logged.

use std::collections::HashMap;
use std::net::IpAddr;
use std::time::{Duration, Instant};

/// Sliding-log limiter (stricter than fixed windows: at most `max`
/// attempts in any trailing window). Two independent logs.
#[derive(Default)]
pub struct Limits {
    register: HashMap<IpAddr, Vec<Instant>>,
    login: HashMap<IpAddr, Vec<Instant>>,
    last_sweep: Option<Instant>,
}

impl Limits {
    /// `true` when the register attempt is allowed (records it on allow).
    pub fn check_register(&mut self, ip: IpAddr, max_per_hour: u32, now: Instant) -> bool {
        check_log(
            self.register.entry(ip).or_default(),
            max_per_hour,
            Duration::from_secs(3600),
            now,
        )
    }

    /// `true` when the login attempt is allowed (records it on allow).
    pub fn check_login(&mut self, ip: IpAddr, max_per_min: u32, now: Instant) -> bool {
        check_log(
            self.login.entry(ip).or_default(),
            max_per_min,
            Duration::from_secs(60),
            now,
        )
    }

    /// Drop empty logs; call about once a minute (the lobby cleanup tick).
    pub fn sweep(&mut self, now: Instant) {
        if self
            .last_sweep
            .is_some_and(|t| now - t < Duration::from_secs(60))
        {
            return;
        }
        self.last_sweep = Some(now);
        self.register.retain(|_, v| !v.is_empty());
        self.login.retain(|_, v| !v.is_empty());
    }
}

fn check_log(log: &mut Vec<Instant>, max: u32, window: Duration, now: Instant) -> bool {
    log.retain(|t| now.saturating_duration_since(*t) < window);
    if log.len() >= max as usize {
        return false;
    }
    log.push(now);
    true
}
