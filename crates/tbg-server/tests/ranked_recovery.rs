//! P60 records and recovery (GAME-DESIGN §14 BR-RI, D-87): v6 -> v7
//! migration, idempotent settlement by match key, ordered sequences,
//! whole-transaction rollback and the durable journal round trip.

use tbg_core::{ArcanaPreset, EndReason, Outcome, Seat};
use tbg_server::ranked::{journal, EndDetail, IntegritySeat, JournalEntry, MatchKey, RankedFinal};
use tbg_server::store::{AccountId, SettleError, Store};

fn temp_dir(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("tbg-p60-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn hash_of(byte: u8) -> [u8; 32] {
    [byte; 32]
}

fn final_for(key: MatchKey, accounts: [AccountId; 2], winner: Option<Seat>) -> RankedFinal {
    RankedFinal {
        key,
        accounts,
        outcome: Outcome {
            winner,
            reason: EndReason::Forfeit,
        },
        end_detail: EndDetail::Surrender,
        combat_ticks: 4_000,
        wall_ms: 215_000,
        reached_running: true,
        ruleset_version: tbg_core::ruleset_version::RULESET_VERSION,
        ruleset_source_hash: tbg_server::ranked::source_hash(),
        seats: [
            IntegritySeat {
                purchases: 9,
                evolutions: 2,
                final_hp: 0,
                final_max_hp: 5_000,
                ..IntegritySeat::default()
            },
            IntegritySeat {
                surrender_tick: Some(3_990),
                final_max_hp: 5_000,
                ..IntegritySeat::default()
            },
        ],
        factions: [[3, 2, 2, 2, 0, 0, 0], [0; 7]],
        arcanas: [ArcanaPreset::DEFAULT; 2],
        stats: None,
    }
}

async fn two_accounts(store: &Store) -> [AccountId; 2] {
    let (a, _) = store.register("Alda".into(), hash_of(1)).await.unwrap();
    let (b, _) = store.register("Bram".into(), hash_of(2)).await.unwrap();
    [a, b]
}

#[tokio::test]
async fn settlement_is_idempotent_by_match_key() {
    let dir = temp_dir("idempotent");
    let store = Store::open(&dir.join("db.sqlite3")).unwrap();
    let accounts = two_accounts(&store).await;
    let seq = store.ranked_high_water().await.unwrap() + 1;
    let entry = JournalEntry {
        completion_seq: seq,
        ended_unix: 1_790_000_000,
        result: final_for([7; 16], accounts, Some(Seat::A)),
    };
    let first = store.settle_ranked(entry.clone()).await.unwrap();
    assert_eq!(first.sequence, seq);
    assert_eq!((first.results[0].0, first.results[0].1), (1000, 1020));
    assert_eq!((first.results[1].0, first.results[1].1), (1000, 980));
    // A retry after commit (crash before journal delete) applies nothing.
    let again = store.settle_ranked(entry.clone()).await.unwrap();
    assert_eq!(again, first);
    assert_eq!(store.ranked_match_count().await.unwrap(), 1);
    // Same key, different envelope: quarantined, never applied.
    let forged = JournalEntry {
        completion_seq: seq + 1,
        result: final_for([7; 16], accounts, Some(Seat::B)),
        ..entry.clone()
    };
    assert!(matches!(
        store.settle_ranked(forged).await,
        Err(SettleError::ConflictingDuplicate)
    ));
    // Equal frozen accounts are an invariant failure, not a draw.
    let same = JournalEntry {
        completion_seq: seq + 1,
        result: final_for([8; 16], [accounts[0]; 2], None),
        ..entry
    };
    assert!(matches!(
        store.settle_ranked(same).await,
        Err(SettleError::SameAccount)
    ));
    assert_eq!(store.ranked_match_count().await.unwrap(), 1);
}

#[tokio::test]
async fn a_failed_insert_rolls_back_both_accounts() {
    let dir = temp_dir("rollback");
    let store = Store::open(&dir.join("db.sqlite3")).unwrap();
    let accounts = two_accounts(&store).await;
    let entry = JournalEntry {
        completion_seq: 1,
        ended_unix: 1_790_000_000,
        result: final_for([1; 16], accounts, Some(Seat::A)),
    };
    store.settle_ranked(entry.clone()).await.unwrap();
    // A new key reusing a committed sequence fails inside the transaction,
    // after both account updates ran: neither may persist.
    let clash = JournalEntry {
        result: final_for([2; 16], accounts, Some(Seat::B)),
        ..entry
    };
    assert!(matches!(
        store.settle_ranked(clash).await,
        Err(SettleError::Sql(_))
    ));
    let (_, a) = store.login("alda".into(), hash_of(1)).await.unwrap();
    let (_, b) = store.login("bram".into(), hash_of(2)).await.unwrap();
    assert_eq!((a.rating, a.ranked_games), (1020, 1));
    assert_eq!((b.rating, b.ranked_games), (980, 1));
    assert_eq!(store.ranked_high_water().await.unwrap(), 1);
}

#[tokio::test]
async fn v6_database_migrates_to_v8_twice_without_touching_history() {
    let dir = temp_dir("migrate");
    let path = dir.join("db.sqlite3");
    {
        let conn = rusqlite::Connection::open(&path).unwrap();
        conn.execute_batch(V6_FIXTURE).unwrap();
    }
    for _ in 0..2 {
        let store = Store::open(&path).unwrap();
        drop(store);
    }
    let conn = rusqlite::Connection::open(&path).unwrap();
    let version: i64 = conn
        .query_row("SELECT MAX(version) FROM schema_version", [], |r| r.get(0))
        .unwrap();
    assert_eq!(version, 8);
    let baseline: (i64, i64, i64) = conn
        .query_row(
            "SELECT rating, ranked_games, mature_games FROM ranked_baseline WHERE account_id = 1",
            [],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .unwrap();
    assert_eq!(baseline, (1100, 25, 25), "legacy games initialize maturity");
    let old: (i64, i64, Option<Vec<u8>>) = conn
        .query_row(
            "SELECT a_before, a_after, match_key FROM ranked_matches WHERE id = 1",
            [],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .unwrap();
    assert_eq!(old, (1080, 1100, None), "pre-v7 rows unchanged, no key");
    let fk: i64 = conn
        .query_row("SELECT count(*) FROM pragma_foreign_key_check", [], |r| {
            r.get(0)
        })
        .unwrap();
    assert_eq!(fk, 0);
    // The decision log is append-only at the SQL level.
    conn.execute(
        "INSERT INTO ranked_decisions (operation_key, match_id, decided_unix, actor, action,
         reason, before_json, after_json) VALUES ('op', 1, 0, 'owner', 'review', 'r', '{}', '{}')",
        [],
    )
    .unwrap();
    assert!(conn
        .execute("UPDATE ranked_decisions SET reason = 'x'", [])
        .is_err());
    assert!(conn.execute("DELETE FROM ranked_decisions", []).is_err());
}

#[test]
fn journal_round_trip_and_ordering() {
    let dir = temp_dir("journal");
    let entries: Vec<JournalEntry> = [3i64, 1, 2]
        .into_iter()
        .map(|seq| JournalEntry {
            completion_seq: seq,
            ended_unix: 1_790_000_000 + seq,
            result: final_for([seq as u8; 16], [10, 11], Some(Seat::B)),
        })
        .collect();
    for e in &entries {
        assert_eq!(journal::decode(&journal::encode(e)).as_ref(), Some(e));
        journal::write(&dir, e).unwrap();
    }
    let loaded = journal::load_all(&dir).unwrap();
    let order: Vec<i64> = loaded.iter().map(|e| e.completion_seq).collect();
    assert_eq!(order, [1, 2, 3], "replay follows completion order");
    journal::remove(&dir, &loaded[0]).unwrap();
    journal::remove(&dir, &loaded[0]).unwrap(); // idempotent
    journal::quarantine(&dir, &loaded[1]).unwrap();
    let left: Vec<i64> = journal::load_all(&dir)
        .unwrap()
        .iter()
        .map(|e| e.completion_seq)
        .collect();
    assert_eq!(left, [3], "quarantined files never replay");
}

/// A minimal v6 database: the shipped v1–v6 tables, one account with 25
/// legacy ranked games and one legacy ranked row.
const V6_FIXTURE: &str = "
CREATE TABLE schema_version(version INTEGER NOT NULL);
INSERT INTO schema_version VALUES (1),(2),(3),(4),(5),(6);
CREATE TABLE matches (id INTEGER PRIMARY KEY, ended_at TEXT NOT NULL,
  duration_s INTEGER NOT NULL, winner TEXT, reason TEXT NOT NULL, seed INTEGER NOT NULL,
  seat_a TEXT NOT NULL, seat_b TEXT NOT NULL, step_us_mean INTEGER NOT NULL,
  step_us_max INTEGER NOT NULL, late_ticks INTEGER NOT NULL, max_lateness_ms INTEGER NOT NULL,
  a_arcanas BLOB, b_arcanas BLOB, arena INTEGER);
CREATE TABLE accounts (id INTEGER PRIMARY KEY, name TEXT NOT NULL,
  name_lower TEXT NOT NULL UNIQUE, key_hash BLOB NOT NULL, created_at TEXT NOT NULL,
  last_login_at TEXT, rating INTEGER NOT NULL DEFAULT 1000,
  peak_rating INTEGER NOT NULL DEFAULT 1000, ranked_games INTEGER NOT NULL DEFAULT 0,
  wins INTEGER NOT NULL DEFAULT 0, losses INTEGER NOT NULL DEFAULT 0,
  draws INTEGER NOT NULL DEFAULT 0, arcanas BLOB, stronghold INTEGER);
CREATE TABLE ranked_matches (id INTEGER PRIMARY KEY, ended_at TEXT NOT NULL,
  a_id INTEGER NOT NULL REFERENCES accounts(id), b_id INTEGER NOT NULL REFERENCES accounts(id),
  winner INTEGER, reason TEXT NOT NULL, duration_s INTEGER NOT NULL,
  a_before INTEGER NOT NULL, a_after INTEGER NOT NULL,
  b_before INTEGER NOT NULL, b_after INTEGER NOT NULL,
  a_factions TEXT NOT NULL, b_factions TEXT NOT NULL, a_arcanas BLOB, b_arcanas BLOB);
CREATE INDEX ranked_by_a ON ranked_matches(a_id, id);
CREATE INDEX ranked_by_b ON ranked_matches(b_id, id);
CREATE INDEX accounts_by_rating ON accounts(rating DESC, wins DESC, name_lower);
INSERT INTO accounts (id, name, name_lower, key_hash, created_at, rating, peak_rating,
  ranked_games, wins, losses, draws)
  VALUES (1, 'Old', 'old', x'00', '2026-01-01T00:00:00Z', 1100, 1120, 25, 15, 10, 0),
         (2, 'Foe', 'foe', x'01', '2026-01-01T00:00:00Z', 900, 1000, 25, 10, 15, 0);
INSERT INTO ranked_matches (id, ended_at, a_id, b_id, winner, reason, duration_s,
  a_before, a_after, b_before, b_after, a_factions, b_factions)
  VALUES (1, '2026-02-01T00:00:00Z', 1, 2, 0, 'Forfeit', 60, 1080, 1100, 920, 900, '{}', '{}');
";
