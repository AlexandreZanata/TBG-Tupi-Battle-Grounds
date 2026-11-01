//! Background native session operations under one per-origin OS lock and real vaults.

use super::{
    native_store::{nonce, Store},
    transport,
    types::{parse_grant, valid_hex, Failure, Grant, Outcome},
};
use crate::platform::credentials;
use serde_json::{json, Value};
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};
use tbg_core::protocol::AccessToken;

pub(super) enum Operation {
    Restore,
    Login(String),
    Register(String),
    Refresh,
    Backup,
    Logout,
}
pub(super) struct Work {
    pub origin: String,
    pub operation: Operation,
    pub remember: bool,
    pub refresh: Option<String>,
    pub family: Option<String>,
    pub account: Option<String>,
    pub device: Option<String>,
    pub access: Option<AccessToken>,
    pub epoch: u64,
    pub current_epoch: Arc<AtomicU64>,
}

struct Vault {
    refresh: String,
    family: String,
    operation: Option<String>,
}
impl Vault {
    fn read(namespace: &str) -> Result<Self, Failure> {
        let raw = credentials::read(namespace).map_err(|e| match e {
            credentials::Error::Missing => Failure::NoSession,
            _ => Failure::StorageUnavailable,
        })?;
        let value: Value = serde_json::from_str(&raw).map_err(|_| Failure::StorageUnavailable)?;
        let refresh = value
            .get("refresh")
            .and_then(Value::as_str)
            .filter(|v| valid_hex(v, 32))
            .ok_or(Failure::StorageUnavailable)?
            .to_owned();
        let family = value
            .get("family")
            .and_then(Value::as_str)
            .filter(|v| valid_hex(v, 16))
            .ok_or(Failure::StorageUnavailable)?
            .to_owned();
        let operation = value
            .get("operation")
            .and_then(Value::as_str)
            .filter(|v| valid_hex(v, 16))
            .map(str::to_owned);
        Ok(Self {
            refresh,
            family,
            operation,
        })
    }
}

pub(super) fn run(mut work: Work) -> Result<Outcome, Failure> {
    let mut store = match Store::open(&work.origin) {
        Ok(store) => Some(store),
        Err(Failure::Busy) => return Err(Failure::Busy),
        Err(error) if matches!(work.operation, Operation::Restore | Operation::Backup) => {
            return Err(error)
        }
        Err(_) => None,
    };
    if matches!(work.operation, Operation::Restore) {
        work.remember = store
            .as_ref()
            .is_some_and(|s| s.meta.get("remember").and_then(Value::as_bool) == Some(true));
    }
    if !matches!(work.operation, Operation::Logout)
        && store
            .as_ref()
            .is_some_and(|s| s.meta.get("logout_pending").and_then(Value::as_bool) == Some(true))
    {
        return logout(&work, store.as_mut());
    }
    match &work.operation {
        Operation::Restore => restore(&work, store.as_mut().ok_or(Failure::StorageUnavailable)?),
        Operation::Login(key) => login(&work, store.as_mut(), Some(key), None),
        Operation::Register(name) => login(&work, store.as_mut(), None, Some(name)),
        Operation::Refresh => refresh(&work, store.as_mut()),
        Operation::Backup => backup(&work, store.as_mut().ok_or(Failure::StorageUnavailable)?),
        Operation::Logout => logout(&work, store.as_mut()),
    }
}

pub(super) fn legacy(origin: &str) -> Option<String> {
    let bound = crate::platform::storage::get(
        crate::platform::storage::Scope::Local,
        "tbg_auth_legacy_origin",
    )
    .unwrap_or_else(|| "https://tbg.abastevo.com.br".into());
    if bound != origin {
        return None;
    }
    crate::config::load_account_key()
}

fn restore(work: &Work, store: &mut Store) -> Result<Outcome, Failure> {
    if store.meta.get("remember").and_then(Value::as_bool) != Some(true) {
        return Ok(Outcome::Guest {
            legacy: legacy(&work.origin),
        });
    }
    let account = store.account().ok_or(Failure::NoSession)?.to_owned();
    let namespace = store.namespace(&work.origin, &account)?;
    let vault = Vault::read(&namespace)?;
    renew(work, Some(store), vault.refresh, vault.family, true)
}

