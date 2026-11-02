//! Schema 10 consolidates two parallel v9 migrations: P66's release
//! directory/ownership and P68's D-97 auth families. Files from either
//! front (both marked 9), from v8 or already combined must upgrade without
//! losing accounts, ratings, results, keys, families or release rows; a
//! partial or foreign shape must refuse to open instead of trusting `9`.

use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use serde_json::json;
use tbg_server::store::{auth_sessions::*, Store};

static NEXT: AtomicU64 = AtomicU64::new(0);

fn path() -> PathBuf {
    std::env::temp_dir().join(format!(
        "tbg-schema10-{}-{}.sqlite",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ))
}

const KEY_HASH: [u8; 32] = [7; 32];

/// A combined file holding every kind of durable row the upgrade must keep.
async fn populated() -> PathBuf {
    let p = path();
    let store = Store::open(&p).unwrap();
    let Reply::Grant { .. } = store
        .auth(Request::Issue(Issuance {
            name: "Upgrader".into(),
            key_hash: KEY_HASH,
            register: true,
            family: [1; 16],
            device: [3; 16],
            channel: Channel::Native,
            persistent: true,
            refresh_hash: [11; 32],
            now: 1_000,
        }))
        .await
        .unwrap()
    else {
        panic!("initial grant")
    };
    store.insert_match(
        &json!({"duration_s": 61, "winner": "a", "reason": "stronghold",
        "seed": 7, "seats": ["Upgrader", "Guest"]}),
    );
    assert_eq!(store.match_count().await.unwrap(), 1);
    drop(store);
    sql(
        &p,
        "UPDATE accounts SET rating = 1234 WHERE name_lower = 'upgrader';
         INSERT INTO release_slots (slot, generation, server_build, contract, endpoint, state,
           heartbeat_unix) VALUES ('a', 'gen-a', 'build-a', 'w30-r2-3d56156049e626c5',
           '/ws/slot-a', 'active', 1);
         INSERT INTO account_owner (account_id, slot, generation, epoch, heartbeat_unix)
           SELECT id, 'a', 'gen-a', 1, 1 FROM accounts WHERE name_lower = 'upgrader';
         INSERT INTO release_audit (at_unix, actor, action, slot, detail)
           VALUES (1, 'operator', 'promote', 'a', 'synthetic');",
    );
    p
}

fn sql(p: &PathBuf, batch: &str) {
    let db = rusqlite::Connection::open(p).unwrap();
    db.execute_batch(&format!("PRAGMA foreign_keys=ON; {batch}"))
        .unwrap();
}

fn count(p: &PathBuf, query: &str) -> i64 {
    let db = rusqlite::Connection::open(p).unwrap();
    db.query_row(query, [], |r| r.get(0)).unwrap()
}

/// Accounts, rating, key, result and baseline rows survive, and the file
/// is now at 10 with both feature table sets and the BR-AS03 index.
async fn assert_combined(p: &PathBuf) {
    let store = Store::open(p).unwrap();
    let (_, profile) = store
        .login("upgrader".into(), KEY_HASH)
        .await
        .expect("the manual AccountKey still signs in");
    assert_eq!(profile.rating, 1234);
    assert_eq!(store.match_count().await.unwrap(), 1);
    drop(store);
    assert_eq!(count(p, "SELECT MAX(version) FROM schema_version"), 10);
    assert_eq!(count(p, "SELECT COUNT(*) FROM ranked_baseline"), 1);
    for table in [
        "release_slots",
        "release_audit",
        "account_owner",
        "auth_sessions",
        "auth_refresh_tokens",
    ] {
        let present = count(
            p,
            &format!("SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='{table}'"),
        );
        assert_eq!(present, 1, "{table}");
    }
    assert_eq!(
        count(
            p,
            "SELECT COUNT(*) FROM sqlite_master WHERE type='index' AND name='auth_refresh_current'"
        ),
        1,
        "BR-AS03: one current refresh per family stays enforced"
    );
}

