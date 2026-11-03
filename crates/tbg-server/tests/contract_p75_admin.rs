//! BR-AD01–15, IC-08/09/10/15/16 and S01–06/10/14: separate operator authority.
#[path = "common/p75.rs"]
mod common;
#[path = "common/moderation.rs"]
mod moderation;
#[path = "common/operator.rs"]
mod operator;
use common::*;
use serde_json::json;

#[tokio::test]
async fn br_ad02_bootstrap_is_idempotent_and_single() {
    let a = operator::server().await;
    let _ = a.mail("operator_claim").await;
    let b = Server::on(a.db.clone(), a.clock.clone(), |c| {
        c.bootstrap_email = tbg_server::mail::settings::Private("different@example.invalid".into())
    })
    .await;
    let _ = b.request("GET", "/health", json!(null), None).await;
    assert_eq!(
        a.number("SELECT COUNT(*) FROM operators WHERE role='owner'"),
        1
    );
    assert_eq!(a.number("SELECT COUNT(*) FROM operators WHERE email='owner@example.invalid' AND key_hash IS NULL"),1);
    assert_eq!(
        a.number("SELECT COUNT(*) FROM mail_outbox WHERE purpose='operator_claim'"),
        1
    );
}

#[tokio::test]
async fn br_ad01_player_and_operator_credentials_never_cross() {
    let s = operator::server().await;
    let op = operator::claim(&s).await;
    assert_eq!(op.request("GET", "session", json!(null)).await.status, 200);
    let player = s.register("Claimant", "player@example.invalid").await;
    assert_eq!(
        s.request(
            "GET",
            "/admin-api/v1/session",
            json!(null),
            player["access_token"].as_str()
        )
        .await
        .status,
        404,
        "IC-10 public router inventory"
    );
    assert_eq!(
        request(
            &op.url,
            "GET",
            "/admin-api/v1/session",
            json!(null),
            vec![(
                "authorization".into(),
                format!("Bearer {}", player["access_token"].as_str().unwrap())
            )]
        )
        .await
        .status,
        401
    );
    assert_eq!(
        request(
            &s.url,
            "GET",
            "/api/me/account",
            json!(null),
            vec![("cookie".into(), op.cookie.clone())]
        )
        .await
        .status,
        401
    );
    assert_eq!(
        s.request(
            "POST",
            "/api/auth/native/login",
            json!({"account_key":op.key,"device_id":random(16),"persistent":false}),
            None
        )
        .await
        .status,
        401
    );
}

#[tokio::test]
async fn br_ad09_csrf_origin_and_content_type_fail_before_audit_or_effect() {
    let s = operator::server().await;
    let op = operator::claim(&s).await;
    let count = s.number("SELECT COUNT(*) FROM admin_audit");
    for headers in [
        vec![("cookie".into(), op.cookie.clone())],
        vec![
            ("cookie".into(), op.cookie.clone()),
            ("origin".into(), "https://evil.invalid".into()),
            ("x-tbg-csrf".into(), op.csrf.clone()),
        ],
        vec![
            ("cookie".into(), op.cookie.clone()),
            ("origin".into(), "https://admin.tupibg.com".into()),
            ("x-tbg-csrf".into(), "wrong".into()),
        ],
    ] {
        assert_eq!(
            request(
                &op.url,
                "POST",
                "/admin-api/v1/session/logout",
                json!({}),
                headers
            )
            .await
            .status,
            403
        );
    }
    assert_eq!(s.number("SELECT COUNT(*) FROM admin_audit"), count);
    assert_eq!(op.request("GET", "session", json!(null)).await.status, 200);
    assert_eq!(
        op.request("GET", "session/logout", json!(null))
            .await
            .status,
        400
    );
    assert_eq!(
        op.request("POST", "session/logout", json!({})).await.status,
        200
    );
    assert_eq!(
        op.request("GET", "session", json!(null)).await.status,
        401,
        "S04 old cookie is revoked"
    );
}

