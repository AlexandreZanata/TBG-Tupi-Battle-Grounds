//! Snapshot-only directory filtering and bounded federation of the two trusted local slots.

use crate::{
    live::{Counts, LiveSnapshot, OnlineRow},
    store::release::{now_unix, SlotRow},
    AppState,
};
use axum::{
    extract::{ConnectInfo, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, net::SocketAddr, sync::Arc, time::Duration};
use tokio::sync::watch;

pub struct DirectoryQuery {
    pub offset: usize,
    pub limit: usize,
    q: String,
    available: bool,
}
impl DirectoryQuery {
    pub fn parse(raw: Option<&str>) -> Result<Self, &'static str> {
        let mut p = Self {
            offset: 0,
            limit: 50,
            q: String::new(),
            available: false,
        };
        let raw = raw.unwrap_or("");
        if raw.len() > 256 {
            return Err("query");
        }
        let mut seen = std::collections::BTreeSet::new();
        for (k, v) in url::form_urlencoded::parse(raw.as_bytes()) {
            if !seen.insert(k.to_string()) {
                return Err("duplicate");
            }
            match k.as_ref() {
                "offset" => p.offset = v.parse::<usize>().map_err(|_| "offset")?.min(100_000),
                "limit" => p.limit = v.parse::<usize>().map_err(|_| "limit")?.clamp(1, 100),
                "q" if v.len() <= 24
                    && v.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_') =>
                {
                    p.q = v.to_ascii_lowercase()
                }
                "available" if v == "0" || v == "1" => p.available = v == "1",
                _ => return Err("filter"),
            }
        }
        Ok(p)
    }
    pub fn matches(&self, row: &OnlineRow) -> bool {
        row.name.to_ascii_lowercase().contains(&self.q) && (!self.available || row.invitable)
    }
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LocalPresence {
    pub slot: Option<String>,
    pub generation: Option<String>,
    pub contract: String,
    pub admitting: bool,
    pub updated_unix: i64,
    pub revision: u64,
    pub counts: Counts,
    pub accounts: Vec<OnlineRow>,
}

pub(crate) async fn internal(
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    State(app): State<AppState>,
) -> Response {
    if !peer.ip().is_loopback()
        || [
            "forwarded",
            "x-forwarded-for",
            "x-forwarded-host",
            "x-real-ip",
        ]
        .iter()
        .any(|h| headers.contains_key(*h))
    {
        return StatusCode::NOT_FOUND.into_response();
    }
    let local = local(&app.local_live.borrow(), &app.release);
    if serde_json::to_vec(&local).map_or(true, |v| v.len() > 2 * 1024 * 1024) {
        return StatusCode::SERVICE_UNAVAILABLE.into_response();
    }
    (
        [(axum::http::header::CACHE_CONTROL, "no-store")],
        Json(local),
    )
        .into_response()
}

fn local(s: &LiveSnapshot, rt: &crate::release::Runtime) -> LocalPresence {
    LocalPresence {
        slot: rt.slot().map(str::to_owned),
        generation: rt.identity.as_ref().map(|i| i.generation.clone()),
        contract: rt.contract.id(),
        admitting: rt.gate.admits(),
        updated_unix: s.updated_unix,
        revision: s.revision,
        counts: s.counts,
        accounts: s.accounts.clone(),
    }
}

/// Fixed loopback addresses match the existing Caddy slot configuration, never directory URLs.
fn fetch(slot: &str, ports: [u16; 2]) -> Option<LocalPresence> {
    let port = match slot {
        "a" => ports[0],
        "b" => ports[1],
        _ => return None,
    };
    let agent = ureq::Agent::config_builder()
        .proxy(None)
        .max_redirects(0)
        .timeout_global(Some(Duration::from_millis(750)))
        .build()
        .new_agent();
    let mut response = agent
        .get(format!("http://127.0.0.1:{port}/api/internal/presence"))
        .call()
        .ok()?;
    let bytes = response
        .body_mut()
        .with_config()
        .limit(2 * 1024 * 1024)
        .read_to_vec()
        .ok()?;
    let snapshot: LocalPresence = serde_json::from_slice(&bytes).ok()?;
    if snapshot.accounts.len() > 2000
        || snapshot
            .accounts
            .iter()
            .any(|r| !tbg_core::account::valid_name_shape(&r.name))
    {
        return None;
    }
    Some(snapshot)
}

/// Pure merge used by the background worker and regression fixtures. Other-slot rows
/// remain visible; the UI compares their ownership domain before offering Challenge.
pub fn merge(
    local: &LiveSnapshot,
    own: &LocalPresence,
    directory: &[SlotRow],
    peers: &[LocalPresence],
    now: i64,
) -> LiveSnapshot {
    let mut out = local.clone();
    let mut names = BTreeMap::new();
    let mut counts = Counts::default();
    let mut complete = !directory.is_empty();
    let mut stale = false;
    for row in directory
        .iter()
        .filter(|r| !matches!(r.state.as_str(), "retired" | "failed"))
    {
        let snapshot = if own.slot.as_deref() == Some(row.slot.as_str()) {
            Some(own)
        } else {
            peers.iter().find(|p| {
                p.slot.as_deref() == Some(row.slot.as_str())
                    && p.generation.as_deref() == Some(row.generation.as_str())
            })
        };
        let Some(p) = snapshot.filter(|p| {
            p.generation.as_deref() == Some(row.generation.as_str())
                && now.saturating_sub(p.updated_unix) <= 15
        }) else {
            complete = false;
            continue;
        };
        let fresh = row.fresh(now) && now.saturating_sub(p.updated_unix) <= 5;
        complete &= fresh;
        stale |= !fresh;
        counts.players = counts.players.saturating_add(p.counts.players);
        counts.in_battle = counts.in_battle.saturating_add(p.counts.in_battle);
        counts.watching = counts.watching.saturating_add(p.counts.watching);
        for account in &p.accounts {
            let mut a = account.clone();
            a.slot = p.slot.clone();
            a.invitable &= fresh && row.admits() && p.admitting && p.contract == own.contract;
            let key = a.name.to_ascii_lowercase();
            // Prefer the fresher controller; lexical slot breaks equal-time ties.
            let candidate = (p.updated_unix, row.slot.clone(), a);
            if names
                .get(&key)
                .is_none_or(|old: &(i64, String, OnlineRow)| {
                    (candidate.0, &candidate.1) > (old.0, &old.1)
                })
            {
                names.insert(key, candidate);
            }
        }
    }
    // Startup or directory failure never erases the process's known connections.
    if directory.is_empty() {
        counts = own.counts;
        for a in &own.accounts {
            let mut a = a.clone();
            a.invitable = false;
            names.insert(
                a.name.to_ascii_lowercase(),
                (own.updated_unix, String::new(), a),
            );
        }
    }
    out.accounts = names.into_values().map(|(_, _, a)| a).collect();
    out.accounts
        .sort_by(|a, b| b.rating.cmp(&a.rating).then_with(|| a.name.cmp(&b.name)));
    counts.accounts = out.accounts.len() as u32;
    out.counts = counts;
    out.complete = complete;
    out.stale = stale;
    out
}

pub(crate) fn start(
    rt: Arc<crate::release::Runtime>,
    source: watch::Receiver<Arc<LiveSnapshot>>,
    ports: [u16; 2],
) -> watch::Receiver<Arc<LiveSnapshot>> {
    if rt.identity.is_none() {
        return source;
    }
    let (tx, rx) = watch::channel(source.borrow().clone());
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(1));
        let mut peers: BTreeMap<String, LocalPresence> = BTreeMap::new();
        let mut revision = 0u64;
        loop {
            interval.tick().await;
            let rows = rt.directory.borrow().clone();
            let mut tasks = tokio::task::JoinSet::new();
            for row in rows
                .iter()
                .filter(|r| {
                    Some(r.slot.as_str()) != rt.slot()
                        && !matches!(r.state.as_str(), "retired" | "failed")
                })
                .take(2)
            {
                let slot = row.slot.clone();
                tasks.spawn_blocking(move || (slot.clone(), fetch(&slot, ports)));
            }
            while let Some(Ok((slot, snapshot))) = tasks.join_next().await {
                if let Some(snapshot) = snapshot {
                    peers.insert(slot, snapshot);
                }
            }
            peers.retain(|slot, p| {
                rows.iter().any(|r| {
                    &r.slot == slot
                        && p.generation.as_ref() == Some(&r.generation)
                        && !matches!(r.state.as_str(), "retired" | "failed")
                }) && now_unix().saturating_sub(p.updated_unix) <= 15
            });
            let local_snapshot = source.borrow().clone();
            let own = local(&local_snapshot, &rt);
            let mut snapshot = merge(
                &local_snapshot,
                &own,
                &rows,
                &peers.values().cloned().collect::<Vec<_>>(),
                now_unix(),
            );
            {
                let old = tx.borrow();
                if old.counts != snapshot.counts
                    || old.accounts != snapshot.accounts
                    || old.complete != snapshot.complete
                    || old.stale != snapshot.stale
                {
                    revision = revision.saturating_add(1);
                }
            }
            snapshot.revision = revision;
            tx.send_replace(Arc::new(snapshot));
        }
    });
    rx
}
