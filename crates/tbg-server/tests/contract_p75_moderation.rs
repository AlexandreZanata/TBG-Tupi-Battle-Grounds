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
