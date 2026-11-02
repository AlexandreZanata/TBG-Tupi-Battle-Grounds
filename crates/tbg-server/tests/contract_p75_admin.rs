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
    let player = s.register("Player", "player@example.invalid").await;
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
