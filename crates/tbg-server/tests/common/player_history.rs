//! Explicit disposable P73 fixture accounts and authoritative terminal envelopes.
#![allow(dead_code)]
use std::{
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};
use tbg_core::protocol::{MatchMode, MatchOrigin, PersonalMatchSummary, TerminalStronghold};
use tbg_server::store::{
    player_history::{Envelope, HistorySide},
    Store,
};
static SERIAL: AtomicU64 = AtomicU64::new(0);
pub fn path() -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "tbg-p73-fixture-{}-{}",
        std::process::id(),
        SERIAL.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir_all(&dir).unwrap();
    dir.join("fixture.sqlite")
}

pub async fn store() -> (Store, PathBuf, [i64; 2]) {
    let path = path();
    let store = Store::open_history(&path, Default::default(), true).unwrap();
    let a = store.register("FixtureA".into(), [1; 32]).await.unwrap().0;
    let b = store.register("FixtureB".into(), [2; 32]).await.unwrap().0;
    (store, path, [a, b])
}
pub fn envelope(number: u32, accounts: [i64; 2]) -> Envelope {
    let mut key = [0; 16];
    key[..4].copy_from_slice(&number.to_le_bytes());
    let sides = [0, 1].map(|seat| {
        let mut id = key;
        id[15] = seat as u8 + 1;
        HistorySide {
            entry_id: id,
            account: Some(accounts[seat]),
            name: format!("Fixture{}", if seat == 0 { "A" } else { "B" }),
            summary: PersonalMatchSummary {
                version: 1,
                quality: "complete".into(),
                mode: MatchMode::Casual,
                origin: MatchOrigin::PrivateRoom,
                ended_unix: 1000,
                arena: Some("forest".into()),
                ruleset_version: Some(1),
                source_hash: Some("fixture".into()),
                combat_ticks: Some(100),
                tick_hz: 20,
                wall_ms: Some(10000),
                outcome: if seat == 0 { "win" } else { "loss" }.into(),
                reason: "surrender".into(),
                purchases: Some(0),
                champions: Some(Vec::new()),
                evolutions: Some(0),
                fortifications: Some(0),
                augments: Some(Vec::new()),
                stronghold: Some(TerminalStronghold {
                    kind: "bastion".into(),
                    hp: if seat == 0 { 100 } else { 0 },
                    max_hp: 100,
                }),
            },
        }
    });
    Envelope {
        format: 1,
        key: format!("result:{}", tbg_server::ranked::hex(&key)),
        result_id: Some(key),
        ranked_key: None,
        ranked_row: None,
        winner: Some(0),
        sides,
    }
}
pub fn owner(account: i64, name: &str) -> tbg_server::store::player_history::Subject {
    tbg_server::store::player_history::Subject::Owner {
        account,
        name: name.into(),
    }
}