#[tokio::test]
async fn br_ad04_claim_is_indistinguishable_and_claim_code_is_single_use() {
    let s = operator::server().await;
    let op = operator::preauth(&s).await;
    let unknown = op
        .request(
            "POST",
            "claim/request",
            json!({"email":"unknown@example.invalid"}),
        )
        .await;
    let known = op
        .request(
            "POST",
            "claim/request",
            json!({"email":"owner@example.invalid"}),
        )
        .await;
    assert_eq!(
        (unknown.status, unknown.body),
        (known.status, known.body),
        "IC-03/S06"
    );
    let code = s.mail("operator_claim").await.code.unwrap();
    let input = json!({"email":"owner@example.invalid","code":code});
    let one = op.request("POST", "claim/complete", input.clone()).await;
    assert_eq!(one.status, 200);
    let two = op.request("POST", "claim/complete", input).await;
    assert_eq!(two.status, 401, "IC-15/S12");
    let key = one.json()["admin_key"].as_str().unwrap().to_owned();
    assert!(tbg_server::admin::key::parse(&key).is_some());
    let conn = rusqlite::Connection::open(&s.db).unwrap();
    conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE)")
        .unwrap();
    let bytes = std::fs::read(&s.db).unwrap();
    assert!(
        !bytes.windows(key.len()).any(|v| v == key.as_bytes()),
        "IC-16"
    );
    assert!(!bytes.windows(code.len()).any(|v| v == code.as_bytes()));
}

#[tokio::test]
async fn br_ad15_audit_failure_aborts_the_action_and_audit_tables_are_append_only() {
    let s = Server::new().await;
    s.register("Keeper", "keeper@example.invalid").await;
    // Seed a complete successful action, then force failure of the next audit insert.
    moderation::action(&s.db, 1, "chat_restrict", Some(NOW + 3600), 0, NOW)
        .await
        .unwrap();
    for table in ["admin_audit", "moderation_actions"] {
        let db = rusqlite::Connection::open(&s.db).unwrap();
        assert!(db.execute(&format!("DELETE FROM {table}"), []).is_err());
        assert!(db
            .execute(&format!("UPDATE {table} SET id=id"), [])
            .is_err());
    }
    s.sql("CREATE TRIGGER force_audit_failure BEFORE INSERT ON admin_audit BEGIN SELECT RAISE(ABORT,'fixture audit failure'); END;");
    assert!(moderation::action(&s.db, 1, "ban", None, 1, NOW)
        .await
        .is_err());
    assert_eq!(
        s.number("SELECT version FROM account_moderation WHERE account_id=1"),
        1
    );
    assert_eq!(s.number("SELECT COUNT(*) FROM moderation_actions"), 1);
    assert_eq!(
        s.number("SELECT COUNT(*) FROM moderation_events"),
        1,
        "IC-08/S13 event rolled back too"
    );
}

#[tokio::test]
async fn br_ad08_idle_absolute_expiry_and_key_rotation_refuse_old_sessions() {
    let s = operator::server().await;
    let op = operator::claim(&s).await;
    s.clock.set(NOW + 1800);
    assert_eq!(
        op.request("GET", "session", json!(null)).await.status,
        401,
        "idle boundary"
    );
    assert_eq!(s.number("SELECT COUNT(*) FROM operators"), 1);
}

