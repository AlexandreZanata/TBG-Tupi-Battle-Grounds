//! Bounded auth grants and public failure states; secret-bearing values have no Debug.

use serde_json::Value;
use tbg_core::protocol::{AccessToken, Profile};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Failure {
    Unavailable,
    Incompatible,
    InvalidKey,
    NameTaken,
    NameInvalid,
    RateLimited,
    Conflict,
    RecoveryRequired,
    StorageUnavailable,
    NoSession,
    InMatch,
    Capacity,
    Busy,
    InvalidOrigin,
    Cancelled,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    Guest,
    Restoring,
    Authenticated,
    Refreshing,
    PersistenceUnavailable,
    Conflict,
    ReauthRequired,
    LogoutPending,
}

pub(super) struct Grant {
    pub access: AccessToken,
    pub refresh: Option<String>,
    pub family: String,
    pub profile: Profile,
    pub account_key: Option<String>,
    pub remaining: u64,
    pub persistent: bool,
}

pub(super) fn parse_grant(value: Value, native: bool) -> Result<Grant, Failure> {
    let access = value
        .get("access_token")
        .and_then(Value::as_str)
        .and_then(AccessToken::from_hex)
        .ok_or(Failure::Incompatible)?;
    let family = value
        .get("session_id")
        .and_then(Value::as_str)
        .filter(|v| valid_hex(v, 16))
        .ok_or(Failure::Incompatible)?
        .to_owned();
    let now = value
        .get("server_time")
        .and_then(Value::as_u64)
        .ok_or(Failure::Incompatible)?;
    let until = value
        .get("access_expires_at")
        .and_then(Value::as_u64)
        .ok_or(Failure::Incompatible)?;
    let session = value
        .get("session_expires_at")
        .and_then(Value::as_u64)
        .ok_or(Failure::Incompatible)?;
    if until <= now
        || until > session
        || until - now > 900
        || session.saturating_sub(now) > 2_592_000
    {
        return Err(Failure::Incompatible);
    }
    let profile: Profile =
        serde_json::from_value(value.get("profile").ok_or(Failure::Incompatible)?.clone())
            .map_err(|_| Failure::Incompatible)?;
    if !tbg_core::account::valid_name_shape(&profile.name) {
        return Err(Failure::Incompatible);
    }
    let refresh = match (native, value.get("refresh_token")) {
        (true, Some(Value::String(token))) if valid_hex(token, 32) => Some(token.clone()),
        (false, None) => None,
        _ => return Err(Failure::Incompatible),
    };
    let account_key = match value.get("account_key") {
        Some(Value::String(key)) if tbg_core::account::parse_key(key).is_some() => {
            Some(key.clone())
        }
        None => None,
        _ => return Err(Failure::Incompatible),
    };
    let persistent = value
        .get("persistent")
        .and_then(Value::as_bool)
        .ok_or(Failure::Incompatible)?;
    Ok(Grant {
        persistent,
        access,
        refresh,
        family,
        profile,
        account_key,
        remaining: until - now,
    })
}

pub(super) fn valid_hex(value: &str, bytes: usize) -> bool {
    value.len() == bytes * 2
        && value
            .bytes()
            .all(|v| v.is_ascii_digit() || (b'a'..=b'f').contains(&v))
}

pub(super) fn public_error(value: &Value) -> Failure {
    match value.get("error").and_then(Value::as_str) {
        Some("auth_required") => Failure::InvalidKey,
        Some("storage_unavailable") => Failure::StorageUnavailable,
        Some("no_session") => Failure::NoSession,
        Some("busy") => Failure::Busy,
        Some("cancelled") => Failure::Cancelled,
        Some("name_taken") => Failure::NameTaken,
        Some("name_invalid") => Failure::NameInvalid,
        Some("rate_limited") => Failure::RateLimited,
        Some("conflict") => Failure::Conflict,
        Some("recovery_required") => Failure::RecoveryRequired,
        Some("in_match") => Failure::InMatch,
        Some("capacity" | "family_limit") => Failure::Capacity,
        Some("unavailable" | "revocation_pending") => Failure::Unavailable,
        _ => Failure::Incompatible,
    }
}

pub(super) enum Outcome {
    RecoveryKey {
        key: Option<String>,
    },
    Guest {
        legacy: Option<String>,
    },
    SignedIn {
        grant: Grant,
        secure: bool,
        warning: Option<Failure>,
        device: String,
    },
    Backup {
        migrated: bool,
    },
    LoggedOut,
    LogoutPending(Failure),
    /// P66: the resume record was written or removed (native vault only).
    #[cfg_attr(target_arch = "wasm32", allow(dead_code))]
    Resumed,
}
