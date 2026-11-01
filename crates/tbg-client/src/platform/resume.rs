//! Native resume record (P66 BR-UP03/U11): its format, freshness and pending
//! writes. Only the auth coordinator touches the vault, under its per-origin
//! process lock (D-97), so restore and persistence have a single owner.

use super::storage::{self, Scope};
use serde_json::{json, Value};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const FRESH_SECS: u64 = 120;
/// How long the menu waits for the vault before a visible temporary session.
pub const RESTORE_WAIT: Duration = Duration::from_secs(5);
const TOUCH_INTERVAL: Duration = Duration::from_secs(10);

/// Session-scope keys the network layer already reads and writes.
const TOKEN: &str = "tbg_token";
const ENDPOINT: &str = "tbg_endpoint";
const FAMILY: &str = "tbg_resume_family";
const ORIGIN: &str = "tbg_resume_origin";

#[derive(Clone, Debug, PartialEq, Eq)]
struct Record {
    token: [u8; 16],
    endpoint: String,
    /// The account family the handle is bound to; `None` for a guest.
    family: Option<String>,
    origin: String,
    saved: u64,
}

struct State {
    ready: bool,
    available: bool,
    /// A coalesced vault write (`Some`) or removal (`None`) for the coordinator.
    pending: Option<Option<String>>,
    touched: Instant,
    started: Instant,
}

fn state() -> &'static Mutex<State> {
    static STATE: OnceLock<Mutex<State>> = OnceLock::new();
    STATE.get_or_init(|| {
        Mutex::new(State {
            ready: false,
            available: false,
            pending: None,
            touched: Instant::now(),
            started: Instant::now(),
        })
    })
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |v| v.as_secs())
}

fn endpoint_valid(value: &str) -> bool {
    value == "/ws"
        || value
            .strip_prefix("/ws/slot-")
            .is_some_and(|v| v.len() == 1 && v.bytes().all(|b| b.is_ascii_lowercase()))
}

fn family_valid(value: &str) -> bool {
    value.len() == 32
        && value
            .bytes()
            .all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'))
}

/// The vault record is bound to this origin and installation (`context`).
pub fn context() -> String {
    format!(
        "{}|{}",
        super::http::origin(&crate::config::server_url()).unwrap_or_default(),
        storage::data_dir().unwrap_or_default().display()
    )
}

fn decode(raw: &str, context: &str, at: u64) -> Option<Record> {
    if raw.len() > 4096 {
        return None;
    }
    let value: Value = serde_json::from_str(raw).ok()?;
    if value.get("schema")?.as_u64()? != 2 || value.get("context")?.as_str()? != context {
        return None;
    }
    let saved = value.get("saved")?.as_u64()?;
    if saved > at || at - saved > FRESH_SECS {
        return None;
    }
    let family = match value.get("family")? {
        Value::Null => None,
        Value::String(f) if family_valid(f) => Some(f.clone()),
        _ => return None,
    };
    let origin = value.get("origin")?.as_str()?.to_owned();
    if !context.starts_with(&format!("{origin}|")) {
        return None;
    }
    let endpoint = value.get("endpoint")?.as_str()?.to_owned();
    endpoint_valid(&endpoint).then_some(Record {
        token: crate::config::parse_invite_token(value.get("token")?.as_str()?)?,
        endpoint,
        family,
        origin,
        saved,
    })
}

fn encode(record: &Record, context: &str) -> String {
    json!({
        "schema": 2,
        "context": context,
        "token": crate::config::hex16(&record.token),
        "endpoint": record.endpoint,
        "family": record.family,
        "origin": record.origin,
        "saved": record.saved,
    })
    .to_string()
}

/// The live session's handle, as the network layer last stored it.
fn current() -> Option<Record> {
    let endpoint = storage::get(Scope::Session, ENDPOINT).filter(|e| endpoint_valid(e))?;
    Some(Record {
        token: crate::config::parse_invite_token(&storage::get(Scope::Session, TOKEN)?)?,
        endpoint,
        family: storage::get(Scope::Session, FAMILY).filter(|f| family_valid(f)),
        origin: storage::get(Scope::Session, ORIGIN)?,
        saved: now(),
    })
}

