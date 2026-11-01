//! Bounded callback HTTP JSON requests, polled by the Bevy interface.

use std::sync::{Arc, Mutex};

/// One result slot per request; no network work blocks Bevy's update loop.
pub struct Request(Arc<Mutex<Option<Result<serde_json::Value, String>>>>);

impl Request {
    pub fn get(path: &str) -> Self {
        let slot = Arc::new(Mutex::new(None));
        let result = slot.clone();
        let Some(origin) = origin(&crate::config::server_url()) else {
            *slot.lock().unwrap_or_else(|e| e.into_inner()) =
                Some(Err("Invalid server URL".into()));
            return Self(slot);
        };
        ehttp::fetch(
            ehttp::Request::get(format!("{origin}{path}")),
            move |response| {
                let parsed = response.and_then(|r| {
                    if !r.ok || r.bytes.len() > 262_144 {
                        return Err("Response unavailable or too large".into());
                    }
                    serde_json::from_slice(&r.bytes).map_err(|_| "Invalid server response".into())
                });
                *result.lock().unwrap_or_else(|e| e.into_inner()) = Some(parsed);
            },
        );
        Self(slot)
    }

    pub fn take(&self) -> Option<Result<serde_json::Value, String>> {
        self.0.lock().ok()?.take()
    }
}

pub fn origin(server: &str) -> Option<String> {
    let (prefix, rest) = if let Some(rest) = server.strip_prefix("wss://") {
        ("https://", rest)
    } else {
        ("http://", server.strip_prefix("ws://")?)
    };
    let host = rest.split('/').next()?;
    if host.is_empty() || host.contains(['@', '?', '#', ' ', '\n']) {
        return None;
    }
    Some(format!("{prefix}{host}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn api_uses_the_websocket_origin_and_preserves_ports() {
        assert_eq!(
            origin("wss://play.example.com/ws"),
            Some("https://play.example.com".into())
        );
        assert_eq!(
            origin("ws://127.0.0.1:8080/ws"),
            Some("http://127.0.0.1:8080".into())
        );
        for bad in [
            "https://bad",
            "ws:///ws",
            "wss://user@host/ws",
            "ws://bad host/ws",
        ] {
            assert_eq!(origin(bad), None)
        }
    }
}
