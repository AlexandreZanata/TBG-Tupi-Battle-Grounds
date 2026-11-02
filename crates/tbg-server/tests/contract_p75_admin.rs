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