/// The coordinator's restore result: `Ok(None)` when no record exists,
/// `Err` when the vault is unavailable. A reply after the visible deadline
/// changes nothing: the temporary session already started.
pub fn restored(read: Result<Option<String>, ()>) {
    let Ok(mut state) = state().lock() else {
        return;
    };
    if state.ready {
        return;
    }
    state.ready = true;
    state.available = read.is_ok();
    let context = context();
    let Some(record) = read
        .ok()
        .flatten()
        .and_then(|raw| decode(&raw, &context, now()))
    else {
        return;
    };
    storage::set(Scope::Session, TOKEN, &crate::config::hex16(&record.token));
    storage::set(Scope::Session, ENDPOINT, &record.endpoint);
    storage::set(Scope::Session, ORIGIN, &record.origin);
    match &record.family {
        Some(family) => {
            storage::set(Scope::Session, FAMILY, family);
        }
        None => storage::remove(Scope::Session, FAMILY),
    }
}

/// The time left for the restore's vault read.
pub fn restore_budget() -> Duration {
    state().lock().map_or(Duration::ZERO, |s| {
        RESTORE_WAIT.saturating_sub(s.started.elapsed())
    })
}

pub fn ready() -> bool {
    let Ok(mut state) = state().lock() else {
        return true;
    };
    if !state.ready && state.started.elapsed() >= RESTORE_WAIT {
        state.ready = true;
        state.available = false;
    }
    state.ready
}

pub fn available() -> bool {
    state().lock().is_ok_and(|s| s.ready && s.available)
}

/// The network layer stored a new handle, slot or family: persist it.
pub fn note_change() {
    let Ok(mut state) = state().lock() else {
        return;
    };
    if !(state.ready && state.available) {
        return;
    }
    if let Some(record) = current() {
        state.pending = Some(Some(encode(&record, &context())));
        state.touched = Instant::now();
    }
}

/// Keep the record fresh while a match runs (a crash may resume it).
pub fn touch() {
    let due = state()
        .lock()
        .is_ok_and(|s| s.touched.elapsed() >= TOUCH_INTERVAL);
    if due {
        note_change();
    }
}

/// An identity change or sign-out: the old handle must not survive a restart.
pub fn clear() {
    if let Ok(mut state) = state().lock() {
        if state.ready && state.available {
            state.pending = Some(None);
        }
    }
}

/// The coordinator takes the latest pending write when it is idle.
pub fn take_pending() -> Option<Option<String>> {
    state().lock().ok()?.pending.take()
}

/// The coordinator's write result. A busy lock keeps the write for later
/// (unless a newer one arrived); any other failure turns persistence off
/// for this process, never into a plaintext fallback.
pub fn persisted(write: Option<String>, result: Result<(), bool>) {
    if let Ok(mut state) = state().lock() {
        match result {
            Ok(()) => {}
            Err(true) => {
                state.pending.get_or_insert(write);
            }
            Err(false) => {
                state.available = false;
                state.pending = None;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn record(family: Option<&str>) -> Record {
        Record {
            token: [7; 16],
            endpoint: "/ws/slot-a".into(),
            family: family.map(str::to_owned),
            origin: "https://a.test".into(),
            saved: 10,
        }
    }

    #[test]
    fn br_up03_resume_is_bound_to_origin_installation_freshness_and_slot() {
        let raw = encode(&record(None), "https://a.test|install-a");
        assert_eq!(
            decode(&raw, "https://a.test|install-a", 130),
            Some(record(None))
        );
        assert!(decode(&raw, "https://b.test|install-a", 10).is_none());
        assert!(decode(&raw, "https://a.test|install-b", 10).is_none());
        assert!(decode(&raw, "https://a.test|install-a", 131).is_none());
        assert!(decode(&raw, "https://a.test|install-a", 9).is_none());
        for bad in [
            "wss://evil.test/ws",
            "/ws/slot-a?token=secret",
            "/ws/slot-aa",
        ] {
            assert!(!endpoint_valid(bad));
        }
    }

    #[test]
    fn br_as06_restored_account_handles_keep_their_family_binding() {
        let family = "0123456789abcdef0123456789abcdef";
        let raw = encode(&record(Some(family)), "https://a.test|install-a");
        let back = decode(&raw, "https://a.test|install-a", 20).unwrap();
        assert_eq!(back.family.as_deref(), Some(family));
        // A record naming another origin than its context, or a malformed
        // family, is never published into the session.
        let foreign = raw.replace(
            "\"origin\":\"https://a.test\"",
            "\"origin\":\"https://b.test\"",
        );
        assert!(decode(&foreign, "https://a.test|install-a", 20).is_none());
        let malformed = raw.replace(family, "not-a-family");
        assert!(decode(&malformed, "https://a.test|install-a", 20).is_none());
        // P66 schema-1 records carry no family binding; they are not trusted now.
        let legacy = raw.replace("\"schema\":2", "\"schema\":1");
        assert!(decode(&legacy, "https://a.test|install-a", 20).is_none());
    }
}
