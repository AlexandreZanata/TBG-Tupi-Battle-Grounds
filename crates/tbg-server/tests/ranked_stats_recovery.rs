//! P61 §5 optional summaries (BR-ST, D-88): written after the authoritative
//! settlement, insert-or-verify, never able to change a rating, and carried
//! through the P60 journal unchanged.

use serde_json::json;
use tbg_core::{ArcanaPreset, EndReason, Outcome, Seat};
use tbg_server::ranked::{journal, EndDetail, IntegritySeat, JournalEntry, RankedFinal};
use tbg_server::store::{StatsWriteError, Store};

fn temp_dir(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("tbg-p61-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn side(units: serde_json::Value) -> serde_json::Value {
    let arcanas = [0u8; 15];
    json!({
        "units": units, "synergy_max": [1, 0, 0, 0],
        "synergy_ticks": [[200, 0], [0, 0], [0, 0], [0, 0]],
        "offered": [[3, 1]], "chosen": [[3, 1]], "rounds": [1, 0],
        "stronghold": [2, 0, 0], "arcanas": arcanas, "complete": true,
    })
}

fn summary(bought: u32) -> serde_json::Value {
    json!({
        "schema": 1, "catalog_hash": "00", "arena": 0, "combat_ticks": 1200, "wall_ms": 63000,
        "sides": [side(json!([[0, bought, 40, 1, 1, 1, 0]])), side(json!([]))],
    })
}

fn entry(accounts: [i64; 2], stats: Option<serde_json::Value>) -> JournalEntry {
    JournalEntry {
        completion_seq: 1,
        ended_unix: 1_790_000_000,
        result: RankedFinal {
            key: [9; 16],
            accounts,
            outcome: Outcome {
                winner: Some(Seat::A),
                reason: EndReason::StrongholdDestroyed,
            },
            end_detail: EndDetail::Stronghold,
            combat_ticks: 1_200,
            wall_ms: 63_000,
            reached_running: true,
            ruleset_version: tbg_core::ruleset_version::RULESET_VERSION,
            ruleset_source_hash: tbg_server::ranked::source_hash(),
            seats: [IntegritySeat::default(); 2],
            factions: [[1, 0, 0, 0, 0, 0, 0], [0; 7]],
            arcanas: [ArcanaPreset::DEFAULT; 2],
            stats,
        },
    }
}

#[tokio::test]
async fn summaries_are_insert_or_verify_and_never_touch_ratings() {
    let dir = temp_dir("verify");
    let store = Store::open(&dir.join("db.sqlite3")).unwrap();
    let (a, _) = store.register("Ana".into(), [1; 32]).await.unwrap();
    let (b, _) = store.register("Bo".into(), [2; 32]).await.unwrap();
    let receipt = store
        .settle_ranked(entry([a, b], Some(summary(2))))
        .await
        .unwrap();
    let id = receipt.sequence;
    assert_eq!(store.save_ranked_stats(id, summary(2)).await, Ok(()));
    assert_eq!(
        store.save_ranked_stats(id, summary(2)).await,
        Ok(()),
        "identical replay"
    );
    assert_eq!(
        store.save_ranked_stats(id, summary(3)).await,
        Err(StatsWriteError::ConflictingDuplicate),
        "evidence is never overwritten"
    );
    let mut bad = summary(1);
    bad["sides"][0]["units"] = json!([[0, 1, 40, 1, 1, 2, 0]]); // tier1 > bought
    assert_eq!(
        store.save_ranked_stats(id, bad).await,
        Err(StatsWriteError::InvalidSummary)
    );
    // No ranked match 999: the optional row fails, nothing else changes.
    assert_eq!(
        store.save_ranked_stats(999, summary(1)).await,
        Err(StatsWriteError::Unavailable)
    );
    let (_, profile) = store.login("ana".into(), [1; 32]).await.unwrap();
    assert_eq!((profile.rating, profile.ranked_games), (1020, 1));
    assert_eq!(store.ranked_match_count().await.unwrap(), 1);
}

#[test]
fn the_journal_carries_the_optional_summary_unchanged() {
    let dir = temp_dir("journal");
    for stats in [None, Some(summary(2))] {
        let e = entry([4, 5], stats);
        assert_eq!(journal::decode(&journal::encode(&e)), Some(e.clone()));
        journal::write(&dir, &e).unwrap();
        assert_eq!(journal::load_all(&dir).unwrap(), vec![e.clone()]);
        journal::remove(&dir, &e).unwrap();
    }
}
