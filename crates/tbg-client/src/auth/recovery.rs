//! Recovery backup replacement through the protected OS-vault namespace used by PR #57.
#[cfg(not(target_arch = "wasm32"))]
pub(super) fn replace_backup(
    store: Option<&super::native_store::Store>,
    origin: &str,
    account: &str,
    key: Option<&str>,
) {
    let (Some(store), Some(key)) = (store, key) else {
        return;
    };
    let Ok(namespace) = store.namespace(origin, account) else {
        return;
    };
    let namespace = format!("{namespace}:recovery-key");
    let present = crate::platform::credentials::read(&namespace).is_ok_and(|old| {
        tbg_core::account::parse_key(&old)
            .is_some_and(|(name, _)| name.eq_ignore_ascii_case(account))
    }) || crate::config::load_account_key().is_some_and(|old| {
        tbg_core::account::parse_key(&old)
            .is_some_and(|(name, _)| name.eq_ignore_ascii_case(account))
    });
    if present {
        // Never remove an existing backup; the recovered key also reaches the save-key screen.
        // PR #57's writer provides the same namespace and remains owned by that front.
        let _ = crate::platform::credentials::write(&namespace, key);
    }
}
