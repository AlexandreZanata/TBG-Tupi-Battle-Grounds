//! BR-MD02/05/11, IC-07/18/19 and S12/S15: moderation is reversible and versioned.
#[path = "common/p75.rs"]
mod common;
#[path = "common/moderation.rs"]
mod moderation;
use common::*;
use serde_json::json;

#[tokio::test]
async fn br_md05_ban_and_unban_preserve_every_row() {
    let s = Server::new().await;
    let grant = s.register("Keeper", "keeper@example.invalid").await;
    s.sql("UPDATE accounts SET rating=1234,ranked_games=8,wins=5 WHERE id=1;CREATE TABLE future_inventory(account_id INTEGER REFERENCES accounts(id),value TEXT);INSERT INTO future_inventory VALUES(1,'owned');");
    s.sql(
        "INSERT INTO moderation_events(at_unix,kind,payload_json) VALUES(1000000,'settings','{}');",
    );
    tokio::time::sleep(std::time::Duration::from_millis(1200)).await;
    let key = grant["account_key"].as_str().unwrap();
    let before = s
        .request("GET", "/api/players/Keeper", json!(null), None)
        .await;
    assert_eq!(before.status, 200);
    moderation::action(&s.db, 1, "ban", None, 0, NOW)
        .await
        .unwrap();
    tokio::time::sleep(std::time::Duration::from_millis(1200)).await;
    assert_eq!(
        s.request("GET", "/api/players/Keeper", json!(null), None)
            .await
            .status,
        404
    );
    let input = json!({"account_key":key,"device_id":random(16),"persistent":false});
    let banned = s
        .request("POST", "/api/auth/native/login", input.clone(), None)
        .await;
    assert_eq!(banned.status, 403);
    assert_eq!(banned.json()["error"], "account_suspended");
    assert!(!banned.body.contains("private note"));
    assert!(
        moderation::action(&s.db, 1, "unban", None, 0, NOW)
            .await
            .is_err(),
        "S12 stale version"
    );
    moderation::action(&s.db, 1, "unban", None, 1, NOW)
        .await
        .unwrap();
    tokio::time::sleep(std::time::Duration::from_millis(1200)).await;
    assert_eq!(
        s.request("GET", "/api/players/Keeper", json!(null), None)
            .await
            .body,
        before.body,
        "IC-07"
    );
    assert_eq!(
        s.number("SELECT COUNT(*) FROM future_inventory WHERE value='owned'"),
        1
    );
    assert_eq!(s.number("SELECT rating FROM accounts WHERE id=1"), 1234);
    assert_eq!(
        s.request("POST", "/api/auth/native/login", input, None)
            .await
            .status,
        200
    );
}

#[tokio::test]
async fn op06_kill_switches_converge_on_both_slots() {
    let a = Server::new().await;
    let b = Server::on(a.db.clone(), a.clock.clone(), |_| {}).await;
    let account = a.register("Keeper", "keeper@example.invalid").await;
    moderation::action(&a.db, 1, "ban", Some(NOW + 3600), 0, NOW)
        .await
        .unwrap();
    tokio::time::sleep(std::time::Duration::from_millis(1200)).await;
    let body =
        json!({"account_key":account["account_key"],"device_id":random(16),"persistent":false});
    for s in [&a, &b] {
        assert_eq!(
            s.request("POST", "/api/auth/native/login", body.clone(), None)
                .await
                .status,
            403
        );
    }
    a.clock.set(NOW + 3600);
    for s in [&a, &b] {
        assert_eq!(
            s.request("POST", "/api/auth/native/login", body.clone(), None)
                .await
                .status,
            200,
            "ban expiry is inclusive"
        );
    }
}

#[path = "common/operator.rs"]
mod operator;
#[path = "common/ws.rs"]
mod ws;

