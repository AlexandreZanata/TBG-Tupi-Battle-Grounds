//! Bridge browser-cookie operations into the same epoch-checked client coordinator.

use super::{
    types::{parse_grant, public_error, Failure, Outcome},
    Kind, ResultSlot,
};
use serde_json::{json, Value};
use wasm_bindgen::{closure::Closure, prelude::*};

#[wasm_bindgen(module = "/assets/auth/session.js")]
extern "C" {
    #[wasm_bindgen(catch,js_name=startAuth)]
    fn start_auth(request: &str, callback: &JsValue) -> Result<(), JsValue>;
    #[wasm_bindgen(js_name=invalidateEpoch)]
    fn invalidate_epoch(epoch: f64);
}

pub(super) fn invalidate(epoch: u64) {
    invalidate_epoch(epoch as f64);
}

pub(super) fn start(
    kind: Kind,
    input: Option<String>,
    origin: String,
    epoch: u64,
    remember: bool,
    slot: ResultSlot,
) {
    let name = match kind {
        Kind::Restore => "restore",
        Kind::Login => "login",
        Kind::Register => "register",
        Kind::Recover => "recover",
        Kind::Refresh => "refresh",
        Kind::Backup => "backup",
        Kind::Logout => "logout",
        // Resume records are native-only; the browser keeps them per tab.
        Kind::Resume => {
            if let Ok(mut slot) = slot.lock() {
                *slot = Some(Err(Failure::Incompatible));
            }
            return;
        }
    };
    let generation = super::slot().map(String::from);
    let request = json!({"kind":name,"input":input,"origin":origin,"epoch":epoch,"remember":remember,"slot":generation})
        .to_string();
    let output = slot.clone();
    let callback = Closure::once_into_js(move |raw: String| {
        let result = parse(&raw);
        if let Ok(mut slot) = output.lock() {
            *slot = Some(result);
        }
    });
    if start_auth(&request, &callback).is_err() {
        if let Ok(mut slot) = slot.lock() {
            *slot = Some(Err(Failure::Unavailable));
        }
    }
}

fn parse(raw: &str) -> Result<Outcome, Failure> {
    if raw.len() > 8192 {
        return Err(Failure::Incompatible);
    }
    let value: Value = serde_json::from_str(raw).map_err(|_| Failure::Incompatible)?;
    match value.get("kind").and_then(Value::as_str) {
        Some("guest") => Ok(Outcome::Guest {
            legacy: value
                .get("legacy")
                .and_then(Value::as_str)
                .filter(|v| tbg_core::account::parse_key(v).is_some())
                .map(str::to_owned),
        }),
        Some("signed_in") => Ok(Outcome::SignedIn {
            grant: parse_grant(
                value.get("grant").ok_or(Failure::Incompatible)?.clone(),
                false,
            )?,
            secure: value.get("secure").and_then(Value::as_bool) == Some(true),
            device: value
                .get("device")
                .and_then(Value::as_str)
                .filter(|s| super::types::valid_hex(s, 16))
                .ok_or(Failure::Incompatible)?
                .to_owned(),
        }),
        Some("backup") => Ok(Outcome::Backup {
            migrated: value.get("migrated").and_then(Value::as_bool) == Some(true),
        }),
        Some("logged_out") => Ok(Outcome::LoggedOut),
        Some("logout_pending") => Ok(Outcome::LogoutPending(public_error(&value))),
        _ => Err(public_error(&value)),
    }
}
