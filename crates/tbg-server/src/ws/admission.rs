//! Who may open a socket: the client IP (trusting a local proxy's
//! `X-Forwarded-For`) and the global / per-IP connection caps.

use std::net::{IpAddr, SocketAddr};
use std::sync::atomic::Ordering::Relaxed;

use axum::http::HeaderMap;

use crate::AppState;

/// Behind a local reverse proxy (Caddy), trust its `X-Forwarded-For`.
pub(super) fn client_ip(peer: SocketAddr, headers: &HeaderMap) -> IpAddr {
    let forwarded = headers
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.split(',').next())
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
