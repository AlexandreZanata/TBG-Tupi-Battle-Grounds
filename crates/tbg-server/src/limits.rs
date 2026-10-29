//! Per-IP rate limits for account endpoints (BR-P01): fixed windows kept
//! as timestamp logs, pruned lazily on every check and swept clean every
//! minute. Memory-only: IPs are never stored on disk or logged.

use std::collections::HashMap;
use std::net::IpAddr;
use std::time::{Duration, Instant};

/// Sliding-log limiter (stricter than fixed windows: at most `max`
/// attempts in any trailing window). Independent logs per endpoint.
#[derive(Default)]
pub struct Limits {
    register: HashMap<IpAddr, Vec<Instant>>,
    login: HashMap<IpAddr, Vec<Instant>>,
    /// P13: read-only API limit (`TBG_API_PER_IP_MIN`, default 120/min).
    api: HashMap<IpAddr, Vec<Instant>>,
    /// P39: Quick Challenge join attempts (default 30/min).
    invite: HashMap<IpAddr, Vec<Instant>>,
    last_sweep: Option<Instant>,
}

impl Limits {
    /// `true` when the register attempt is allowed. It reserves one slot
    /// while the store works; `refund_register` gives it back when no
    /// account was created, so only created accounts count (BR-P01).
    pub fn check_register(&mut self, ip: IpAddr, max_per_hour: u32, now: Instant) -> bool {
        check_log(
            self.register.entry(ip).or_default(),
            max_per_hour,
            Duration::from_secs(3600),
            now,
        )
    }

    /// Return the slot a refused registration reserved (name taken, store
    /// unavailable): a free name must not be blocked by earlier refusals.
    pub fn refund_register(&mut self, ip: IpAddr) {
        if let Some(log) = self.register.get_mut(&ip) {
            log.pop();
        }
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

    /// `true` when an `/api/*` request is allowed (records it on allow).
    pub fn check_api(&mut self, ip: IpAddr, max_per_min: u32, now: Instant) -> bool {
        check_log(
            self.api.entry(ip).or_default(),
            max_per_min,
            Duration::from_secs(60),
            now,
        )
    }

    /// `true` when a Quick Challenge join is allowed (records it on allow).
    pub fn check_invite(&mut self, ip: IpAddr, max_per_min: u32, now: Instant) -> bool {
        check_log(
            self.invite.entry(ip).or_default(),
            max_per_min,
            Duration::from_secs(60),
            now,
        )
    }

    /// Drop expired entries and empty logs; at most once a minute.
    pub fn sweep(&mut self, now: Instant) {
        if self
            .last_sweep
            .is_some_and(|t| now - t < Duration::from_secs(60))
        {
            return;
        }
        self.last_sweep = Some(now);
        // Prune expired entries first: an IP seen once and never again
        // would otherwise keep a one-entry log forever.
        prune(&mut self.register, Duration::from_secs(3600), now);
        prune(&mut self.login, Duration::from_secs(60), now);
        prune(&mut self.api, Duration::from_secs(60), now);
        prune(&mut self.invite, Duration::from_secs(60), now);
    }
}

fn prune(logs: &mut HashMap<IpAddr, Vec<Instant>>, window: Duration, now: Instant) {
    logs.retain(|_, v| {
        v.retain(|t| now.saturating_duration_since(*t) < window);
        !v.is_empty()
    });
}

fn check_log(log: &mut Vec<Instant>, max: u32, window: Duration, now: Instant) -> bool {
    log.retain(|t| now.saturating_duration_since(*t) < window);
    if log.len() >= max as usize {
        return false;
    }
    log.push(now);
    true
}