fn login(
    work: &Work,
    mut store: Option<&mut Store>,
    key: Option<&str>,
    name: Option<&str>,
) -> Result<Outcome, Failure> {
    if work.refresh.is_some()
        || store
            .as_ref()
            .is_some_and(|s| s.meta.get("remember").and_then(Value::as_bool) == Some(true))
    {
        match logout(work, store.as_deref_mut())? {
            Outcome::LoggedOut => {}
            Outcome::LogoutPending(error) => return Ok(Outcome::LogoutPending(error)),
            _ => return Err(Failure::Unavailable),
        }
    }
    let device = match &store {
        Some(s) => s.device()?.to_owned(),
        None => nonce(16)?,
    };
    let body = if let Some(key) = key {
        json!({"account_key":key,"device_id":device,"persistent":work.remember})
    } else {
        json!({"name":name.ok_or(Failure::Incompatible)?,"device_id":device,"persistent":work.remember,"registration_retry_key":nonce(32)?})
    };
    let operation = if key.is_some() { "login" } else { "register" };
    let value = retry(&work.origin, operation, body, None)?;
    let grant = parse_grant(value, true)?;
    let secure = save_grant(work, store.as_deref_mut(), &grant, None)?;
    cancelled(work, &grant, store)?;
    Ok(Outcome::SignedIn {
        grant,
        secure,
        device,
    })
}

fn refresh(work: &Work, mut store: Option<&mut Store>) -> Result<Outcome, Failure> {
    let account = work.account.as_deref().ok_or(Failure::NoSession)?;
    if work.remember {
        if let Some(s) = &mut store {
            if let Ok(vault) = Vault::read(&s.namespace(&work.origin, account)?) {
                if work
                    .family
                    .as_ref()
                    .is_none_or(|family| *family == vault.family)
                {
                    return renew(work, store, vault.refresh, vault.family, true);
                }
                // A different process saved a newer family. Never overwrite it.
                return renew(
                    work,
                    None,
                    work.refresh.clone().ok_or(Failure::Conflict)?,
                    work.family.clone().ok_or(Failure::Conflict)?,
                    false,
                );
            }
        }
    }
    renew(
        work,
        store,
        work.refresh.clone().ok_or(Failure::NoSession)?,
        work.family.clone().ok_or(Failure::NoSession)?,
        false,
    )
}

fn renew(
    work: &Work,
    mut store: Option<&mut Store>,
    refresh: String,
    family: String,
    persist: bool,
) -> Result<Outcome, Failure> {
    let mut operation = nonce(16)?;
    if let Some(s) = &mut store {
        if let Some(pending) = s
            .meta
            .get("operation")
            .and_then(Value::as_str)
            .filter(|v| valid_hex(v, 16))
        {
            let account = s.account().ok_or(Failure::NoSession)?;
            let stored = Vault::read(&s.namespace(&work.origin, account)?)?;
            if stored.operation.as_deref() != Some(pending) {
                operation = pending.to_owned();
            }
        }
        s.meta["operation"] = json!(operation);
        s.save()?;
    }
    let grant = parse_grant(
        retry(
            &work.origin,
            "refresh",
            json!({"refresh_token":refresh,"operation_id":operation}),
            None,
        )?,
        true,
    )?;
    if grant.family != family {
        return Err(Failure::Incompatible);
    }
    let device = match &store {
        Some(s) => s.device()?.to_owned(),
        None => work.device.clone().ok_or(Failure::NoSession)?,
    };
    let secure = if persist {
        save_grant(work, store.as_deref_mut(), &grant, Some(&operation))?
    } else {
        if let Some(s) = &mut store {
            s.meta["remember"] = json!(false);
            s.meta["operation"] = Value::Null;
            s.save()?;
        }
        false
    };
    cancelled(work, &grant, store)?;
    Ok(Outcome::SignedIn {
        grant,
        secure,
        device,
    })
}