#[tokio::test]
async fn s01_area_inventory_is_complete_with_success_and_denial_per_route() {
    use tbg_server::admin::policy::ROUTES;
    let s = operator::server().await;
    let op = operator::claim(&s).await;
    s.register("Keeper", "keeper@example.invalid").await;
    let cases: Vec<(&str, &str, &str, serde_json::Value)> = vec![
        ("GET", "overview", "overview", json!(null)),
        ("GET", "accounts", "accounts?prefix=Ke", json!(null)),
        ("GET", "accounts/{name}", "accounts/Keeper", json!(null)),
        (
            "POST",
            "accounts/{name}/ban",
            "accounts/Keeper/ban",
            json!({"duration_hours":1,"reason":"other","expected_version":0}),
        ),
        (
            "POST",
            "accounts/{name}/unban",
            "accounts/Keeper/unban",
            json!({"reason":"other","expected_version":1}),
        ),
        (
            "POST",
            "accounts/{name}/chat",
            "accounts/Keeper/chat",
            json!({"duration_hours":1,"reason":"other","expected_version":2}),
        ),
        (
            "POST",
            "accounts/{name}/emoji",
            "accounts/Keeper/emoji",
            json!({"duration_hours":1,"reason":"other","expected_version":3}),
        ),
        (
            "POST",
            "accounts/{name}/revoke-sessions",
            "accounts/Keeper/revoke-sessions",
            json!({"reason":"other"}),
        ),
        (
            "POST",
            "accounts/{name}/send-recovery",
            "accounts/Keeper/send-recovery",
            json!({"reason":"other"}),
        ),
        ("GET", "reports", "reports", json!(null)),
        ("GET", "reports/{id}", "reports/1", json!(null)),
        (
            "POST",
            "reports/{id}/resolve",
            "reports/1/resolve",
            json!({"outcome":"dismissed","note":"reviewed","expected_status":"open"}),
        ),
        ("GET", "emoji", "emoji", json!(null)),
        ("GET", "emoji/{id}", "emoji/1", json!(null)),
        (
            "GET",
            "emoji/{id}/sheet.png",
            "emoji/1/sheet.png",
            json!(null),
        ),
        (
            "POST",
            "emoji/{id}/approve",
            "emoji/1/approve",
            json!({"reason":"reviewed"}),
        ),
        (
            "POST",
            "emoji/{id}/reject",
            "emoji/2/reject",
            json!({"reason":"other"}),
        ),
        (
            "POST",
            "emoji/{id}/revoke",
            "emoji/1/revoke",
            json!({"reason":"other"}),
        ),
        ("GET", "chat/terms", "chat/terms", json!(null)),
        (
            "POST",
            "chat/terms",
            "chat/terms",
            json!({"term":"fixture","reason":"other"}),
        ),
        (
            "DELETE",
            "chat/terms/{term}",
            "chat/terms/fixture",
            json!({"reason":"other"}),
        ),
        ("GET", "settings", "settings", json!(null)),
        (
            "PUT",
            "settings/{key}",
            "settings/chat_enabled",
            json!({"value":1,"reason":"other"}),
        ),
        ("GET", "email-blocks", "email-blocks", json!(null)),
        (
            "POST",
            "email-blocks",
            "email-blocks",
            json!({"email":"blocked@example.invalid","reason":"other"}),
        ),
        (
            "DELETE",
            "email-blocks/{email}",
            "email-blocks/blocked%40example.invalid",
            json!({"reason":"other"}),
        ),
        ("GET", "operators", "operators", json!(null)),
        (
            "POST",
            "operators",
            "operators",
            json!({"email":"mod@example.invalid","role":"moderator"}),
        ),
        (
            "POST",
            "operators/{id}/disable",
            "operators/2/disable",
            json!({"reason":"other"}),
        ),
        (
            "POST",
            "operators/{id}/enable",
            "operators/2/enable",
            json!({"reason":"other"}),
        ),
        ("GET", "audit", "audit", json!(null)),
        ("GET", "system", "system", json!(null)),
    ];
    let inventoried: Vec<_> = ROUTES
        .iter()
        .filter(|r| !r.path.starts_with("claim/") && !r.path.starts_with("session"))
        .map(|r| (r.method, r.path))
        .collect();
    assert_eq!(
        inventoried,
        cases
            .iter()
            .map(|(m, p, _, _)| (*m, *p))
            .collect::<Vec<_>>(),
        "S01 a route without a behavioral matrix row fails"
    );
    s.sql("INSERT INTO accounts(id,name,name_lower,key_hash,created_at) VALUES(2,'Other','other',zeroblob(32),'fixture');INSERT INTO reports(created_unix,reporter_id,target_id,reason,context,evidence_json) VALUES(1000000,1,2,'other','profile','{}');INSERT INTO emoji_assets(id,owner_account_id,content_hash,status,label,sheet,frame_px,frames,columns,delays_json,source_format,source_bytes,created_unix) VALUES(1,1,zeroblob(32),'pending','one',x'89504e47',16,1,1,'[100]','png',10,1000000),(2,1,randomblob(32),'pending','two',x'89504e47',16,1,1,'[100]','png',10,1000000);");
    for (method, _, path, body) in cases {
        let denied = request(
            &op.url,
            method,
            &format!("/admin-api/v1/{path}"),
            body.clone(),
            vec![
                ("origin".into(), "https://admin.tupibg.com".into()),
                ("x-tbg-csrf".into(), "wrong".into()),
            ],
        )
        .await;
        assert_eq!(denied.status, 401, "unauthenticated: {method} {path}");
        let allowed = op.request(method, path, body).await;
        assert_eq!(
            allowed.status, 200,
            "owner: {method} {path}: {}",
            allowed.body
        );
    }
}

