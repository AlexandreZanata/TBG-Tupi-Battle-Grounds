//! D-97: late account replies cannot cross logout/switch epochs; credentials stay memory-only.
use super::*;
use serde_json::json;

#[test]
fn http_grants_enforce_ttl_channels_and_strict_credential_widths() {
    let profile = json!({"name":"SyntheticUser","rating":1000,"ranked_games":0,"wins":0,"losses":0,"draws":0});
    let value = json!({"access_token":"ab".repeat(32),"refresh_token":"cd".repeat(32),"session_id":"ef".repeat(16),"server_time":1000,"access_expires_at":1900,"session_expires_at":2_593_000,"profile":profile,"persistent":true});
    assert!(types::parse_grant(value.clone(), true).is_ok());
    assert!(matches!(
        types::parse_grant(value.clone(), false),
        Err(Failure::Incompatible)
    ));
    for (field, bad) in [
        ("access_token", json!("ab".repeat(31))),
        ("access_token", json!("AB".repeat(32))),
        ("refresh_token", json!("cd".repeat(33))),
        ("access_expires_at", json!(1901)),
        ("session_expires_at", json!(2_593_001)),
    ] {
        let mut broken = value.clone();
        broken[field] = bad;
        assert!(matches!(
            types::parse_grant(broken, true),
            Err(Failure::Incompatible)
        ));
    }
    let mut browser = value.clone();
    browser.as_object_mut().unwrap().remove("refresh_token");
    assert!(types::parse_grant(browser, false).is_ok());
}

