//! Synthetic operator actions reuse the production atomic store path.
#![allow(dead_code)]
use serde_json::json;
use tbg_server::store::{audit, moderation, Store};
pub async fn action(
    path: &std::path::Path,
    account: i64,
    kind: &str,
    until: Option<i64>,
    expected: i64,
    now: i64,
) -> Result<moderation::State, moderation::Error> {
    let store = Store::open(path).unwrap();
    let kind = kind.to_owned();
    store.release(move |db| {
        db.execute("INSERT OR IGNORE INTO operators(id,email,role,status,created_unix,origin) VALUES(1,'owner@example.invalid','owner','pending_claim',0,'bootstrap')",[]).unwrap();
        moderation::apply(db,moderation::Action{account,kind:kind.clone(),until,reason:"other".into(),note:Some("private note".into()),report:None,expected,audit:audit::Entry{operator:Some(1),actor:"operator:1".into(),action:kind,kind:"account".into(),target:account.to_string(),reason:Some("other".into()),note:None,before:json!({"version":expected}),after:json!({"version":expected+1}),request:"0".repeat(32)}},now)
    }).await.unwrap()
}