#[tokio::test]
async fn s01_moderator_and_disabled_operator_are_checked_against_current_state() {
    let s = operator::server().await;
    let op = operator::claim(&s).await;
    s.register("Keeper", "private@example.invalid").await;
    s.sql("UPDATE operators SET role='moderator' WHERE id=1;");
    let detail = op.request("GET", "accounts/Keeper", json!(null)).await;
    assert_eq!(detail.status, 200);
    assert!(!detail.body.contains("private@example.invalid"), "S17");
    for (method, path, body) in [
        (
            "PUT",
            "settings/chat_enabled",
            json!({"value":1,"reason":"other"}),
        ),
        (
            "POST",
            "operators",
            json!({"email":"m@example.invalid","role":"moderator"}),
        ),
        (
            "POST",
            "accounts/Keeper/ban",
            json!({"duration_hours":31*24,"reason":"other","expected_version":0}),
        ),
        ("GET", "audit", json!(null)),
        ("GET", "email-blocks", json!(null)),
    ] {
        assert_eq!(
            op.request(method, path, body).await.status,
            403,
            "moderator denied {path}"
        );
    }
    assert_eq!(
        op.request(
            "POST",
            "accounts/Keeper/chat",
            json!({"duration_hours":1,"reason":"other","expected_version":0})
        )
        .await
        .status,
        200
    );
    s.sql("UPDATE operators SET status='disabled' WHERE id=1;");
    assert_eq!(op.request("GET", "overview", json!(null)).await.status, 401);
    assert_eq!(
        op.request(
            "POST",
            "accounts/Keeper/chat",
            json!({"duration_hours":1,"reason":"other","expected_version":1})
        )
        .await
        .status,
        401
    );
}