#[tokio::test]
async fn p66_v9_file_gains_the_auth_families_and_keeps_its_release_rows() {
    let p = populated().await;
    // Exactly P66's v9: release tables, no auth tables, version 9.
    sql(
        &p,
        "DROP TABLE auth_refresh_tokens; DROP TABLE auth_sessions;
         DELETE FROM schema_version WHERE version = 10;",
    );
    assert_combined(&p).await;
    assert_eq!(count(&p, "SELECT COUNT(*) FROM release_slots"), 1);
    assert_eq!(count(&p, "SELECT COUNT(*) FROM account_owner"), 1);
    assert_eq!(count(&p, "SELECT COUNT(*) FROM release_audit"), 1);
    assert_eq!(count(&p, "SELECT COUNT(*) FROM auth_sessions"), 0);
    let _ = std::fs::remove_file(p);
}

#[tokio::test]
async fn p68_v9_file_gains_the_release_directory_and_keeps_its_families() {
    let p = populated().await;
    // Exactly P68's v9: auth tables, no release tables, version 9.
    sql(
        &p,
        "DROP TABLE account_owner; DROP TABLE release_audit; DROP TABLE release_slots;
         DELETE FROM schema_version WHERE version = 10;",
    );
    assert_combined(&p).await;
    assert_eq!(count(&p, "SELECT COUNT(*) FROM auth_sessions"), 1);
    assert_eq!(count(&p, "SELECT COUNT(*) FROM auth_refresh_tokens"), 1);
    assert_eq!(count(&p, "SELECT COUNT(*) FROM release_slots"), 0);
    // The surviving refresh still rotates: the family is usable after upgrade.
    let store = Store::open(&p).unwrap();
    assert!(store
        .auth(Request::Rotate {
            old_hash: [11; 32],
            new_hash: [12; 32],
            operation_hash: [13; 32],
            channel: Channel::Native,
            now: 1_100,
        })
        .await
        .is_ok());
    drop(store);
    let _ = std::fs::remove_file(p);
}

#[tokio::test]
async fn v8_file_gains_both_sets_and_a_combined_file_reopens_unchanged() {
    let p = populated().await;
    sql(
        &p,
        "DROP TABLE auth_refresh_tokens; DROP TABLE auth_sessions;
         DROP TABLE account_owner; DROP TABLE release_audit; DROP TABLE release_slots;
         DELETE FROM schema_version WHERE version IN (9, 10);",
    );
    assert_combined(&p).await;
    let versions = count(&p, "SELECT COUNT(*) FROM schema_version");
    assert_combined(&p).await;
    assert_eq!(
        count(&p, "SELECT COUNT(*) FROM schema_version"),
        versions,
        "re-running on a combined file is a no-op"
    );
    let _ = std::fs::remove_file(p);
}

/// BR-AP11/13 (D-103): the opt-in schema 11 is additive on a file reached
/// from either v9 family; it keeps every older row, defaults accounts to
/// private, reopens without the flag and is a no-op when installed twice.
#[tokio::test]
async fn opt_in_history_schema_11_applies_over_both_reconciled_v9_families() {
    for drop_one_family in [
        "DROP TABLE auth_refresh_tokens; DROP TABLE auth_sessions;",
        "DROP TABLE account_owner; DROP TABLE release_audit; DROP TABLE release_slots;",
    ] {
        let p = populated().await;
        sql(
            &p,
            &format!("{drop_one_family} DELETE FROM schema_version WHERE version = 10;"),
        );
        assert_combined(&p).await;
        let store = Store::open_history(&p, Default::default(), true).unwrap();
        let (_, profile) = store
            .login("upgrader".into(), KEY_HASH)
            .await
            .expect("the AccountKey still signs in on schema 11");
        assert_eq!(profile.rating, 1234);
        drop(store);
        assert_eq!(count(&p, "SELECT MAX(version) FROM schema_version"), 11);
        assert_eq!(
            count(
                &p,
                "SELECT COUNT(*) FROM accounts
                 WHERE history_visibility = 'private' AND history_privacy_revision = 0"
            ),
            count(&p, "SELECT COUNT(*) FROM accounts"),
            "BR-AP11: existing accounts stay Only me"
        );
        assert_eq!(count(&p, "SELECT COUNT(*) FROM ranked_baseline"), 1);
        assert_eq!(count(&p, "SELECT COUNT(*) FROM matches"), 1);
        let versions = count(&p, "SELECT COUNT(*) FROM schema_version");
        drop(Store::open(&p).expect("a flag-off restart opens schema 11"));
        drop(Store::open_history(&p, Default::default(), true).unwrap());
        assert_eq!(
            count(&p, "SELECT COUNT(*) FROM schema_version"),
            versions,
            "re-running the opt-in migration is a no-op"
        );
        let _ = std::fs::remove_file(p);
    }
}