fn save_grant(
    work: &Work,
    store: Option<&mut Store>,
    grant: &Grant,
    operation: Option<&str>,
) -> Result<bool, Failure> {
    let Some(store) = store else {
        return Ok(false);
    };
    if work.current_epoch.load(Ordering::Acquire) != work.epoch {
        return Ok(false);
    }
    let namespace = store.namespace(&work.origin, &grant.profile.name)?;
    let mut secure = false;
    if work.remember {
        let secret=json!({"refresh":grant.refresh.as_deref().ok_or(Failure::Incompatible)?,"family":grant.family,"operation":operation}).to_string();
        if credentials::write(&namespace, &secret).is_ok() {
            let confirmed = transport::post(
                &work.origin,
                "confirm",
                json!({"refresh_token":grant.refresh}),
                None,
            );
            secure = confirmed.is_ok_and(|v| {
                v.get("session_id").and_then(Value::as_str) == Some(grant.family.as_str())
            });
        }
    }
    store.meta["account"] = json!(grant.profile.name);
    store.meta["family"] = json!(grant.family);
    store.meta["remember"] = json!(secure);
    store.meta["logout_pending"] = json!(false);
    store.meta["operation"] = Value::Null;
    if store.save().is_err() {
        if Vault::read(&namespace).is_ok_and(|v| v.family == grant.family) {
            let _ = credentials::remove(&namespace);
        }
        return Ok(false);
    }
    Ok(secure)
}

fn cancelled(work: &Work, grant: &Grant, mut store: Option<&mut Store>) -> Result<(), Failure> {
    if work.current_epoch.load(Ordering::Acquire) == work.epoch {
        return Ok(());
    }
    let revoked = transport::post(
        &work.origin,
        "logout",
        json!({"refresh_token":grant.refresh}),
        Some(grant.access),
    )
    .is_ok();
    if !revoked {
        if let Some(s) = &mut store {
            let namespace = format!(
                "{}:logout:{}",
                s.namespace(&work.origin, &grant.profile.name)?,
                grant.family
            );
            let secret = json!({"refresh":grant.refresh,"family":grant.family}).to_string();
            let secure = credentials::write(&namespace, &secret).is_ok();
            s.meta["logout_pending"] = json!(true);
            s.meta["pending_logout_family"] = json!(grant.family);
            s.meta["pending_logout_account"] = json!(grant.profile.name);
            s.meta["pending_logout_secure"] = json!(secure);
            // If no vault is available, retain a nonsecret recovery-required marker.
            // Never claim another family's logout completed this cancellation.
            s.save()?;
        }
    } else if let Some(s) = store {
        let namespace = s.namespace(&work.origin, &grant.profile.name)?;
        if Vault::read(&namespace).is_ok_and(|v| v.family == grant.family) {
            let _ = credentials::remove(&namespace);
        }
        if s.meta.get("family").and_then(Value::as_str) == Some(grant.family.as_str()) {
            s.meta["remember"] = json!(false);
            s.save()?;
        }
    }
    Err(if revoked {
        Failure::Cancelled
    } else {
        Failure::Unavailable
    })
}

fn finish_pending(work: &Work, store: &mut Store) -> Result<Option<Outcome>, Failure> {
    let Some(family) = store
        .meta
        .get("pending_logout_family")
        .and_then(Value::as_str)
        .filter(|id| valid_hex(id, 16))
        .map(str::to_owned)
    else {
        return Ok(None);
    };
    let account = store
        .meta
        .get("pending_logout_account")
        .and_then(Value::as_str)
        .filter(|v| tbg_core::account::valid_name_shape(v))
        .ok_or(Failure::RecoveryRequired)?;
    let namespace = format!(
        "{}:logout:{family}",
        store.namespace(&work.origin, account)?
    );
    let vault = match Vault::read(&namespace) {
        Ok(v) if v.family == family => v,
        _ => return Ok(Some(Outcome::LogoutPending(Failure::RecoveryRequired))),
    };
    match transport::post(
        &work.origin,
        "logout",
        json!({"refresh_token":vault.refresh}),
        None,
    ) {
        Ok(_) => {
            credentials::remove(&namespace).map_err(|_| Failure::StorageUnavailable)?;
            let base = namespace
                .split(":logout:")
                .next()
                .ok_or(Failure::StorageUnavailable)?;
            if Vault::read(base).is_ok_and(|v| v.family == family) {
                credentials::remove(base).map_err(|_| Failure::StorageUnavailable)?;
            }
            for key in [
                "pending_logout_family",
                "pending_logout_account",
                "pending_logout_secure",
            ] {
                store
                    .meta
                    .as_object_mut()
                    .ok_or(Failure::StorageUnavailable)?
                    .remove(key);
            }
            store.meta["logout_pending"] = json!(false);
            store.meta["remember"] = json!(false);
            store.save()?;
            Ok(Some(Outcome::LoggedOut))
        }
        Err(error) => Ok(Some(Outcome::LogoutPending(error))),
    }
}

