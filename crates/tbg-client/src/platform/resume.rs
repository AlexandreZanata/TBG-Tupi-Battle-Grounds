//! Asynchronous, origin/installation-bound native resume credentials in the real OS vault.

use super::credentials;
use serde_json::{json, Value};
use std::sync::{mpsc, Arc, Mutex, OnceLock};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const FRESH_SECS: u64 = 120;
const RESTORE_WAIT: Duration = Duration::from_secs(5);
const TOUCH_INTERVAL: Duration = Duration::from_secs(10);

#[derive(Clone)]
struct Record {
    token: Option<[u8; 16]>,
    endpoint: Option<String>,
    saved: u64,
}

struct State {
    ready: bool,
    available: bool,
    record: Record,
    pending: Option<Option<Record>>,
    touched: Instant,
}

struct Service {
    state: Arc<Mutex<State>>,
    wake: mpsc::SyncSender<()>,
    started: Instant,
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

fn decode(raw: &str, context: &str, at: u64) -> Option<Record> {
    if raw.len() > 4096 {
        return None;
    }
    let value: Value = serde_json::from_str(raw).ok()?;
    if value.get("schema")?.as_u64()? != 1 || value.get("context")?.as_str()? != context {
        return None;
    }
    let saved = value.get("saved")?.as_u64()?;
    if saved > at || at - saved > FRESH_SECS {
        return None;
    }
    let token = crate::config::parse_invite_token(value.get("token")?.as_str()?)?;
    let endpoint = value.get("endpoint")?.as_str()?.to_owned();
    endpoint_valid(&endpoint).then_some(Record {
        token: Some(token),
        endpoint: Some(endpoint),
        saved,
    })
}

fn encode(record: &Record, context: &str) -> Option<String> {
    let token = crate::config::hex16(&record.token?);
    let endpoint = record.endpoint.as_deref().filter(|v| endpoint_valid(v))?;
    Some(json!({"schema":1,"context":context,"token":token,"endpoint":endpoint,"saved":record.saved}).to_string())
}

fn namespace(context: &str) -> String {
    // Naming only, not authentication: the full context is verified inside the vault record.
    let hash = context.bytes().fold(0xcbf29ce484222325_u64, |h, b| {
        (h ^ u64::from(b)).wrapping_mul(0x100000001b3)
    });
    format!("p66-resume:{hash:016x}")
}

fn service() -> &'static Service {
    static SERVICE: OnceLock<Service> = OnceLock::new();
    SERVICE.get_or_init(|| {
        let record = Record {
            token: None,
            endpoint: None,
            saved: 0,
        };
        let state = Arc::new(Mutex::new(State {
            ready: false,
            available: false,
            record,
            pending: None,
            touched: Instant::now(),
        }));
        let context = format!(
            "{}|{}",
            super::http::origin(&crate::config::server_url()).unwrap_or_default(),
            super::storage::data_dir().unwrap_or_default().display()
        );
        let worker = state.clone();
        let (wake, receiver) = mpsc::sync_channel(1);
        std::thread::spawn(move || run(worker, receiver, context));
        Service {
            state,
            wake,
            started: Instant::now(),
        }
    })
}

fn run(state: Arc<Mutex<State>>, receiver: mpsc::Receiver<()>, context: String) {
    let key = namespace(&context);
    let initial = credentials::read(&key);
    if let Ok(mut state) = state.lock() {
        // A late vault reply must not replace a new temporary session after timeout.
        if !state.ready {
            state.available = initial.is_ok() || initial == Err(credentials::Error::Missing);
            if let Ok(raw) = initial {
                if let Some(record) = decode(&raw, &context, now()) {
                    state.record = record;
                }
            }
            state.ready = true;
        }
    }
    while receiver.recv().is_ok() {
        let pending = state.lock().ok().and_then(|mut s| s.pending.take());
        let Some(pending) = pending else { continue };
        let result = match pending {
            Some(record) => {
                encode(&record, &context).map_or(Ok(()), |raw| credentials::write(&key, &raw))
            }
            None => credentials::remove(&key),
        };
        if result.is_err() {
            if let Ok(mut state) = state.lock() {
                state.available = false;
            }
        }
    }
}

pub fn initialize() {
    let _ = service();
}

pub fn ready() -> bool {
    let service = service();
    let Ok(mut state) = service.state.lock() else {
        return true;
    };
    if !state.ready && service.started.elapsed() >= RESTORE_WAIT {
        state.ready = true;
        state.available = false;
    }
    state.ready
}

pub fn available() -> bool {
    service().state.lock().is_ok_and(|s| s.ready && s.available)
}

pub fn token() -> Option<[u8; 16]> {
    let state = service().state.lock().ok()?;
    (state.record.saved <= now() && now() - state.record.saved <= FRESH_SECS)
        .then_some(state.record.token)
        .flatten()
}

pub fn endpoint() -> Option<String> {
    token()?;
    service().state.lock().ok()?.record.endpoint.clone()
}

fn change(edit: impl FnOnce(&mut Record)) {
    let service = service();
    if let Ok(mut state) = service.state.lock() {
        edit(&mut state.record);
        state.record.saved = now();
        state.touched = Instant::now();
        if state.available && state.record.token.is_some() && state.record.endpoint.is_some() {
            state.pending = Some(Some(state.record.clone()));
            let _ = service.wake.try_send(());
        }
    }
}

pub fn save_token(token: &[u8; 16]) {
    change(|r| r.token = Some(*token));
}
pub fn save_endpoint(endpoint: &str) {
    if endpoint_valid(endpoint) {
        change(|r| r.endpoint = Some(endpoint.into()));
    }
}

pub fn touch() {
    let due = service()
        .state
        .lock()
        .is_ok_and(|s| s.touched.elapsed() >= TOUCH_INTERVAL);
    if due {
        change(|_| {});
    }
}

pub fn clear() {
    let service = service();
    if let Ok(mut state) = service.state.lock() {
        state.record.token = None;
        state.record.endpoint = None;
        state.pending = Some(None);
        let _ = service.wake.try_send(());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn br_up03_resume_is_bound_to_origin_installation_freshness_and_slot() {
        let record = Record {
            token: Some([7; 16]),
            endpoint: Some("/ws/slot-a".into()),
            saved: 10,
        };
        let raw = encode(&record, "https://a.test|install-a").unwrap();
        assert_eq!(
            decode(&raw, "https://a.test|install-a", 130).unwrap().token,
            record.token
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
}