#[tokio::test]
async fn s01_every_route_has_all_five_principals_and_a_wrong_method() {
    use tbg_server::admin::policy::{Capability, ROUTES};
    let s = operator::server().await;
    let op = operator::claim(&s).await;
    let pre = operator::preauth(&s).await;
    s.register("Keeper", "keeper@example.invalid").await;
    // Independent policy inventory: adding a route requires an explicit security decision here.
    let expected = [
        ("POST", "claim/request", true),
        ("POST", "claim/complete", true),
        ("POST", "session/start", true),
        ("POST", "session/verify", true),
        ("GET", "session", true),
        ("POST", "session/reauth", true),
        ("POST", "session/logout", true),
        ("GET", "overview", true),
        ("GET", "accounts", true),
        ("GET", "accounts/{name}", true),
        ("POST", "accounts/{name}/ban", true),
        ("POST", "accounts/{name}/unban", false),
        ("POST", "accounts/{name}/chat", true),
        ("POST", "accounts/{name}/emoji", true),
        ("POST", "accounts/{name}/revoke-sessions", true),
        ("POST", "accounts/{name}/send-recovery", false),
        ("GET", "reports", true),
        ("GET", "reports/{id}", true),
        ("POST", "reports/{id}/resolve", true),
        ("GET", "emoji", true),
        ("GET", "emoji/{id}", true),
        ("GET", "emoji/{id}/sheet.png", true),
        ("POST", "emoji/{id}/approve", true),
        ("POST", "emoji/{id}/reject", true),
        ("POST", "emoji/{id}/revoke", true),
        ("GET", "chat/terms", true),
        ("POST", "chat/terms", true),
        ("DELETE", "chat/terms/{term}", true),
        ("GET", "settings", true),
        ("PUT", "settings/{key}", false),
        ("GET", "email-blocks", false),
        ("POST", "email-blocks", false),
        ("DELETE", "email-blocks/{email}", false),
        ("GET", "operators", false),
        ("POST", "operators", false),
        ("POST", "operators/{id}/disable", false),
        ("POST", "operators/{id}/enable", false),
        ("GET", "audit", false),
        ("GET", "system", true),
    ];
    assert_eq!(
        ROUTES
            .iter()
            .map(|r| (r.method, r.path))
            .collect::<Vec<_>>(),
        expected.iter().map(|r| (r.0, r.1)).collect::<Vec<_>>()
    );
    let before = s.number("SELECT COUNT(*) FROM admin_audit");
    for (route, (_, _, moderator)) in ROUTES.iter().zip(expected) {
        let path = route
            .path
            .replace("{name}", "Keeper")
            .replace("{id}", "9999")
            .replace("{term}", "fixture")
            .replace("{key}", "chat_enabled")
            .replace("{email}", "blocked%40example.invalid");
        let path = if path == "accounts" {
            "accounts?prefix=Ke".into()
        } else {
            path
        };
        for principal in ["none", "expired", "disabled", "moderator", "owner"] {
            s.sql("UPDATE operators SET status='active',role='owner';UPDATE operator_sessions SET last_seen_unix=1000000;");
            if principal == "expired" {
                s.sql("UPDATE operator_sessions SET last_seen_unix=998200;");
            }
            if principal == "disabled" {
                s.sql("UPDATE operators SET status='disabled';");
            }
            if principal == "moderator" {
                s.sql("UPDATE operators SET role='moderator';");
            }
            let public = route.capability == Capability::Public;
            let cookie = if principal == "none" {
                pre.cookie.clone()
            } else {
                format!("{}; {}", pre.cookie, op.cookie)
            };
            let result = request(
                &op.url,
                route.method,
                &format!("/admin-api/v1/{path}"),
                json!({"unexpected":true}),
                vec![
                    ("cookie".into(), cookie),
                    ("origin".into(), "https://admin.tupibg.com".into()),
                    (
                        "x-tbg-csrf".into(),
                        if public {
                            pre.csrf.clone()
                        } else {
                            op.csrf.clone()
                        },
                    ),
                ],
            )
            .await;
            if public {
                assert_eq!(result.status, 400, "{principal} {} {path}", route.method);
            } else if matches!(principal, "none" | "expired" | "disabled") {
                assert_eq!(result.status, 401, "{principal} {path}");
            } else if principal == "moderator" && !moderator {
                assert_eq!(result.status, 403, "moderator {path}");
            } else {
                assert!(
                    matches!(result.status, 200 | 400),
                    "authorized {principal} {} {path}: {} {}",
                    route.method,
                    result.status,
                    result.body
                );
            }
        }
        assert_eq!(
            op.request("TRACE", &path, json!({})).await.status,
            400,
            "wrong method {path}"
        );
    }
    assert_eq!(
        s.number("SELECT COUNT(*) FROM admin_audit"),
        before,
        "invalid bodies never produce effects or audit"
    );
}

#[tokio::test]
async fn s02_s03_forged_actors_and_non_json_posts_have_no_effect() {
    let s = operator::server().await;
    let op = operator::claim(&s).await;
    s.register("Keeper", "keeper@example.invalid").await;
    let before = s.number("SELECT COUNT(*) FROM admin_audit");
    for forged in [
        json!({"duration_hours":1,"reason":"other","expected_version":0,"account_id":2}),
        json!({"duration_hours":1,"reason":"other","expected_version":0,"operator_id":99,"role":"owner"}),
        json!({"duration_hours":1,"reason":"other","expected_version":0,"version":99}),
    ] {
        assert_eq!(
            op.request("POST", "accounts/Keeper/ban", forged)
                .await
                .status,
            400
        );
    }
    for mime in ["text/plain", "application/x-www-form-urlencoded"] {
        let r = request(
            &op.url,
            "POST",
            "/admin-api/v1/session/logout",
            json!({}),
            vec![
                ("cookie".into(), op.cookie.clone()),
                ("origin".into(), "https://admin.tupibg.com".into()),
                ("x-tbg-csrf".into(), op.csrf.clone()),
                ("content-type".into(), mime.into()),
            ],
        )
        .await;
        assert_eq!(r.status, 403);
    }
    assert_eq!(s.number("SELECT COUNT(*) FROM admin_audit"), before);
    assert_eq!(s.number("SELECT COUNT(*) FROM account_moderation"), 0);
    assert_eq!(op.request("GET", "session", json!(null)).await.status, 200);
}