fn logout(work: &Work, mut store: Option<&mut Store>) -> Result<Outcome, Failure> {
    if let Some(s) = &mut store {
        if let Some(outcome) = finish_pending(work, s)? {
            return Ok(outcome);
        }
    }
    let mut token = work.refresh.clone();
    let account = work
        .account
        .as_deref()
        .or_else(|| store.as_ref().and_then(|s| s.account()))
        .map(str::to_owned);
    let namespace = match (&store, &account) {
        (Some(s), Some(account)) => Some(s.namespace(&work.origin, account)?),
        _ => None,
    };
    let owns_meta = store.as_ref().is_some_and(|s| {
        work.family
            .as_ref()
            .is_none_or(|id| s.meta.get("family").and_then(Value::as_str) == Some(id.as_str()))
    });
    if owns_meta {
        if let Some(s) = &mut store {
            s.meta["logout_pending"] = json!(true);
            let _ = s.save();
        }
    }
    if let Some(namespace) = &namespace {
        if let Ok(vault) = Vault::read(namespace) {
            if work.family.as_ref().is_none_or(|id| *id == vault.family) {
                token = Some(vault.refresh);
            }
        }
    }
    let result = transport::post(
        &work.origin,
        "logout",
        json!({"refresh_token":token}),
        work.access,
    );
    match result {
        Ok(_) => {
            if let Some(namespace) = namespace {
                if Vault::read(&namespace)
                    .is_ok_and(|v| work.family.as_ref().is_none_or(|id| *id == v.family))
                {
                    let _ = credentials::remove(&namespace);
                }
            }
            if let Some(s) = store.filter(|_| owns_meta) {
                s.meta["remember"] = json!(false);
                s.meta["logout_pending"] = json!(false);
                s.meta["family"] = Value::Null;
                s.save()?;
            }
            Ok(Outcome::LoggedOut)
        }
        Err(error) => Ok(Outcome::LogoutPending(error)),
    }
}

fn backup(work: &Work, store: &mut Store) -> Result<Outcome, Failure> {
    if !work.remember {
        return Ok(Outcome::Backup { migrated: false });
    }
    let account = work.account.as_deref().ok_or(Failure::NoSession)?;
    let namespace = store.namespace(&work.origin, account)?;
    let vault = Vault::read(&namespace)?;
    if work.family.as_ref() != Some(&vault.family) {
        return Err(Failure::Conflict);
    }
    let confirmed = transport::post(
        &work.origin,
        "confirm",
        json!({"refresh_token":vault.refresh}),
        None,
    )?;
    if confirmed.get("session_id").and_then(Value::as_str) != Some(vault.family.as_str()) {
        return Err(Failure::Incompatible);
    }
    if work.current_epoch.load(Ordering::Acquire) != work.epoch {
        return Err(Failure::Cancelled);
    }
    store.meta["backed_up"] = json!(true);
    store.save()?;
    if let Some(key) = legacy(&work.origin) {
        if tbg_core::account::parse_key(&key)
            .is_some_and(|(name, _)| name.eq_ignore_ascii_case(account))
            && !crate::platform::storage::remove_if_matches(
                crate::platform::storage::Scope::Local,
                "tbg_account_key",
                &key,
            )
        {
            return Err(Failure::StorageUnavailable);
        }
    }
    Ok(Outcome::Backup { migrated: true })
}

fn retry(
    origin: &str,
    operation: &str,
    body: Value,
    access: Option<AccessToken>,
) -> Result<Value, Failure> {
    match transport::post(origin, operation, body.clone(), access) {
        Err(Failure::Unavailable) if operation == "register" || operation == "refresh" => {
            transport::post(origin, operation, body, access)
        }
        result => result,
    }
}
