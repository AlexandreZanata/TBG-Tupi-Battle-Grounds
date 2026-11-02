//! BR-RC01–RC08 and IC-01/02/03/15/16/20: recovery preserves the stable AccountId.
#[path = "common/p75.rs"]
mod common;
use common::*;
use serde_json::{json, Value};
use std::collections::BTreeMap;

async fn verified(s: &Server) -> Value {
    let grant = s.register("Keeper", "keeper@example.invalid").await;
    let code = s.mail("account_verify").await.code.unwrap();
    assert_eq!(
        s.request(
            "POST",
            "/api/me/email/verify",
            json!({"code":code}),
            grant["access_token"].as_str()
        )
        .await
        .status,
        200
    );
    grant
}
async fn recovery_code(s: &Server) -> String {
    assert_eq!(
        s.request(
            "POST",
            "/api/auth/native/recovery-request",
            json!({"name":"Keeper","email":"keeper@example.invalid","locale":"en"}),
            None
        )
        .await
        .status,
        202
    );
    s.mail("account_recovery").await.code.unwrap()
}
fn body(code: &str) -> Value {
    json!({"name":"Keeper","code":code,"device_id":random(16),"persistent":false,"registration_retry_key":random(32)})
}

/// Discover every direct account FK at runtime, including future unrelated tables.
fn snapshot(path: &std::path::Path) -> BTreeMap<String, Vec<String>> {
    let db = rusqlite::Connection::open(path).unwrap();
    let tables: Vec<String> = {
        let mut s=db.prepare("SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%' ORDER BY name").unwrap();
        s.query_map([], |r| r.get(0))
            .unwrap()
            .map(Result::unwrap)
            .collect()
    };
    let mut out = BTreeMap::new();
    for table in tables {
        let bound: bool = db
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM pragma_foreign_key_list(?1) WHERE [table]='accounts')",
                [&table],
                |r| r.get(0),
            )
            .unwrap();
        if !bound
            || table.starts_with("recovery_")
            || table == "auth_sessions"
            || table == "account_owner"
        {
            continue;
        }
        let mut stmt = db
            .prepare(&format!(
                "SELECT * FROM \"{}\" ORDER BY rowid",
                table.replace('"', "\"\"")
            ))
            .unwrap();
        let columns = stmt.column_count();
        let rows = stmt
            .query_map([], |r| {
                Ok((0..columns)
                    .map(|i| format!("{:?}", r.get_ref(i).unwrap()))
                    .collect::<Vec<_>>()
                    .join("|"))
            })
            .unwrap()
            .map(Result::unwrap)
            .collect();
        out.insert(table, rows);
    }
    out
}

#[tokio::test]
async fn br_rc04_recovery_preserves_every_account_row() {
    let s = Server::new().await;
    let old = verified(&s).await;
    s.sql("UPDATE accounts SET rating=1234,wins=9,arcanas=x'010203',stronghold=2 WHERE id=1;CREATE TABLE future_inventory(account_id INTEGER REFERENCES accounts(id),owned BLOB);INSERT INTO future_inventory VALUES(1,x'00112233');INSERT INTO account_emoji_slots VALUES(1,0,3,NULL,1000000);INSERT INTO account_moderation(account_id,chat_until_unix,updated_unix) VALUES(1,2000000,1000000);");
    let code = recovery_code(&s).await;
    let before = snapshot(&s.db);
    let created = s.number("SELECT COUNT(*) FROM accounts");
    let input = body(&code);
    let r = s
        .request("POST", "/api/auth/native/recover", input.clone(), None)
        .await;
    assert_eq!(r.status, 200, "{}", r.body);
    let new = r.json();
    assert_eq!(
        snapshot(&s.db),
        before,
        "IC-01 includes every discovered account FK"
    );
    assert_eq!(s.number("SELECT COUNT(*) FROM accounts"), created);
    assert_eq!(new["profile"]["rating"], 1234);
    let key = new["account_key"].as_str().unwrap();
    assert!(tbg_core::account::parse_key(key).is_some(), "IC-20");
    assert_ne!(new["account_key"], old["account_key"]);
    let retry = s
        .request("POST", "/api/auth/native/recover", input, None)
        .await;
    assert_eq!(retry.body, r.body, "D-97 same retry returns the same grant");
    let conn = rusqlite::Connection::open(&s.db).unwrap();
    conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE)")
        .unwrap();
    for suffix in ["", "-wal"] {
        let bytes = std::fs::read(format!("{}{suffix}", s.db.display())).unwrap_or_default();
        for secret in [
            key,
            code.as_str(),
            new["refresh_token"].as_str().unwrap(),
            new["access_token"].as_str().unwrap(),
        ] {
            assert!(
                !bytes.windows(secret.len()).any(|v| v == secret.as_bytes()),
                "IC-16/S10"
            );
        }
    }
}