#[tokio::test]
async fn ic03_session_start_is_byte_identical_for_every_outcome() {
    let s = operator::server().await;
    let owner = operator::claim(&s).await;
    let pre = operator::preauth(&s).await;
    let mut responses = Vec::new();
    for key in [&owner.key, "wrong", &owner.key, "wrong"] {
        let r = pre
            .request("POST", "session/start", json!({"admin_key":key}))
            .await;
        responses.push((r.status, r.body));
    }
    s.sql("CREATE TRIGGER mail_outage BEFORE INSERT ON mail_outbox BEGIN SELECT RAISE(ABORT,'mail unavailable'); END;");
    let r = pre
        .request("POST", "session/start", json!({"admin_key":owner.key}))
        .await;
    responses.push((r.status, r.body));
    assert!(
        responses.iter().all(|r| r == &responses[0]),
        "IC-03/S06 includes unavailable outbox"
    );
}

#[tokio::test]
async fn s04_absolute_expiry_claim_rotation_disable_and_restart_end_sessions() {
    let s = operator::server().await;
    let mut op = operator::claim(&s).await;
    s.sql("UPDATE operator_sessions SET absolute_expires_unix=1000000;");
    assert_eq!(op.request("GET", "session", json!(null)).await.status, 401);
    let pre = operator::preauth(&s).await;
    op = operator::sign_in(
        &s,
        operator::Operator {
            key: op.key.clone(),
            ..pre
        },
    )
    .await;
    let pre = operator::preauth(&s).await;
    assert_eq!(
        pre.request(
            "POST",
            "claim/request",
            json!({"email":"owner@example.invalid"})
        )
        .await
        .status,
        202
    );
    let code = s.mail("operator_claim").await.code.unwrap();
    let rotated = pre
        .request(
            "POST",
            "claim/complete",
            json!({"email":"owner@example.invalid","code":code}),
        )
        .await;
    assert_eq!(rotated.status, 200);
    assert_eq!(op.request("GET", "session", json!(null)).await.status, 401);
    let pre = operator::preauth(&s).await;
    let mut op = operator::sign_in(
        &s,
        operator::Operator {
            key: rotated.json()["admin_key"].as_str().unwrap().into(),
            ..pre
        },
    )
    .await;
    s.sql("UPDATE operators SET status='disabled';");
    assert_eq!(op.request("GET", "session", json!(null)).await.status, 401);
    s.sql("UPDATE operators SET status='active';");
    // Disabling through the real route revokes sessions; restart independently revokes all console sessions.
    let _b = Server::on(s.db.clone(), s.clock.clone(), |c| {
        c.admin.enabled = true;
        c.admin.addr = "127.0.0.1:0".into();
    })
    .await;
    assert_eq!(
        _b.request("GET", "/health", json!(null), None).await.status,
        200
    );
    assert_eq!(
        op.request("GET", "session", json!(null)).await.status,
        401,
        "S04 process restart"
    );
    op.key.clear();
}