#[tokio::test]
async fn ic18_all_six_switches_converge_disable_and_restore_on_both_slots() {
    let mut cache_a = None;
    let a = Server::configured(|c| {
        c.admin.enabled = true;
        c.admin.addr = "127.0.0.1:0".into();
        c.bootstrap_email = tbg_server::mail::settings::Private("owner@example.invalid".into());
        cache_a = Some(c.moderation.clone());
    })
    .await;
    let mut cache_b = None;
    let b = Server::on(a.db.clone(), a.clock.clone(), |c| {
        cache_b = Some(c.moderation.clone())
    })
    .await;
    let op = operator::claim(&a).await;
    let grant = a.register("Keeper", "keeper@example.invalid").await;
    let code = a.mail("account_verify").await.code.unwrap();
    assert_eq!(
        a.request(
            "POST",
            "/api/me/email/verify",
            json!({"code":code}),
            grant["access_token"].as_str()
        )
        .await
        .status,
        200
    );
    let other = b
        .request(
            "POST",
            "/api/auth/native/login",
            json!({"account_key":grant["account_key"],"device_id":random(16),"persistent":false}),
            None,
        )
        .await
        .json();
    let mut pairs = Vec::new();
    for (server, grant) in [(&a, &grant), (&b, &other)] {
        let mut left = ws::connect(&server.url, Some(grant)).await;
        let mut right = ws::connect(&server.url, None).await;
        ws::send(&mut left, tbg_core::protocol::ClientMsg::QuickMatch).await;
        ws::send(&mut right, tbg_core::protocol::ClientMsg::QuickMatch).await;
        ws::until(&mut left, |m| {
            matches!(m, tbg_core::protocol::ServerMsg::MatchStart { .. })
        })
        .await;
        ws::until(&mut right, |m| {
            matches!(m, tbg_core::protocol::ServerMsg::MatchStart { .. })
        })
        .await;
        pairs.push((left, right));
    }
    let keys = [
        "chat_enabled",
        "emoji_enabled",
        "custom_emoji_enabled",
        "emoji_uploads_enabled",
        "reports_enabled",
        "registration_enabled",
    ];
    for enabled in [false, true] {
        for key in keys {
            let current = a.number(&format!(
                "SELECT value FROM admin_settings WHERE key='{key}'"
            ));
            if current != i64::from(enabled) {
                assert_eq!(
                    op.request(
                        "PUT",
                        &format!("settings/{key}"),
                        json!({"value":u8::from(enabled),"reason":"other"})
                    )
                    .await
                    .status,
                    200
                );
            }
        }
        tokio::time::sleep(std::time::Duration::from_millis(1200)).await;
        for cache in [cache_a.as_ref().unwrap(), cache_b.as_ref().unwrap()] {
            for key in keys {
                assert_eq!(cache.enabled(key), enabled, "IC-18 {key}");
            }
        }
        for (left, _) in &mut pairs {
            ws::until(left,|m|matches!(m,tbg_core::protocol::ServerMsg::MatchSocial(p) if p.emojis==enabled&&p.custom_emojis==enabled&&p.chat==if enabled {tbg_core::protocol::ChatPermission::Allowed}else{tbg_core::protocol::ChatPermission::Disabled})).await;
        }
        for (server, access) in [
            (&a, grant["access_token"].as_str()),
            (&b, other["access_token"].as_str()),
        ] {
            let report = server
                .request(
                    "POST",
                    "/api/me/reports",
                    json!({"context":"profile","target":"Nobody","reason":"other"}),
                    access,
                )
                .await;
            assert_eq!(
                report.status,
                if enabled { 422 } else { 503 },
                "reports switch on both slots"
            );
            let upload = bytes_request(
                &server.url,
                "POST",
                "/api/me/emoji?label=valid",
                vec![0],
                "image/png",
                access,
            )
            .await;
            assert_eq!(
                upload.status,
                if enabled { 422 } else { 503 },
                "upload switch on both slots"
            );
        }
        if !enabled {
            for server in [&a, &b] {
                assert_eq!(
                    server
                        .request(
                            "POST",
                            "/api/auth/native/register",
                            registration("SwitchedOff", "new@example.invalid"),
                            None
                        )
                        .await
                        .status,
                    503
                );
            }
        }
    }
    for (left, _) in &mut pairs {
        ws::send(left, tbg_core::protocol::ClientMsg::Surrender).await;
    }
}

