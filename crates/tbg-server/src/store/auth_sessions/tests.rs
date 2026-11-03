//! D-97/BR-AS02–03: auth writes serialize with another slot before reading their snapshot.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use super::*;
use crate::store::{recovery::service::Enrollment, Store};
use rusqlite::Connection;

fn path() -> PathBuf {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    std::env::temp_dir().join(format!(
        "tbg-auth-writer-{}-{}.sqlite3",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ))
}

fn registration() -> Request {
    Request::IssueEmail {
        issue: Issuance {
            name: "WriterRace".into(),
            key_hash: [7; 32],
            register: true,
            family: [1; 16],
            device: [2; 16],
            channel: Channel::Native,
            persistent: true,
            refresh_hash: [3; 32],
            now: 1_000,
        },
        email: Enrollment {
            email: "race@example.invalid".into(),
            locale: "en".into(),
        },
    }
}

/// A second slot's real write reservation precedes the auth operation.
/// SQLite cannot busy-wait a deferred read-to-write upgrade; it can wait at BEGIN IMMEDIATE.
async fn during_peer_write(store: &Store, path: &Path, request: Request) -> Result<Reply, Error> {
    let peer = Connection::open(path).unwrap();
    peer.execute_batch("BEGIN IMMEDIATE; UPDATE admin_settings SET updated_unix=updated_unix+1 WHERE key='registration_enabled'").unwrap();
    let committed = tokio::task::spawn_blocking(move || {
        std::thread::sleep(Duration::from_millis(150));
        peer.execute_batch("COMMIT").unwrap();
    });
    let reply = store.auth(request).await;
    committed.await.unwrap();
    reply
}

#[tokio::test]
async fn registration_waits_for_a_peer_writer_and_keeps_one_complete_identity() {
    let path = path();
    let store = Store::open(&path).unwrap();
    let reply = during_peer_write(&store, &path, registration())
        .await
        .expect("BR-P01: a brief peer write must not reject a valid registration");
    let Reply::Grant { family, .. } = reply else {
        panic!("registration must issue its family");
    };
    assert!(store.login("writerrace".into(), [7; 32]).await.is_ok());
    let db = Connection::open(&path).unwrap();
    for table in [
        "accounts",
        "ranked_baseline",
        "recovery_identity",
        "auth_sessions",
        "auth_refresh_tokens",
        "mail_outbox",
    ] {
        let count: i64 = db
            .query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |r| r.get(0))
            .unwrap();
        assert_eq!(
            count, 1,
            "IC-01: registration commits exactly one {table} row"
        );
    }
    let account: i64 = db
        .query_row("SELECT account_id FROM recovery_identity", [], |r| r.get(0))
        .unwrap();
    assert_eq!(account, family.account);
}

#[tokio::test]
async fn refresh_waits_for_a_peer_writer_and_preserves_the_family_and_account() {
    let path = path();
    let store = Store::open(&path).unwrap();
    let Reply::Grant { family: before, .. } = store.auth(registration()).await.unwrap() else {
        panic!("initial family");
    };
    let request = Request::Rotate {
        old_hash: [3; 32],
        new_hash: [4; 32],
        operation_hash: [5; 32],
        channel: Channel::Native,
        now: 1_010,
    };
    let reply = during_peer_write(&store, &path, request)
        .await
        .expect("BR-AS02/03: a peer write must not discard a valid refresh");
    let Reply::Grant { family: after, .. } = reply else {
        panic!("refresh must issue its successor");
    };
    assert_eq!(
        (after.id, after.account, after.expires_at),
        (before.id, before.account, before.expires_at)
    );
    assert!(matches!(
        store
            .auth(Request::Confirm {
                hash: [3; 32],
                channel: Channel::Native,
                now: 1_010
            })
            .await,
        Err(Error::Unauthorized)
    ));
    assert!(matches!(
        store
            .auth(Request::Confirm {
                hash: [4; 32],
                channel: Channel::Native,
                now: 1_010
            })
            .await,
        Ok(Reply::Grant { .. })
    ));
    assert!(store.login("writerrace".into(), [7; 32]).await.is_ok());
    let db = Connection::open(&path).unwrap();
    let (accounts, families, history, current): (i64, i64, i64, i64) = db.query_row("SELECT (SELECT COUNT(*) FROM accounts),(SELECT COUNT(*) FROM auth_sessions),(SELECT COUNT(*) FROM auth_refresh_tokens),(SELECT COUNT(*) FROM auth_refresh_tokens WHERE consumed_at IS NULL)",[],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?))).unwrap();
    assert_eq!(
        (accounts, families, history, current),
        (1, 1, 2, 1),
        "BR-AS03: one family/current hash, retained consumed history"
    );
}
