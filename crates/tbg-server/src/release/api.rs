//! P66 release endpoints: `/api/admission` tells a client which slot path
//! accepts its exact match contract (or that it must update), `/ready` is
//! readiness for new admissions (a draining slot is alive, not ready), and
//! loopback-only `/api/release` shows the slot's state for operators.

use std::net::SocketAddr;

use axum::extract::{ConnectInfo, RawQuery, State};
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;

use crate::store::release::{now_unix, SlotIdentity, SlotRow};
use crate::store::release_admin::retire_blockers;
use crate::AppState;

/// Where a client with a given contract may start new play.
#[derive(Debug, PartialEq, Eq)]
pub enum Decision {
    Admit {
        endpoint: String,
        slot: String,
        generation: String,
    },
    UpdateRequired {
        contract: String,
    },
    Unavailable,
}

/// Choose the slot for `client` (P66 §6.2/§6.4, BR-UP09). Only fresh,
/// admitting slots with the exact contract qualify; a same-contract canary
/// takes buckets below its percentage. Without a directory (single
/// server) the process itself answers.
pub fn decide(
    rows: &[SlotRow],
    own: Option<&SlotIdentity>,
    own_contract: &str,
    client: &str,
    bucket: u8,
    now: i64,
) -> Decision {
    let live: Vec<&SlotRow> = rows.iter().filter(|r| r.fresh(now) && r.admits()).collect();
    if own.is_none() {
        return if client == own_contract {
            Decision::Admit {
                endpoint: "/ws".into(),
                slot: String::new(),
                generation: String::new(),
            }
        } else {
            Decision::UpdateRequired {
                contract: own_contract.into(),
            }
        };
    }
    let fits: Vec<&&SlotRow> = live.iter().filter(|r| r.contract == client).collect();
    let active = fits.iter().find(|r| r.state == "active");
    let canary = fits.iter().find(|r| r.state == "canary");
    let pick = match (active, canary) {
        (Some(a), Some(c)) => Some(if bucket < c.canary_pct { c } else { a }),
        (Some(a), None) => Some(a),
        (None, Some(c)) => Some(c),
        (None, None) => None,
    };
    match pick {
        Some(r) => Decision::Admit {
            endpoint: r.endpoint.clone(),
            slot: r.slot.clone(),
            generation: r.generation.clone(),
        },
        None => match live.iter().find(|r| r.state == "active").or(live.first()) {
            Some(r) => Decision::UpdateRequired {
                contract: r.contract.clone(),
            },
            None => Decision::Unavailable,
        },
    }
}

/// `contract`, `build`, `outcome` and `bucket` query values; `None` for a
/// missing or malformed contract (anything else just falls back).
fn params(raw: Option<String>) -> Option<(String, String, String, u8)> {
    let (mut contract, mut build, mut outcome, mut bucket) =
        (None, String::new(), String::new(), None);
    for pair in raw.unwrap_or_default().split('&') {
        let (k, v) = pair.split_once('=').unwrap_or((pair, ""));
        match k {
            "contract" => contract = tbg_core::contract::MatchContract::parse(v).map(|c| c.id()),
            "build" => build = v.chars().take(64).collect(),
            "outcome" => outcome = v.chars().take(64).collect(),
            "bucket" => bucket = v.parse::<u8>().ok().filter(|b| *b < 100),
            _ => {}
        }
    }
    let bucket = bucket.unwrap_or_else(|| (now_unix().rem_euclid(100)) as u8);
    Some((contract?, build, outcome, bucket))
}

pub(crate) async fn admission(
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    State(app): State<AppState>,
    RawQuery(raw): RawQuery,
) -> Response {
    let ip = crate::ws::client_ip(peer, &headers);
    if crate::api::limited(&app, ip) {
        return StatusCode::TOO_MANY_REQUESTS.into_response();
    }
    let Some((contract, build, outcome, bucket)) = params(raw) else {
        return StatusCode::BAD_REQUEST.into_response();
    };
    if !build.is_empty() {
        let outcome = if outcome.is_empty() {
            "none"
        } else {
            outcome.as_str()
        };
        app.metrics.record_update_outcome(&build, outcome);
    }
    let rt = &app.release;
    let rows = rt.directory.borrow().clone();
    let decision = decide(
        &rows,
        rt.identity.as_ref(),
        &rt.contract.id(),
        &contract,
        bucket,
        now_unix(),
    );
    let counter = match &decision {
        Decision::Admit { .. } => &app.metrics.admit_ok,
        Decision::UpdateRequired { .. } => &app.metrics.admit_update_required,
        Decision::Unavailable => &app.metrics.admit_unavailable,
    };
    counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let body = match decision {
        Decision::Admit {
            endpoint,
            slot,
            generation,
        } => {
            json!({"status": "ok", "endpoint": endpoint, "slot": slot, "generation": generation})
        }
        Decision::UpdateRequired { contract } => {
            json!({"status": "update_required", "contract": contract})
        }
        Decision::Unavailable => json!({"status": "unavailable"}),
    };
    ([(header::CACHE_CONTROL, "no-store")], Json(body)).into_response()
}

/// Readiness: 200 only while this process may admit new matches.
pub(crate) async fn ready(State(app): State<AppState>) -> Response {
    if app.release.gate.admits() {
        (StatusCode::OK, "ready").into_response()
    } else {
        (StatusCode::SERVICE_UNAVAILABLE, "not admitting").into_response()
    }
}

/// Operators only (loopback, not proxied): identity, gate, load, the
/// directory and what still blocks this slot's retirement.
pub(crate) async fn status(
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    State(app): State<AppState>,
) -> Response {
    if !peer.ip().is_loopback() || headers.contains_key("x-forwarded-for") {
        return StatusCode::NOT_FOUND.into_response();
    }
    let rt = &app.release;
    let load = super::Runtime::load(&app.metrics, app.store.as_deref());
    let rows = rt.directory.borrow().clone();
    let own = rt.identity.as_ref().and_then(|i| {
        rows.iter()
            .find(|r| r.slot == i.slot && r.generation == i.generation)
    });
    let mut blockers = own.map(retire_blockers).unwrap_or_default();
    if load.matches_running > 0 || load.results_pending > 0 {
        blockers.push(format!(
            "live load: {} running, {} results pending",
            load.matches_running, load.results_pending
        ));
    }
    let gate = &rt.gate;
    let relaxed = std::sync::atomic::Ordering::Relaxed;
    Json(json!({
        "slot": rt.slot(),
        "generation": rt.identity.as_ref().map(|i| i.generation.clone()),
        "server_build": rt.identity.as_ref().map(|i| i.server_build.clone()),
        "contract": rt.contract.id(),
        "admitting": gate.admits(),
        "ranked": gate.ranked_open(),
        "shutdown": gate.shutdown.load(relaxed),
        "load": {
            "matches_running": load.matches_running,
            "results_pending": load.results_pending,
            "ranked_inflight": load.ranked_inflight,
        },
        "retire_blockers": blockers,
        "directory": rows.iter().map(|r| json!({
            "slot": r.slot, "generation": r.generation, "state": r.state,
            "contract": r.contract, "canary_pct": r.canary_pct,
            "ranked_authority": r.ranked_authority, "fresh": r.fresh(now_unix()),
            "matches_running": r.matches_running, "results_pending": r.results_pending,
        })).collect::<Vec<_>>(),
    }))
    .into_response()
}

#[cfg(test)]
mod tests;
