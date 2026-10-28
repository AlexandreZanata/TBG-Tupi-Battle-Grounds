//! Who may open a socket: the client IP (trusting a local proxy's
//! `X-Forwarded-For`) and the global / per-IP connection caps.

use std::net::{IpAddr, SocketAddr};
use std::sync::atomic::Ordering::Relaxed;

use axum::http::HeaderMap;

use crate::AppState;

/// Behind a local reverse proxy (Caddy), trust its `X-Forwarded-For`.
/// Shared with the read-only API rate limiter.
pub(crate) fn client_ip(peer: SocketAddr, headers: &HeaderMap) -> IpAddr {
    let forwarded = headers
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        // The last entry is the one our own proxy added; earlier entries
        // come from the client and can be forged.
        .and_then(|s| s.rsplit(',').next())
        .and_then(|s| s.trim().parse().ok());
    match forwarded {
        Some(ip) if peer.ip().is_loopback() => ip,
        _ => peer.ip(),
    }
}

/// Counts this socket against the global and per-IP caps while alive.
pub(super) struct Slot {
    app: AppState,
    ip: IpAddr,
}

impl Slot {
    pub(super) fn acquire(app: &AppState, ip: IpAddr) -> Option<Slot> {
        let mut ips = app.ips.lock().ok()?;
        let per_ip = ips.entry(ip).or_insert(0);
        let total = app.metrics.connections.load(Relaxed) as usize;
        if *per_ip >= app.config.max_per_ip || total >= app.config.max_connections {
            return None;
        }
        *per_ip += 1;
        app.metrics.connections.fetch_add(1, Relaxed);
        Some(Slot {
            app: app.clone(),
            ip,
        })
    }
}

impl Drop for Slot {
    fn drop(&mut self) {
        self.app.metrics.connections.fetch_sub(1, Relaxed);
        if let Ok(mut ips) = self.app.ips.lock() {
            if let Some(n) = ips.get_mut(&self.ip) {
                *n -= 1;
                if *n == 0 {
                    ips.remove(&self.ip);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn headers(xff: &str) -> HeaderMap {
        let mut h = HeaderMap::new();
        h.insert("x-forwarded-for", xff.parse().unwrap());
        h
    }

    #[test]
    fn forwarded_ip_trusted_only_from_local_proxy() {
        let local: SocketAddr = "127.0.0.1:40000".parse().unwrap();
        let remote: SocketAddr = "203.0.113.9:40000".parse().unwrap();
        let spoofed = headers("1.1.1.1, 198.51.100.7");
        assert_eq!(client_ip(local, &spoofed).to_string(), "198.51.100.7");
        assert_eq!(client_ip(remote, &spoofed).to_string(), "203.0.113.9");
        assert_eq!(client_ip(local, &HeaderMap::new()).to_string(), "127.0.0.1");
    }
}
