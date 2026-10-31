//! P66 admission (§6.2/§6.4, BR-UP09): before a new link connects it asks
//! `/api/admission` which fixed slot path accepts this build's exact match
//! contract. A resumed match returns to the endpoint it was admitted on
//! (its original generation). A server without the route, or one that
//! does not answer in time, gets the legacy `/ws` path with the contract
//! named, which the server still checks in the upgrade.

use crate::config;
use crate::platform::http::Request;

/// Wait this long for an admission answer before the legacy fallback.
const ADMISSION_WAIT: f64 = 5.0;

/// This build's match contract, `w<wire>-r<ruleset>-<16 hex>` (the hash
/// comes from `build.rs`; a test pins it to `tbg-core`).
pub fn contract_id() -> String {
    format!(
        "w{}-r{}-{}",
        tbg_core::protocol::PROTOCOL_VERSION,
        tbg_core::ruleset_version::RULESET_VERSION,
        env!("TBG_CONTRACT_SOURCE_HASH")
    )
}

/// Where a link connects.
pub(super) enum Route {
    /// Asked `/api/admission` at `since`; no answer yet.
    Pending { request: Request, since: f64 },
    /// Connect here. `fresh` routes came from admission (new play may be
    /// requested on them); a resume route only reattaches a session.
    Ready {
        url: String,
        endpoint: String,
        fresh: bool,
    },
    /// The admitted contracts no longer include this build.
    UpdateRequired,
}

impl Route {
    /// A resumed session goes back where it was admitted; anything else
    /// asks for admission first.
    pub(super) fn start(resume_endpoint: Option<String>, now: f64) -> Route {
        let server = config::server_url();
        if let Some(endpoint) = resume_endpoint {
            if let Some(url) = ws_url(&server, &endpoint, &contract_id()) {
                return Route::Ready {
                    url,
                    endpoint,
                    fresh: false,
                };
            }
        }
        Route::Pending {
            request: Request::get(&admission_path()),
            since: now,
        }
    }

    /// Resolve a pending admission; `true` when the route just became
    /// ready to connect.
    pub(super) fn advance(&mut self, now: f64) -> bool {
        let Route::Pending { request, since } = self else {
            return false;
        };
        let answer = request.take();
        if answer.is_none() && now - *since < ADMISSION_WAIT {
            return false;
        }
        *self = resolve(answer.and_then(Result::ok), &config::server_url());
        matches!(self, Route::Ready { .. })
    }
}

/// `/api/admission?contract=…&build=…&outcome=…&bucket=…`.
fn admission_path() -> String {
    let update = crate::platform::update_report();
    format!(
        "/api/admission?contract={}&build={}&outcome={}&bucket={}",
        contract_id(),
        update.build,
        update.outcome,
        config::rollout_bucket()
    )
}

/// Turn an admission answer (or its absence) into a route.
pub(super) fn resolve(answer: Option<serde_json::Value>, server: &str) -> Route {
    let contract = contract_id();
    let legacy = || match ws_url(server, "/ws", &contract) {
        Some(url) => Route::Ready {
            url,
            endpoint: "/ws".into(),
            fresh: true,
        },
        None => Route::UpdateRequired,
    };
    let Some(answer) = answer else {
        return legacy();
    };
    match answer.get("status").and_then(|s| s.as_str()) {
        Some("ok") => answer
            .get("endpoint")
            .and_then(|e| e.as_str())
            .filter(|e| valid_endpoint(e))
            .and_then(|e| ws_url(server, e, &contract).map(|url| (url, e.to_string())))
            .map_or_else(legacy, |(url, endpoint)| Route::Ready {
                url,
                endpoint,
                fresh: true,
            }),
        Some("update_required") => Route::UpdateRequired,
        _ => legacy(),
    }
}

/// Only server-chosen slot paths: `/ws` or `/ws/slot-<letter>`.
fn valid_endpoint(endpoint: &str) -> bool {
    endpoint == "/ws"
        || endpoint
            .strip_prefix("/ws/slot-")
            .is_some_and(|s| s.len() == 1 && s.bytes().all(|b| b.is_ascii_lowercase()))
}

/// The WebSocket URL for `endpoint` on the configured server, naming the
/// contract. `None` for a malformed server URL or endpoint.
pub fn ws_url(server: &str, endpoint: &str, contract: &str) -> Option<String> {
    if !valid_endpoint(endpoint) {
        return None;
    }
    let (scheme, rest) = server
        .strip_prefix("wss://")
        .map(|r| ("wss://", r))
        .or_else(|| server.strip_prefix("ws://").map(|r| ("ws://", r)))?;
    let host = rest.split(['/', '?', '#']).next()?;
    if host.is_empty() || host.contains(['@', ' ', '\n']) {
        return None;
    }
    Some(format!("{scheme}{host}{endpoint}?contract={contract}"))
}

#[cfg(test)]
mod tests;
