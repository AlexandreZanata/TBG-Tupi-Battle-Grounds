//! BR-ID01–ID11, IC-04/14 and S06/S16: private email identity through real HTTP.
#[path = "common/p75.rs"]
mod common;
use common::*;
use serde_json::json;

#[tokio::test]
async fn br_id01_email_is_required_and_invalid_email_does_not_consume_registration_slots() {
    let server = Server::configured(|c| c.register_per_ip_hour = 1).await;
    for email in [
        "",
        "a\r\nb@example.invalid",
        "a@xn--bad.invalid",
        "a@localhost",
    ] {
        let r = server
            .request(
                "POST",
                "/api/auth/native/register",
                registration("Keeper", email),
                None,
            )
            .await;
        assert_eq!(r.status, 400);
        assert_eq!(server.number("SELECT COUNT(*) FROM accounts"), 0);
    }
    let grant = server.register("Keeper", "A+keep@EXAMPLE.invalid").await;
    let access = grant["access_token"].as_str().unwrap();
    let panel = server
        .request("GET", "/api/me/account", json!(null), Some(access))
        .await;
    assert_eq!(panel.json()["email_state"], "unverified");
    assert_eq!(panel.json()["email_masked"], "A***p@example.invalid");
    let mail = server.mail("account_verify").await;
    let verified = server
        .request(
            "POST",
            "/api/me/email/verify",
            json!({"code":mail.code.unwrap()}),
            Some(access),
        )
        .await;
    assert_eq!(verified.status, 200);
    assert_eq!(
        server
            .request("GET", "/api/me/account", json!(null), Some(access))
            .await
            .json()["email_state"],
        "verified"
    );
}

#[tokio::test]
async fn br_id08_sixth_address_is_refused_and_id09_blocks_exact_normalized_addresses() {
    let s = Server::new().await;
    for n in 0..5 {
        s.register(&format!("Sibling{n}"), "family@example.invalid")
            .await;
    }
    let r = s
        .request(
            "POST",
            "/api/auth/native/register",
            registration("Sixth", "family@example.invalid"),
            None,
        )
        .await;
    assert_eq!(r.status, 409);
    assert_eq!(r.json()["error"], "email_limit");
    s.sql("INSERT INTO operators(email,role,status,created_unix,origin) VALUES('o@example.invalid','owner','pending_claim',0,'bootstrap');INSERT INTO email_blocks VALUES('blocked@example.invalid',0,1,'other');");
    let r = s
        .request(
            "POST",
            "/api/auth/native/register",
            registration("Blocked", "blocked@EXAMPLE.invalid"),
            None,
        )
        .await;
    assert_eq!(r.status, 403);
    assert_eq!(s.number("SELECT COUNT(*) FROM accounts"), 5);
}

#[tokio::test]
async fn br_id04_email_never_leaves_private_routes() {
    let s = Server::new().await;
    let canary = format!("canary{}@example.invalid", random(16));
    let grant = s.register("PrivateMail", &canary).await;
    for path in [
        "/api/leaderboard",
        "/api/players/PrivateMail",
        "/api/live",
        "/api/online",
        "/api/players/PrivateMail/matches",
        "/api/patch-notes",
        "/metrics",
    ] {
        let r = s.request("GET", path, json!(null), None).await;
        assert!(!r.body.contains(&canary), "IC-04: {path}");
    }
    assert!(!grant.to_string().contains(&canary));
    assert!(!format!("{:?}", tbg_server::Config::default()).contains(&canary));
}

#[tokio::test]
async fn br_id05_legacy_add_and_routine_change_require_key_and_both_codes() {
    let s = Server::new().await;
    let grant = s.register("OldPlayer", "old@example.invalid").await;
    let key = grant["account_key"].as_str().unwrap();
    let access = grant["access_token"].as_str().unwrap();
    let code = s.mail("account_verify").await.code.unwrap();
    assert_eq!(
        s.request(
            "POST",
            "/api/me/email/verify",
            json!({"code":code}),
            Some(access)
        )
        .await
        .status,
        200
    );
    assert_eq!(
        s.request(
            "POST",
            "/api/me/email",
            json!({"account_key":"wrong","email":"new@example.invalid","locale":"en"}),
            Some(access)
        )
        .await
        .status,
        401
    );
    assert_eq!(
        s.request(
            "POST",
            "/api/me/email",
            json!({"account_key":key,"email":"new@example.invalid","locale":"en"}),
            Some(access)
        )
        .await
        .status,
        202
    );
    let old = s.mail("account_change_old").await.code.unwrap();
    let new = s.mail("account_change_new").await.code.unwrap();
    assert_eq!(
        s.request(
            "POST",
            "/api/me/email/change/confirm",
            json!({"old_code":old,"new_code":new}),
            Some(access)
        )
        .await
        .status,
        200
    );
    assert_eq!(
        s.request("GET", "/api/me/account", json!(null), Some(access))
            .await
            .json()["email_masked"],
        "n***w@example.invalid"
    );
}

#[tokio::test]
async fn id06_mistyped_old_code_keeps_the_valid_new_code_usable() {
    let s = Server::new().await;
    let g = s.register("Keeper", "old@example.invalid").await;
    let access = g["access_token"].as_str();
    let code = s.mail("account_verify").await.code.unwrap();
    assert_eq!(
        s.request("POST", "/api/me/email/verify", json!({"code":code}), access)
            .await
            .status,
        200
    );
    assert_eq!(
        s.request(
            "POST",
            "/api/me/email",
            json!({"account_key":g["account_key"],"email":"new@example.invalid","locale":"en"}),
            access
        )
        .await
        .status,
        202
    );
    let old = s.mail("account_change_old").await.code.unwrap();
    let new = s.mail("account_change_new").await.code.unwrap();
    assert_eq!(
        s.request(
            "POST",
            "/api/me/email/change/confirm",
            json!({"old_code":"00000-00000","new_code":new}),
            access
        )
        .await
        .status,
        401
    );
    assert_eq!(
        s.request(
            "POST",
            "/api/me/email/change/confirm",
            json!({"old_code":old,"new_code":new}),
            access
        )
        .await
        .status,
        200,
        "both proofs remain retryable after a typo"
    );
}

#[tokio::test]
async fn ic14_fifth_registration_keeps_its_same_response_retry() {
    let s = Server::new().await;
    for i in 0..4 {
        s.register(&format!("Sibling{i}"), "family@example.invalid")
            .await;
    }
    let input = registration("LastSibling", "family@example.invalid");
    let first = s
        .request("POST", "/api/auth/native/register", input.clone(), None)
        .await;
    assert_eq!(first.status, 200);
    let retry = s
        .request("POST", "/api/auth/native/register", input, None)
        .await;
    assert_eq!(
        retry.body, first.body,
        "email cap never breaks the D-97 committed grant retry"
    );
}

#[tokio::test]
async fn s16_headers_controls_and_unsupported_locales_are_refused() {
    let s = Server::configured(|c| c.register_per_ip_hour = 1).await;
    for input in [
        registration("Bad\r\nName", "valid@example.invalid"),
        registration("Keeper", "a@example.invalid\r\nBcc:x@example.invalid"),
        {
            let mut input = registration("Keeper", "valid@example.invalid");
            input["locale"] = json!("en\r\nInjected:yes");
            input
        },
    ] {
        assert_eq!(
            s.request("POST", "/api/auth/native/register", input, None)
                .await
                .status,
            400
        );
    }
    assert_eq!(s.number("SELECT COUNT(*) FROM accounts"), 0);
    s.register("Keeper", "valid@example.invalid").await;
}
