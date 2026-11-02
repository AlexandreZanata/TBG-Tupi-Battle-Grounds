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
        mut grant,
        mut secure,
        warning,
        device,
    } = native::run(work)?
    else {
        return Err(Failure::Incompatible);
    };
    let key = grant.account_key.clone().ok_or(Failure::Incompatible)?;
    let retry_case = crate::config::url_param("auth-persistence-retry").is_some();
    if retry_case {
        // The owned test proxy rejects both initial confirm attempts, after
        // the real OS vault write. Recovery must not create a second family.
        let meta = Store::open(&origin)?;
        if secure
            || warning != Some(Failure::Unavailable)
            || meta
                .meta
                .get("remember")
                .and_then(serde_json::Value::as_bool)
                != Some(false)
            || meta
                .meta
                .get("remember_requested")
                .and_then(serde_json::Value::as_bool)
                != Some(true)
            || meta
                .meta
                .get("credential_saved")
                .and_then(serde_json::Value::as_bool)
                != Some(true)
        {
            return Err(Failure::Incompatible);
        }
        drop(meta);
        let result = native::run(Work {
            origin: origin.clone(),
            operation: Operation::Restore,
            remember: false,
            refresh: None,
            family: None,
            account: None,
            device: None,
            access: None,
            epoch: 1,
            current_epoch: current.clone(),
        })?;
        match result {
            Outcome::SignedIn {
                grant: recovered,
                secure: true,
                warning: None,
                ..
            } if recovered.family == grant.family => {
                grant = recovered;
                secure = true;
            }
            _ => return Err(Failure::Incompatible),
        }
    }
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
    let key_case = crate::config::url_param("auth-key-backup").is_some();
    if key_case {
        let key_work = |input| Work {
            origin: origin.clone(),
            operation: Operation::RecoveryKey(input),
            remember: true,
            refresh: rotated.refresh.clone(),
            family: Some(rotated.family.clone()),
            account: Some(rotated.profile.name.clone()),
            device: Some(device.clone()),
            access: Some(rotated.access),
            epoch: 1,
            current_epoch: current.clone(),
        };
        if !matches!(native::run(key_work(None))?, Outcome::RecoveryKey { key: Some(saved) } if saved == key)
        {
            return Err(Failure::Incompatible);
        }
        if !storage::remove_if_matches(Scope::Local, "tbg_account_key", &key)
            || !matches!(
                native::run(key_work(None))?,
                Outcome::RecoveryKey { key: None }
            )
            || !storage::set(Scope::Local, "tbg_account_key", &key)
        {
            return Err(Failure::Incompatible);
        }
        let mut wrong = key.as_bytes().to_vec();
        let last = wrong.last_mut().ok_or(Failure::Incompatible)?;
        *last = if *last == b'0' { b'1' } else { b'0' };
        let wrong = String::from_utf8(wrong).map_err(|_| Failure::Incompatible)?;
        if !matches!(native::run(key_work(Some(wrong))), Err(Failure::InvalidKey)) {
            return Err(Failure::Incompatible);
        }
        for input in [Some(key.clone()), None] {
            if !matches!(native::run(key_work(input))?, Outcome::RecoveryKey { key: Some(saved) } if saved == key)
            {
                return Err(Failure::Incompatible);
            }
        }
        let downloads = directories::UserDirs::new()
            .and_then(|dirs| dirs.download_dir().map(std::path::Path::to_path_buf))
            .ok_or(Failure::StorageUnavailable)?;
        if !downloads.starts_with(dir.parent().ok_or(Failure::InvalidOrigin)?) {
            return Err(Failure::InvalidOrigin);
        }
        if !crate::platform::copy(&key) || crate::platform::paste().as_deref() != Some(&key) {
            return Err(Failure::StorageUnavailable);
        }
        if !crate::ui::account::download_key(&rotated.profile.name, &key) {
            return Err(Failure::StorageUnavailable);
        }
        let exported = downloads.join(format!("{}-tbg-key.txt", rotated.profile.name));
        if std::fs::read_to_string(&exported).map_err(|_| Failure::StorageUnavailable)? != key {
            return Err(Failure::Incompatible);
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if std::fs::metadata(&exported)
                .map_err(|_| Failure::StorageUnavailable)?
                .permissions()
                .mode()
                & 0o777
                != 0o600
            {
                return Err(Failure::StorageUnavailable);
            }
        }
        std::fs::remove_file(exported).map_err(|_| Failure::StorageUnavailable)?;
        let proof = transport::post(
            &origin,
            "confirm",
            json!({"refresh_token":rotated.refresh}),
            None,
        )?;
        if proof["session_id"].as_str() != Some(&rotated.family) {
            return Err(Failure::Incompatible);
        }
        let meta = Store::open(&origin)?;
        let namespace = format!(
            "{}:recovery-key",
            meta.namespace(&origin, &rotated.profile.name)?
        );
        crate::platform::credentials::remove(&namespace)
            .map_err(|_| Failure::StorageUnavailable)?;
    }
    if retry_case {
        // BR-AS03/04: model a previous failed vault write by leaving an older,
        // consumed refresh in this synthetic entry while memory has the current
        // generation. Renewal must never replay that stale credential.
        let mut meta = Store::open(&origin)?;
        let namespace = meta.namespace(&origin, &rotated.profile.name)?;
        let stale =
            json!({"refresh":grant.refresh,"family":grant.family,"operation":null}).to_string();
        crate::platform::credentials::write(&namespace, &stale)
            .map_err(|_| Failure::StorageUnavailable)?;
        meta.meta["credential_saved"] = json!(false);
        meta.meta["remember"] = json!(false);
        meta.save()?;
        drop(meta);
        match native::run(Work {
            origin: origin.clone(),
            operation: Operation::Refresh,
            remember: true,
            refresh: rotated.refresh.clone(),
            family: Some(rotated.family.clone()),
            account: Some(rotated.profile.name.clone()),
            device: Some(device.clone()),
            access: Some(rotated.access),
            epoch: 1,
            current_epoch: current.clone(),
        })? {
            Outcome::SignedIn {
                grant: recovered,
                secure: true,
                warning: None,
                ..
            } if recovered.family == rotated.family => rotated = recovered,
            _ => return Err(Failure::Incompatible),
        }
    }
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
                    key.as_str(),
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
        json!({"synthetic_loopback_only":true,"secure_persistence":secure,"restored_same_family":restored,"process_coordination":coordinated,"migration_completed":migrated,"legacy_preserved_in_temporary_mode":!secure&&key_preserved,"ordinary_auth_metadata_contains_no_tokens":clean,"offline_pending_observed":offline_pending,"remote_revocation":true,"vault_cleanup":vault_cleanup,"recovery_key_backup_roundtrip":key_case,"confirmation_outage_recovered":retry_case,"stale_vault_recovered":retry_case}),
    )
}