#[tokio::test]
async fn a_partial_or_foreign_v9_shape_refuses_to_open() {
    let partial = populated().await;
    sql(
        &partial,
        "DROP TABLE auth_refresh_tokens; DELETE FROM schema_version WHERE version = 10;",
    );
    assert!(Store::open(&partial).is_err(), "partial auth set");
    let _ = std::fs::remove_file(partial);

    let foreign = populated().await;
    sql(
        &foreign,
        "DROP TABLE account_owner; DROP TABLE release_audit; DROP TABLE release_slots;
         CREATE TABLE release_slots (slot TEXT PRIMARY KEY, note TEXT);
         CREATE TABLE release_audit (id INTEGER PRIMARY KEY);
         CREATE TABLE account_owner (account_id INTEGER PRIMARY KEY);
         DELETE FROM schema_version WHERE version = 10;",
    );
    assert!(
        Store::open(&foreign).is_err(),
        "unrelated tables named alike"
    );
    // Refusal changed nothing: the version and the auth rows are intact.
    assert_eq!(
        count(&foreign, "SELECT MAX(version) FROM schema_version"),
        9
    );
    assert_eq!(count(&foreign, "SELECT COUNT(*) FROM auth_sessions"), 1);
    let _ = std::fs::remove_file(foreign);
}

/// IC-13: a v12 marker cannot hide the separately opted-in history v11.
#[tokio::test]
async fn p75_history_installs_after_12_and_reopens_without_changes() {
    let p = populated().await;
    let before = count(&p, "SELECT COUNT(*) FROM accounts");
    drop(Store::open_history(&p, Default::default(), true).unwrap());
    assert_eq!(
        count(
            &p,
            "SELECT COUNT(*) FROM schema_version WHERE version IN (11,12)"
        ),
        2
    );
    assert_eq!(count(&p, "SELECT COUNT(*) FROM accounts"), before);
    let db = rusqlite::Connection::open(&p).unwrap();
    let dump = |db: &rusqlite::Connection| -> Vec<(String, String)> {
        let mut stmt = db
            .prepare("SELECT name,sql FROM sqlite_master WHERE sql IS NOT NULL ORDER BY name")
            .unwrap();
        stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?)))
            .unwrap()
            .map(Result::unwrap)
            .collect()
    };
    let shape = dump(&db);
    drop(Store::open_history(&p, Default::default(), true).unwrap());
    assert_eq!(dump(&db), shape);
    assert_eq!(count(&p, "SELECT COUNT(*) FROM admin_settings"), 6);
    let _ = std::fs::remove_file(p);
}

/// IC-13/S13: partial P75 tables or missing append-only guards fail closed.
#[tokio::test]
async fn p75_partial_or_foreign_shape_aborts_without_repair() {
    for damage in [
        "DROP TABLE operator_sessions",
        "DROP TRIGGER admin_audit_no_update",
        "DROP INDEX operators_single_owner",
    ] {
        let p = populated().await;
        sql(&p, damage);
        assert!(Store::open(&p).is_err(), "{damage}");
        assert_eq!(count(&p, "SELECT COUNT(*) FROM accounts"), 1);
        let _ = std::fs::remove_file(p);
    }
}