#[tokio::test]
async fn s05_operator_guesses_share_the_code_budget_and_fresh_claim_recovers() {
    let a = operator::server().await;
    let _ = logs();
    let owner = operator::claim(&a).await;
    let b = Server::on(a.db.clone(), a.clock.clone(), |c| {
        c.admin.enabled = true;
        c.admin.addr = "127.0.0.1:0".into();
    })
    .await;
    let pre = operator::preauth(&a).await;
    let other = operator::preauth(&b).await;
    let start = pre
        .request("POST", "session/start", json!({"admin_key":owner.key}))
        .await;
    let code = a.mail("operator_login").await.code.unwrap();
    for n in 0..5 {
        let client = if n % 2 == 0 { &pre } else { &other };
        assert_eq!(
            client
                .request(
                    "POST",
                    "session/verify",
                    json!({"attempt":start.json()["attempt"],"code":"00000-00000"})
                )
                .await
                .status,
            401
        );
    }
    assert_eq!(
        pre.request(
            "POST",
            "session/verify",
            json!({"attempt":start.json()["attempt"],"code":code})
        )
        .await
        .status,
        401
    );
    assert_eq!(
        pre.request(
            "POST",
            "claim/request",
            json!({"email":"owner@example.invalid"})
        )
        .await
        .status,
        202
    );
    let claim = a.mail("operator_claim").await.code.unwrap();
    let r = pre
        .request(
            "POST",
            "claim/complete",
            json!({"email":"owner@example.invalid","code":claim}),
        )
        .await;
    assert_eq!(r.status, 200);
    let new_key = r.json()["admin_key"].as_str().unwrap().to_owned();
    let secrets = [
        owner.key.as_str(),
        code.as_str(),
        claim.as_str(),
        new_key.as_str(),
    ];
    let captured = logs();
    assert!(captured.contains("P75_LOG_CAPTURE_ACTIVE"));
    for suffix in ["", "-wal"] {
        let bytes = std::fs::read(format!("{}{suffix}", a.db.display())).unwrap_or_default();
        for secret in secrets {
            assert!(
                !bytes.windows(secret.len()).any(|v| v == secret.as_bytes()),
                "IC-16 operator DB/WAL"
            );
            assert!(!captured.contains(secret), "IC-16 captured logs");
        }
    }
}

#[tokio::test]
async fn s12_claim_race_rotates_once_and_audits_one_effect() {
    let s = operator::server().await;
    let pre = operator::preauth(&s).await;
    let code = s.mail("operator_claim").await.code.unwrap();
    let input = json!({"email":"owner@example.invalid","code":code});
    let (one, two) = tokio::join!(
        pre.request("POST", "claim/complete", input.clone()),
        pre.request("POST", "claim/complete", input)
    );
    let mut status = [one.status, two.status];
    status.sort();
    assert_eq!(status, [200, 401]);
    assert_eq!(
        s.number("SELECT COUNT(*) FROM admin_audit WHERE action='key_rotated'"),
        1
    );
    assert_eq!(s.number("SELECT key_epoch FROM operators"), 1);
}

#[tokio::test]
async fn s05_admin_key_attempts_across_ips_share_the_operator_budget_but_allow_fresh_claim() {
    let s = operator::server().await;
    let owner = operator::claim(&s).await;
    let pre = operator::preauth(&s).await;
    let before = s.number("SELECT COUNT(*) FROM mail_outbox WHERE purpose='operator_login'");
    for ip in 1..=12 {
        let result = request(
            &pre.url,
            "POST",
            "/admin-api/v1/session/start",
            json!({"admin_key":owner.key}),
            vec![
                ("cookie".into(), pre.cookie.clone()),
                ("x-tbg-csrf".into(), pre.csrf.clone()),
                ("origin".into(), "https://admin.tupibg.com".into()),
                ("x-forwarded-for".into(), format!("192.0.2.{ip}")),
            ],
        )
        .await;
        assert_eq!(result.status, 202);
    }
    assert!(
        s.number("SELECT COUNT(*) FROM mail_outbox WHERE purpose='operator_login'") - before <= 9,
        "operator's first sign-in already used one of ten slots"
    );
    assert_eq!(
        pre.request(
            "POST",
            "claim/request",
            json!({"email":"owner@example.invalid"})
        )
        .await
        .status,
        202
    );
    let code = s.mail("operator_claim").await.code.unwrap();
    assert_eq!(
        pre.request(
            "POST",
            "claim/complete",
            json!({"email":"owner@example.invalid","code":code})
        )
        .await
        .status,
        200
    );
}