#[tokio::test]
async fn ic07_every_account_fk_is_identical_after_ban_unban() {
    let s = Server::configured(|c| c.history_migrate = true).await;
    s.register("Keeper", "keeper@example.invalid").await;
    s.sql("CREATE TABLE future_ownership(account_id INTEGER REFERENCES accounts(id),payload BLOB);INSERT INTO future_ownership VALUES(1,x'ffaa00');");
    let before = bound_rows(
        &s.db,
        &["auth_sessions", "account_moderation", "moderation_actions"],
        &[],
    );
    moderation::action(&s.db, 1, "ban", Some(NOW + 3600), 0, NOW)
        .await
        .unwrap();
    moderation::action(&s.db, 1, "unban", None, 1, NOW)
        .await
        .unwrap();
    assert_eq!(
        bound_rows(
            &s.db,
            &["auth_sessions", "account_moderation", "moderation_actions"],
            &[]
        ),
        before,
        "IC-07 every future account FK"
    );
}

#[tokio::test]
async fn br_md02_cross_slot_ban_forfeits_and_other_actions_do_not_end_a_match() {
    use tbg_core::protocol::{ClientMsg, ServerMsg};
    let a = Server::new().await;
    let grant = a.register("Keeper", "keeper@example.invalid").await;
    let b = Server::on(a.db.clone(), a.clock.clone(), |_| {}).await;
    let signed = b
        .request(
            "POST",
            "/api/auth/native/login",
            json!({"account_key":grant["account_key"],"device_id":random(16),"persistent":false}),
            None,
        )
        .await
        .json();
    let mut player = ws::connect(&b.url, Some(&signed)).await;
    let mut peer = ws::connect(&b.url, None).await;
    ws::send(&mut player, ClientMsg::QuickMatch).await;
    ws::send(&mut peer, ClientMsg::QuickMatch).await;
    let start = ws::until(&mut player, |m| matches!(m, ServerMsg::MatchStart { .. })).await;
    let seat = match start {
        ServerMsg::MatchStart { you, .. } => you,
        _ => unreachable!(),
    };
    ws::until(&mut peer, |m| matches!(m, ServerMsg::MatchStart { .. })).await;
    moderation::action(&a.db, 1, "chat_restrict", Some(NOW + 3600), 0, NOW)
        .await
        .unwrap();
    moderation::action(&a.db, 1, "emoji_restrict", Some(NOW + 3600), 1, NOW)
        .await
        .unwrap();
    let live = ws::until(&mut peer, |m| matches!(m, ServerMsg::View(_))).await;
    assert!(
        matches!(live,ServerMsg::View(v) if v.outcome.is_none()),
        "restrictions never invent an outcome"
    );
    let started = tokio::time::Instant::now();
    moderation::action(&a.db, 1, "ban", None, 2, NOW)
        .await
        .unwrap();
    let end = tokio::time::timeout_at(
        started + std::time::Duration::from_secs(2),
        ws::until(&mut peer, |m| matches!(m, ServerMsg::MatchEnd { .. })),
    )
    .await
    .expect("ban feed exceeded 2 s");
    assert!(
        matches!(end,ServerMsg::MatchEnd{outcome,..} if outcome.reason==tbg_core::EndReason::Forfeit&&outcome.winner==Some(seat.other())),
        "IC-19 documented ban forfeit"
    );
}