#[tokio::test]
async fn br_rc05_old_key_and_sessions_die_on_every_slot() {
    let a = Server::new().await;
    let old = verified(&a).await;
    let b = Server::on(a.db.clone(), a.clock.clone(), |_| {}).await;
    let old_key = old["account_key"].as_str().unwrap();
    let signed = b
        .request(
            "POST",
            "/api/auth/native/login",
            json!({"account_key":old_key,"device_id":random(16),"persistent":false}),
            None,
        )
        .await
        .json();
    let code = recovery_code(&a).await;
    let recovered = a
        .request("POST", "/api/auth/native/recover", body(&code), None)
        .await;
    assert_eq!(recovered.status, 200);
    assert_eq!(
        b.request(
            "POST",
            "/api/auth/native/login",
            json!({"account_key":old_key,"device_id":random(16),"persistent":false}),
            None
        )
        .await
        .status,
        401
    );
    assert_eq!(
        b.request(
            "POST",
            "/api/auth/native/refresh",
            json!({"refresh_token":signed["refresh_token"],"operation_id":random(16)}),
            None
        )
        .await
        .status,
        401
    );
    tokio::time::sleep(std::time::Duration::from_millis(1200)).await;
    assert_eq!(
        b.request(
            "GET",
            "/api/me/account",
            json!(null),
            signed["access_token"].as_str()
        )
        .await
        .status,
        401
    );
    assert_eq!(b.request("POST","/api/auth/native/login",json!({"account_key":recovered.json()["account_key"],"device_id":random(16),"persistent":false}),None).await.status,200);
}

#[tokio::test]
async fn br_rc01_request_is_indistinguishable() {
    let s = Server::new().await;
    let _ = verified(&s).await;
    let mut answers = Vec::new();
    for (name, email) in [
        ("Unknown", "keeper@example.invalid"),
        ("Keeper", "other@example.invalid"),
        ("Keeper", "keeper@example.invalid"),
        ("Keeper", "keeper@example.invalid"),
        ("Keeper", "keeper@example.invalid"),
        ("Keeper", "keeper@example.invalid"),
    ] {
        let r = s
            .request(
                "POST",
                "/api/auth/native/recovery-request",
                json!({"name":name,"email":email,"locale":"en"}),
                None,
            )
            .await;
        answers.push((r.status, r.body));
    }
    assert!(
        answers.iter().all(|v| v == &answers[0]),
        "IC-03/S06 includes throttled attempts"
    );
    s.sql("DROP TABLE mail_outbox");
    let r = s
        .request(
            "POST",
            "/api/auth/native/recovery-request",
            json!({"name":"Keeper","email":"keeper@example.invalid","locale":"en"}),
            None,
        )
        .await;
    assert_eq!(
        (r.status, r.body),
        answers[0],
        "S13: store outage remains generic"
    );
}

#[tokio::test]
async fn br_rc03_codes_are_single_use_bound_expiring_and_limited_to_five_guesses() {
    let s = Server::new().await;
    let grant = s.register("Keeper", "keeper@example.invalid").await;
    let verification = s.mail("account_verify").await.code.unwrap();
    assert_eq!(
        s.request(
            "POST",
            "/api/auth/native/recover",
            body(&verification),
            None
        )
        .await
        .status,
        401,
        "IC-15 wrong purpose"
    );
    assert_eq!(
        s.request(
            "POST",
            "/api/me/email/verify",
            json!({"code":verification}),
            grant["access_token"].as_str()
        )
        .await
        .status,
        200
    );
    let code = recovery_code(&s).await;
    for _ in 0..5 {
        assert_eq!(
            s.request(
                "POST",
                "/api/auth/native/recover",
                body("00000-00000"),
                None
            )
            .await
            .status,
            401
        );
    }
    assert_eq!(
        s.request("POST", "/api/auth/native/recover", body(&code), None)
            .await
            .status,
        401,
        "S05 fifth guess invalidates even the real code"
    );
    s.clock.set(NOW + 3601);
    let code = recovery_code(&s).await;
    s.clock.set(NOW + 3601 + 900);
    assert_eq!(
        s.request("POST", "/api/auth/native/recover", body(&code), None)
            .await
            .status,
        401,
        "expiry is exclusive"
    );
}
