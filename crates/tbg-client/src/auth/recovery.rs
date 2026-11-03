//! Recovery backup replacement through the protected OS-vault namespace used by PR #57.
#[cfg(not(target_arch = "wasm32"))]
pub(super) fn replace_backup(
    store: Option<&super::native_store::Store>,
    origin: &str,
    account: &str,
    key: Option<&str>,
) -> bool {
    let (Some(store), Some(key)) = (store, key) else {
        return false;
    };
    let Ok(namespace) = store.namespace(origin, account) else {
        return false;
    };
    let namespace = format!("{namespace}:recovery-key");
    let protected = match crate::platform::credentials::read(&namespace) {
        Ok(old) => {
            if !tbg_core::account::parse_key(&old)
                .is_some_and(|(name, _)| name.eq_ignore_ascii_case(account))
            {
                return false;
            }
            true
        }
        Err(crate::platform::credentials::Error::Missing) => false,
        Err(_) => return false,
    };
    let legacy = super::native::legacy(origin).is_some_and(|old| {
        tbg_core::account::parse_key(&old)
            .is_some_and(|(name, _)| name.eq_ignore_ascii_case(account))
    });
    if protected || legacy {
        // Replace only an existing same-account backup; never delete it or opt into persistence.
        return crate::platform::credentials::write(&namespace, key).is_ok();
    }
    true
}