#[tokio::test]
async fn ic08_recovery_audit_update_and_delete_are_refused() {
    let s = Server::new().await;
    s.register("Keeper", "keeper@example.invalid").await;
    let db = rusqlite::Connection::open(&s.db).unwrap();
    assert!(db
        .execute("UPDATE recovery_audit SET action='forged'", [])
        .is_err());
    assert!(db.execute("DELETE FROM recovery_audit", []).is_err());
    assert_eq!(s.number("SELECT COUNT(*) FROM recovery_audit"), 1);
}

#[tokio::test]
async fn s04_disabled_moderator_sessions_stay_revoked_after_reenable() {
    let s = operator::server().await;
    let owner = operator::claim(&s).await;
    assert_eq!(
        owner
            .request(
                "POST",
                "operators",
                json!({"email":"mod@example.invalid","role":"moderator"})
            )
            .await
            .status,
        200
    );
    let pre = operator::preauth(&s).await;
    let code = s.mail("operator_claim").await.code.unwrap();
    let key = pre
        .request(
            "POST",
            "claim/complete",
            json!({"email":"mod@example.invalid","code":code}),
        )
        .await
        .json()["admin_key"]
        .as_str()
        .unwrap()
        .to_owned();
    let moderator = operator::sign_in(&s, operator::Operator { key, ..pre }).await;
    assert_eq!(
        moderator
            .request("GET", "session", json!(null))
            .await
            .status,
        200
    );
    assert_eq!(
        owner
            .request("POST", "operators/2/disable", json!({"reason":"other"}))
            .await
            .status,
        200
    );
    assert_eq!(
        moderator
            .request("GET", "session", json!(null))
            .await
            .status,
        401
    );
    assert_eq!(
        owner
            .request("POST", "operators/2/enable", json!({"reason":"other"}))
            .await
            .status,
        200
    );
    assert_eq!(
        moderator
            .request("GET", "session", json!(null))
            .await
            .status,
        401,
        "S04 enabling an operator never resurrects a session"
    );
}

#[tokio::test]
async fn s07_controls_rtl_and_zero_width_comments_notes_and_terms_are_refused() {
    let s = operator::server().await;
    let owner = operator::claim(&s).await;
    let player = s.register("Reporter", "reporter@example.invalid").await;
    s.register("Target", "target@example.invalid").await;
    let before = s.number("SELECT COUNT(*) FROM admin_audit");
    for value in [
        "bad\r\nvalue",
        "bad\u{202e}value",
        "bad\u{200b}value",
        "bad\0value",
    ] {
        assert_eq!(
            s.request(
                "POST",
                "/api/me/reports",
                json!({"context":"profile","target":"Target","reason":"other","comment":value}),
                player["access_token"].as_str()
            )
            .await
            .status,
            400
        );
        assert_eq!(
            owner
                .request(
                    "POST",
                    "accounts/Target/chat",
                    json!({"duration_hours":1,"expected_version":0,"reason":"other","note":value})
                )
                .await
                .status,
            400
        );
        assert_eq!(
            owner
                .request("POST", "chat/terms", json!({"term":value,"reason":"other"}))
                .await
                .status,
            400
        );
    }
    assert_eq!(s.number("SELECT COUNT(*) FROM reports"), 0);
    assert_eq!(s.number("SELECT COUNT(*) FROM account_moderation"), 0);
    assert_eq!(s.number("SELECT COUNT(*) FROM admin_audit"), before);
    let markup = "<script>alert('literal text')</script>";
    assert_eq!(
        s.request(
            "POST",
            "/api/me/reports",
            json!({"context":"profile","target":"Target","reason":"other","comment":markup}),
            player["access_token"].as_str()
        )
        .await
        .status,
        201
    );
    let report = owner.request("GET", "reports/1", json!(null)).await;
    assert_eq!(
        report.json()["comment"],
        markup,
        "operators receive text; the console's text-only gate forbids markup interpretation"
    );
}