#[test]
fn untrusted_transport_failures_never_become_ui_text() {
    for value in [
        json!({"error":"https://private/secret"}),
        json!({"error":"bad-token-value"}),
        json!({"refresh_token":"cd".repeat(32)}),
    ] {
        assert_eq!(types::public_error(&value), Failure::Incompatible);
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn native_auth_never_sends_credentials_to_public_cleartext_or_credential_urls() {
    for address in [
        "ws://example.com/ws",
        "wss://user:password@example.com/ws",
        "wss://example.com/ws?token=value",
        "wss://example.com/ws#fragment",
    ] {
        assert_eq!(transport::origin(address), Err(Failure::InvalidOrigin));
    }
    assert_eq!(
        transport::origin("wss://example.com/ws"),
        Ok("https://example.com".into())
    );
    assert_eq!(
        transport::origin("ws://[::1]:8080/ws"),
        Ok("http://[::1]:8080".into())
    );
}

#[test]
fn a08_cancelled_completed_login_never_publishes_identity_and_retains_revocation_proof() {
    let mut session = Session {
        backup_after_login: false,
        origin: Err(Failure::InvalidOrigin),
        pending: None,
        next: Some(Command::Guest),
        refresh: None,
        family: None,
        account: None,
        device: None,
        access: None,
        deadline: 0.,
        retry_at: 0.,
        secure: false,
        remember: true,
        resume_write: None,
        resume_retry_at: 0.,
    };
    let mut account = crate::ui::account::Account::default();
    let mut inbox = crate::ui::account::AccountInbox::default();
    let grant = Grant {
        access: AccessToken([0xab; 32]),
        refresh: Some("cd".repeat(32)),
        family: "ef".repeat(16),
        profile: Profile {
            name: "CancelledUser".into(),
            rating: 1000,
            ranked_games: 0,
            wins: 0,
            losses: 0,
            draws: 0,
        },
        account_key: None,
        remaining: 900,
        persistent: true,
    };
    // Epoch zero is stale without mutating the process-global epoch or calling a provider.
    session.settle(
        0,
        Kind::Login,
        Ok(Outcome::SignedIn {
            grant,
            secure: true,
            warning: None,
            device: "01".repeat(16),
        }),
        0.,
        &mut account,
        &mut inbox,
    );
    assert!(account.profile.is_none() && inbox.0.is_empty());
    assert!(
        session.refresh.is_some(),
        "A08: do not lose the proof needed to revoke a completed late grant"
    );
    assert!(
        matches!(
            session.pending,
            Some(Pending {
                kind: Kind::Logout,
                ..
            })
        ),
        "A08: cancellation schedules revocation rather than publishing the old success"
    );
    assert!(
        session.next.is_none(),
        "the queued Guest action is fulfilled by revocation, never followed by a second logout"
    );
}

#[test]
fn p66_auth_http_follows_the_admitted_generation_slot() {
    set_endpoint("/ws/slot-b");
    assert_eq!(slot(), Some('b'));
    for unrouted in ["/ws", "/ws/slot-bb", "/ws/slot-B", "/ws/slot-", "/other"] {
        set_endpoint(unrouted);
        assert_eq!(slot(), None, "{unrouted}");
    }
}

#[test]
fn br_as07_one_forced_renewal_per_refusal_and_never_after_revocation() {
    use tbg_core::protocol::AuthRefusal;
    let view = Snapshot {
        access: Some(AccessToken([1; 32])),
        status: Status::Authenticated,
        ..Snapshot::default()
    };
    let forced = AtomicBool::new(false);
    assert!(should_renew(AuthRefusal::Expired, &view, &forced));
    assert!(
        !should_renew(AuthRefusal::Invalid, &view, &forced),
        "a renewed Access the server still refuses (misrouted generation) must not loop"
    );
    // `AuthState::Active` re-arms the single renewal.
    forced.store(false, Ordering::Release);
    assert!(should_renew(AuthRefusal::Invalid, &view, &forced));
    assert!(!should_renew(
        AuthRefusal::Revoked,
        &view,
        &AtomicBool::new(false)
    ));
    let signed_out = Snapshot {
        access: None,
        ..view.clone()
    };
    assert!(!should_renew(
        AuthRefusal::Expired,
        &signed_out,
        &AtomicBool::new(false)
    ));
}

#[test]
fn p66_u09_updates_wait_for_restore_renewal_sign_out_and_vault_work() {
    for busy in [Status::Restoring, Status::Refreshing, Status::LogoutPending] {
        assert!(!update_idle(false, busy));
    }
    for idle in [
        Status::Guest,
        Status::Authenticated,
        Status::PersistenceUnavailable,
        Status::Conflict,
        Status::ReauthRequired,
    ] {
        assert!(update_idle(false, idle));
        assert!(
            !update_idle(true, idle),
            "any coordinator operation blocks updates"
        );
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn p66_a_busy_resume_write_is_kept_without_touching_identity() {
    let mut session = Session {
        backup_after_login: false,
        origin: Err(Failure::InvalidOrigin),
        pending: None,
        next: None,
        refresh: None,
        family: None,
        account: None,
        device: None,
        access: None,
        deadline: 0.,
        retry_at: 0.,
        secure: false,
        remember: false,
        resume_write: Some(Some("synthetic-record".into())),
        resume_retry_at: 0.,
    };
    let mut account = crate::ui::account::Account::default();
    let mut inbox = crate::ui::account::AccountInbox::default();
    // Epoch zero is stale: resume results bypass the identity state machine.
    session.settle(
        0,
        Kind::Resume,
        Err(Failure::Busy),
        0.,
        &mut account,
        &mut inbox,
    );
    assert_eq!(
        crate::platform::resume::take_pending(),
        Some(Some("synthetic-record".into())),
        "another process held the lock: retry the same write later"
    );
    assert!(session.pending.is_none() && session.next.is_none());
    assert!(account.profile.is_none() && inbox.0.is_empty());
    assert!(
        session.resume_retry_at > 0.,
        "a busy lock backs off instead of retrying every frame"
    );
}

#[test]
fn br_as04_11_confirmation_retry_retains_identity_and_recovers_persistence() {
    let mut session = Session {
        backup_after_login: false,
        origin: Err(Failure::InvalidOrigin),
        pending: None,
        next: None,
        refresh: None,
        family: None,
        account: None,
        device: None,
        access: None,
        deadline: 0.,
        retry_at: 0.,
        secure: false,
        remember: true,
        resume_write: None,
        resume_retry_at: 0.,
    };
    let grant = || Grant {
        access: AccessToken([0xab; 32]),
        refresh: Some("cd".repeat(32)),
        family: "ef".repeat(16),
        profile: Profile {
            name: "RetryUser".into(),
            rating: 1000,
            ranked_games: 0,
            wins: 0,
            losses: 0,
            draws: 0,
        },
        account_key: None,
        remaining: 900,
        persistent: true,
    };
    let previous = snapshot();
    let mut account = crate::ui::account::Account::default();
    let mut inbox = crate::ui::account::AccountInbox::default();
    session.complete(
        Kind::Restore,
        Ok(Outcome::SignedIn {
            grant: grant(),
            secure: false,
            warning: Some(Failure::Unavailable),
            device: "01".repeat(16),
        }),
        10.,
        &mut account,
        &mut inbox,
    );
    let pending = snapshot();
    assert_eq!(pending.status, Status::PersistenceUnavailable);
    assert_eq!(
        pending.error,
        Some(Failure::Unavailable),
        "BR-AS11: a failed confirmation is not a failed vault"
    );
    assert!(pending.access.is_some() && pending.profile.is_some() && pending.remember);
    assert_eq!(
        pending.active_status(),
        Status::PersistenceUnavailable,
        "BR-AS06: queue/socket admission cannot confirm persistence"
    );
    assert_eq!(
        session.retry_at, 70.,
        "use the existing bounded retry delay"
    );
    session.complete(
        Kind::Refresh,
        Ok(Outcome::SignedIn {
            grant: grant(),
            secure: true,
            warning: None,
            device: "01".repeat(16),
        }),
        70.,
        &mut account,
        &mut inbox,
    );
    let recovered = snapshot();
    assert_eq!(recovered.status, Status::Authenticated);
    assert!(recovered.secure && recovered.remember && recovered.access == pending.access);
    assert_eq!(recovered.error, None);
    // BR-AS07: a transient renewal failure retains the current identity and
    // its existing secure-storage status until the coordinator can retry.
    session.complete(
        Kind::Refresh,
        Err(Failure::Unavailable),
        80.,
        &mut account,
        &mut inbox,
    );
    let outage = snapshot();
    assert_eq!(outage.error, Some(Failure::Unavailable));
    assert!(outage.secure && outage.access == recovered.access);
    *VIEW.write().unwrap() = previous;
}

#[test]
fn br_as05_recovery_key_failures_do_not_log_out_the_current_account() {
    let mut session = Session {
        backup_after_login: false,
        origin: Err(Failure::InvalidOrigin),
        pending: None,
        next: None,
        refresh: Some("cd".repeat(32)),
        family: Some("ef".repeat(16)),
        account: Some("BackupUser".into()),
        device: Some("01".repeat(16)),
        access: Some(AccessToken([0xab; 32])),
        deadline: 900.,
        retry_at: 0.,
        secure: true,
        remember: true,
        resume_write: None,
        resume_retry_at: 0.,
    };
    let previous = snapshot();
    let mut account = crate::ui::account::Account::default();
    let mut inbox = crate::ui::account::AccountInbox::default();
    session.complete(
        Kind::RecoveryKey,
        Err(Failure::InvalidKey),
        0.,
        &mut account,
        &mut inbox,
    );
    assert!(matches!(account.key_result, Some(Err(Failure::InvalidKey))));
    let after = snapshot();
    assert!(
        after.status == previous.status
            && after.access == previous.access
            && after.family == previous.family
    );
    assert!(session.access.is_some() && session.secure && session.remember && inbox.0.is_empty());
}
