//! Explicit account-key backup in the OS vault, independent of rotating session tokens.

use super::{
    native::Work,
    native_store::{nonce, Store},
    transport,
    types::{parse_grant, Failure, Outcome},
};
use crate::platform::credentials;
use serde_json::json;
use std::sync::atomic::Ordering;

fn matches_account(key: &str, account: &str) -> bool {
    tbg_core::account::parse_key(key).is_some_and(|(name, _)| name.eq_ignore_ascii_case(account))
}

pub(super) fn run(work: &Work, input: Option<&str>) -> Result<Outcome, Failure> {
    let account = work.account.as_deref().ok_or(Failure::NoSession)?;
    if work.access.is_none() {
        return Err(Failure::NoSession);
    }
    let store = Store::open(&work.origin)?;
    let namespace = format!("{}:recovery-key", store.namespace(&work.origin, account)?);
    let key = if let Some(key) = input {
        if !matches_account(key, account) {
            return Err(Failure::InvalidKey);
        }
        // Verify the supplied manual key using the existing native endpoint.
        // This short-lived proof never claims a socket/controller or replaces
        // the current session, and is revoked before writing the backup.
        let proof = parse_grant(
            transport::post(
                &work.origin,
                "login",
                json!({
                    "account_key":key,"device_id":nonce(16)?,"persistent":false
                }),
                None,
            )?,
            true,
        )?;
        transport::post(
            &work.origin,
            "logout",
            json!({"refresh_token":proof.refresh}),
            Some(proof.access),
        )?;
        if !proof.profile.name.eq_ignore_ascii_case(account) {
            return Err(Failure::InvalidKey);
        }
        if work.current_epoch.load(Ordering::Acquire) != work.epoch {
            return Err(Failure::Cancelled);
        }
        credentials::write(&namespace, key).map_err(|_| Failure::StorageUnavailable)?;
        Some(key.to_owned())
    } else {
        match credentials::read(&namespace) {
            Ok(key) if matches_account(&key, account) => Some(key),
            Ok(_) => return Err(Failure::InvalidKey),
            Err(credentials::Error::Missing) => {
                super::native::legacy(&work.origin).filter(|key| matches_account(key, account))
            }
            Err(_) => return Err(Failure::StorageUnavailable),
        }
    };
    Ok(Outcome::RecoveryKey { key })
}
