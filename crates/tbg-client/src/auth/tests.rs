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
