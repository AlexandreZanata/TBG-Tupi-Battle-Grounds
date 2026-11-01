//! Real per-user OS vaults; callers serialize each credential with the auth process lock.

use keyring_core::Entry;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    Missing,
    Unavailable,
    Invalid,
}

fn entry(namespace: &str) -> Result<Entry, Error> {
    if namespace.len() > 2048 || namespace.is_empty() {
        return Err(Error::Invalid);
    }
    #[cfg(target_os = "linux")]
    {
        let store =
            dbus_secret_service_keyring_store::Store::new().map_err(|_| Error::Unavailable)?;
        keyring_core::set_default_store(store);
        Entry::new("org.tupi.TBG.auth.v1", namespace).map_err(|_| Error::Unavailable)
    }
    #[cfg(target_os = "windows")]
    {
        let store = windows_native_keyring_store::Store::new().map_err(|_| Error::Unavailable)?;
        keyring_core::set_default_store(store);
        let target = format!("org.tupi.TBG.auth.v1/{namespace}");
        let modifiers = std::collections::HashMap::from([
            ("persistence", "Local"),
            ("target", target.as_str()),
        ]);
        Entry::new_with_modifiers("org.tupi.TBG.auth.v1", namespace, &modifiers)
            .map_err(|_| Error::Unavailable)
    }
    #[cfg(not(any(target_os = "linux", target_os = "windows")))]
    {
        Err(Error::Unavailable)
    }
}

pub fn read(namespace: &str) -> Result<String, Error> {
    let value = entry(namespace)?.get_password().map_err(|e| match e {
        keyring_core::Error::NoEntry => Error::Missing,
        _ => Error::Unavailable,
    })?;
    if value.len() > 4096 {
        return Err(Error::Invalid);
    }
    Ok(value)
}

pub fn write(namespace: &str, value: &str) -> Result<(), Error> {
    if value.len() > 4096 {
        return Err(Error::Invalid);
    }
    let item = entry(namespace)?;
    item.set_password(value).map_err(|_| Error::Unavailable)?;
    if item.get_password().map_err(|_| Error::Unavailable)? != value {
        return Err(Error::Unavailable);
    }
    #[cfg(target_os = "windows")]
    if item
        .get_attributes()
        .map_err(|_| Error::Unavailable)?
        .get("persistence")
        .is_none_or(|v| !v.eq_ignore_ascii_case("Local"))
    {
        return Err(Error::Unavailable);
    }
    Ok(())
}

pub fn remove(namespace: &str) -> Result<(), Error> {
    match entry(namespace)?.delete_credential() {
        Ok(()) | Err(keyring_core::Error::NoEntry) => Ok(()),
        Err(_) => Err(Error::Unavailable),
    }
}

/// This opt-in journey creates/deletes only a freshly randomized synthetic entry.
#[cfg(feature = "ui-acceptance")]
pub fn probe() {
    let mut raw = [0; 32];
    if getrandom::fill(&mut raw).is_err() {
        println!("{{\"vault\":\"unavailable\",\"stage\":\"entropy\"}}");
        return;
    }
    let id: String = raw.iter().map(|b| format!("{b:02x}")).collect();
    let namespace = format!("p66-synthetic-acceptance:{id}");
    let value = format!("synthetic-refresh-{id}");
    let written = write(&namespace, &value).is_ok();
    let read_back = written && read(&namespace).is_ok_and(|v| v == value);
    let removed = remove(&namespace).is_ok();
    println!(
        "{{\"vault\":\"{}\",\"write_readback\":{},\"cleanup\":{}}}",
        if read_back {
            "available"
        } else {
            "unavailable"
        },
        read_back,
        removed
    );
}
