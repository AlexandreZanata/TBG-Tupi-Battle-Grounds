//! Opt-in real native-provider acceptance against a synthetic loopback backend only.
use super::{
    native::{self, Operation, Work},
    native_store::{self, Store},
    transport,
    types::{Failure, Outcome},
};
use serde_json::json;
use std::sync::{atomic::AtomicU64, Arc};

pub(super) fn run() -> Result<serde_json::Value, Failure> {
    let origin = transport::origin(&crate::config::server_url())?;
    let url = url::Url::parse(&origin).map_err(|_| Failure::InvalidOrigin)?;
    let local = match url.host() {
        Some(url::Host::Ipv4(ip)) => ip.is_loopback(),
        Some(url::Host::Ipv6(ip)) => ip.is_loopback(),
        Some(url::Host::Domain("localhost")) => true,
        _ => false,
    };
    if !local {
        return Err(Failure::InvalidOrigin);
    }
    let dir = std::env::var_os("TBG_DATA_DIR")
        .map(std::path::PathBuf::from)
        .ok_or(Failure::StorageUnavailable)?;
    if !dir.starts_with(std::env::temp_dir())
        || dir.exists()
            && std::fs::read_dir(&dir)
                .map_err(|_| Failure::StorageUnavailable)?
                .next()
                .is_some()
    {
        return Err(Failure::StorageUnavailable);
    }
    let current = Arc::new(AtomicU64::new(1));
    let mut work = Work {
        origin: origin.clone(),
        operation: Operation::Register(format!("P68{}", native_store::nonce(6)?)),
        remember: true,
        refresh: None,
        family: None,
        account: None,
        device: None,
        access: None,
        epoch: 1,
        current_epoch: current.clone(),
    };
    let Outcome::SignedIn {
        grant,
        secure,
        device,
    } = native::run(work)?
    else {
        return Err(Failure::Incompatible);
    };
    let key = grant.account_key.clone().ok_or(Failure::Incompatible)?;
    let make = |operation| Work {
        origin: origin.clone(),
        operation,
        remember: true,
        refresh: grant.refresh.clone(),
        family: Some(grant.family.clone()),
        account: Some(grant.profile.name.clone()),
        device: Some(device.clone()),
        access: Some(grant.access),
        epoch: 1,
        current_epoch: current.clone(),
    };
    // Model a pre-existing key only in this fresh synthetic test directory.
    use crate::platform::storage::{self, Scope};
    if !storage::set(Scope::Local, "tbg_account_key", &key)
        || !storage::set(Scope::Local, "tbg_auth_legacy_origin", &origin)
    {
        return Err(Failure::StorageUnavailable);
    }
    let lock = Store::open(&origin)?;
    let coordinated = matches!(native::run(make(Operation::Refresh)), Err(Failure::Busy));
    drop(lock);
    let mut rotated = super::types::Grant {
        access: grant.access,
        refresh: grant.refresh.clone(),
        family: grant.family.clone(),
        profile: grant.profile.clone(),
        account_key: None,
        remaining: grant.remaining,
        persistent: grant.persistent,
    };
    let restored = if secure {
        match native::run(make(Operation::Restore))? {
            Outcome::SignedIn {
                grant: restored,
                secure: true,
                ..
            } if restored.family == rotated.family => {
                rotated = restored;
                true
            }
            _ => return Err(Failure::Incompatible),
        }
    } else {
        false
    };
    work = Work {
        origin: origin.clone(),
        operation: Operation::Backup,
        remember: true,
        refresh: rotated.refresh.clone(),
        family: Some(rotated.family.clone()),
        account: Some(rotated.profile.name.clone()),
        device: Some(device),
        access: Some(rotated.access),
        epoch: 1,
        current_epoch: current.clone(),
    };
    let migrated = matches!(native::run(work), Ok(Outcome::Backup { migrated: true }));
    let key_preserved = storage::get(Scope::Local, "tbg_account_key").as_deref() == Some(&key);
    if secure && (!migrated || key_preserved) || !secure && (migrated || !key_preserved) {
        return Err(Failure::StorageUnavailable);
    }
    let mut clean = true;
    if let Ok(entries) = std::fs::read_dir(dir.join("auth")) {
        for entry in entries.flatten() {
            if entry.path().is_file() {
                let bytes = std::fs::read(entry.path()).map_err(|_| Failure::StorageUnavailable)?;
                for secret in [
                    rotated.refresh.as_deref().unwrap_or(""),
                    &rotated
                        .access
                        .0
                        .iter()
                        .map(|v| format!("{v:02x}"))
                        .collect::<String>(),
                ] {
                    if !secret.is_empty()
                        && bytes.windows(secret.len()).any(|v| v == secret.as_bytes())
                    {
                        clean = false;
                    }
                }
            }
        }
    }
    let mut offline_pending = false;
    if crate::config::url_param("auth-stepwise").is_some() {
        use std::io::Write;
        println!("{}", json!({"stage":"ready","secure_persistence":secure}));
        let _ = std::io::stdout().flush();
        let mut input = String::new();
        std::io::stdin()
            .read_line(&mut input)
            .map_err(|_| Failure::Unavailable)?;
        if input.trim() != "offline" {
            return Err(Failure::Cancelled);
        }
        let revoke = || Work {
            origin: origin.clone(),
            operation: Operation::Logout,
            remember: true,
            refresh: rotated.refresh.clone(),
            family: Some(rotated.family.clone()),
            account: Some(rotated.profile.name.clone()),
            device: None,
            access: Some(rotated.access),
            epoch: 1,
            current_epoch: current.clone(),
        };
        offline_pending = matches!(native::run(revoke()), Ok(Outcome::LogoutPending(_)));
        let pending = Store::open(&origin)?
            .meta
            .get("logout_pending")
            .and_then(serde_json::Value::as_bool)
            == Some(true);
        if !offline_pending || !pending {
            return Err(Failure::Incompatible);
        }
        println!(
            "{}",
            json!({"stage":"offline-pending","nonsecret_marker":pending})
        );
        let _ = std::io::stdout().flush();
        input.clear();
        std::io::stdin()
            .read_line(&mut input)
            .map_err(|_| Failure::Unavailable)?;
        if input.trim() != "online" {
            return Err(Failure::Cancelled);
        }
        if !matches!(native::run(revoke())?, Outcome::LoggedOut) {
            return Err(Failure::Unavailable);
        }
    }
    let value = transport::post(&origin, "logout-all", json!({"account_key":key}), None)?;
    if value.get("ok").and_then(serde_json::Value::as_bool) != Some(true) {
        return Err(Failure::Incompatible);
    }
    let meta = Store::open(&origin)?;
    let namespace = meta.namespace(&origin, &rotated.profile.name)?;
    let vault_cleanup = crate::platform::credentials::remove(&namespace).is_ok();
    Ok(
        json!({"synthetic_loopback_only":true,"secure_persistence":secure,"restored_same_family":restored,"process_coordination":coordinated,"migration_completed":migrated,"legacy_preserved_in_temporary_mode":!secure&&key_preserved,"ordinary_auth_metadata_contains_no_tokens":clean,"offline_pending_observed":offline_pending,"remote_revocation":true,"vault_cleanup":vault_cleanup}),
    )
}