#[tokio::test]
async fn s15_banned_refresh_resume_lease_guest_claim_and_blocked_reregistration_fail() {
    use tbg_core::protocol::{ClientMsg, ServerMsg, PROTOCOL_VERSION};
    let s = Server::new().await;
    let grant = s.register("Keeper", "keeper@example.invalid").await;
    let access = tbg_server::botclient::native_auth::parse(&grant)
        .unwrap()
        .access;
    let url = s.url.replacen("http://", "ws://", 1) + "/ws";
    let mut player = tokio_tungstenite::connect_async(&url).await.unwrap().0;
    ws::send(
        &mut player,
        ClientMsg::AuthenticatedHello {
            version: PROTOCOL_VERSION,
            resume: None,
            access,
        },
    )
    .await;
    let resume = match ws::until(&mut player, |m| matches!(m, ServerMsg::Welcome { .. })).await {
        ServerMsg::Welcome { token, .. } => token,
        _ => unreachable!(),
    };
    let mut peer = ws::connect(&s.url, None).await;
    ws::send(&mut player, ClientMsg::QuickMatch).await;
    ws::send(&mut peer, ClientMsg::QuickMatch).await;
    ws::until(&mut peer, |m| matches!(m, ServerMsg::MatchStart { .. })).await;
    moderation::action(&s.db, 1, "ban", None, 0, NOW)
        .await
        .unwrap();
    s.sql("INSERT INTO email_blocks VALUES('keeper@example.invalid',1000000,1,'ban_evasion');");
    tokio::time::sleep(std::time::Duration::from_millis(1200)).await;
    assert!(matches!(
        s.request(
            "POST",
            "/api/auth/native/refresh",
            json!({"refresh_token":grant["refresh_token"],"operation_id":random(16)}),
            None
        )
        .await
        .status,
        401 | 403
    ));
    for resume in [None, Some(resume)] {
        let mut socket = tokio_tungstenite::connect_async(&url).await.unwrap().0;
        ws::send(
            &mut socket,
            ClientMsg::AuthenticatedHello {
                version: PROTOCOL_VERSION,
                resume,
                access,
            },
        )
        .await;
        assert!(
            matches!(ws::recv(&mut socket).await, ServerMsg::AuthRequired { .. }),
            "banned Access and seat lease cannot resume"
        );
    }
    let mut guest = ws::connect(&s.url, None).await;
    ws::send(&mut guest, ClientMsg::Reauthenticate { access }).await;
    assert!(matches!(
        ws::until(&mut guest, |m| matches!(m, ServerMsg::AuthRequired { .. })).await,
        ServerMsg::AuthRequired { .. }
    ));
    ws::send(&mut guest, ClientMsg::ClaimAccount { take_over: true }).await;
    assert!(matches!(
        ws::until(&mut guest, |m| matches!(m, ServerMsg::AuthRequired { .. })).await,
        ServerMsg::AuthRequired { .. }
    ));
    s.clock.set(NOW + 901);
    let mut socket = tokio_tungstenite::connect_async(&url).await.unwrap().0;
    ws::send(
        &mut socket,
        ClientMsg::AuthenticatedHello {
            version: PROTOCOL_VERSION,
            resume: Some(resume),
            access,
        },
    )
    .await;
    assert!(
        matches!(ws::recv(&mut socket).await, ServerMsg::AuthRequired { .. }),
        "expired lease still checks the ban"
    );
    assert_eq!(
        s.request(
            "POST",
            "/api/auth/native/register",
            registration("Evader", "keeper@EXAMPLE.invalid"),
            None
        )
        .await
        .status,
        403
    );
    assert_eq!(s.number("SELECT COUNT(*) FROM accounts"), 1);
}

#[tokio::test]
async fn s12_ban_and_unban_race_uses_one_expected_version() {
    let s = Server::new().await;
    s.register("Keeper", "keeper@example.invalid").await;
    let (one, two) = tokio::join!(
        moderation::action(&s.db, 1, "ban", None, 0, NOW),
        moderation::action(&s.db, 1, "unban", None, 0, NOW)
    );
    assert_eq!(u8::from(one.is_ok()) + u8::from(two.is_ok()), 1);
    assert_eq!(s.number("SELECT version FROM account_moderation"), 1);
    assert_eq!(s.number("SELECT COUNT(*) FROM moderation_actions"), 1);
    assert_eq!(
        s.number("SELECT COUNT(*) FROM admin_audit WHERE target_kind='account'"),
        1
    );
}
