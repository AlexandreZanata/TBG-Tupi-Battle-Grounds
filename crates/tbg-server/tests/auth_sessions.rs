//! D-97/BR-AS02–03/10: hash-only atomic refresh history preserves ranked accounts.

use std::sync::atomic::{AtomicU64, Ordering};
use tbg_server::store::{auth_sessions::*, Store};

static NEXT: AtomicU64 = AtomicU64::new(0);

fn path() -> std::path::PathBuf {
    std::env::temp_dir().join(format!(
        "tbg-p68-auth-{}-{}.sqlite",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ))
}

fn issue(register: bool, family: u8, hash: u8, now: i64) -> Request {
    Request::Issue(Issuance {
        name: "P68Tester".into(),
        key_hash: [7; 32],
        register,
        family: [family; 16],
        device: [3; 16],
        channel: Channel::Native,
        persistent: true,
        refresh_hash: [hash; 32],
        now,
    })
}

#[tokio::test]
async fn br_as02_rotation_keeps_absolute_expiry_and_consumed_history() {
    let p = path();
    let store = Store::open(&p).unwrap();
    let Reply::Grant {
        family, profile, ..
    } = store.auth(issue(true, 1, 11, 1000)).await.unwrap()
    else {
        panic!("initial grant")
    };
    assert_eq!(profile.rating, 1000);
    assert_eq!(family.expires_at, 1000 + FAMILY_SECONDS);
    let request = || Request::Rotate {
        old_hash: [11; 32],
        new_hash: [12; 32],
        operation_hash: [41; 32],
        channel: Channel::Native,
        now: 1010,
    };
    let Reply::Grant {
        family: rotated, ..
    } = store.auth(request()).await.unwrap()
    else {
        panic!("rotated grant")
    };
    assert_eq!(
        rotated.expires_at, family.expires_at,
        "BR-AS02: rotation never extends the family"
    );
    assert!(matches!(
        store.auth(request()).await.unwrap(),
        Reply::Consumed {
            at: 1010,
            operation_matches: true,
            ..
        }
    ));
    assert!(matches!(
        store
            .auth(Request::Confirm {
                hash: [11; 32],
                channel: Channel::Native,
                now: 1010
            })
            .await,
        Err(Error::Unauthorized)
    ));
    assert!(matches!(
        store
            .auth(Request::Confirm {
                hash: [12; 32],
                channel: Channel::Browser,
                now: 1010
            })
            .await,
        Err(Error::Unauthorized)
    ));
    assert!(matches!(
        store
            .auth(Request::Confirm {
                hash: [12; 32],
                channel: Channel::Native,
                now: family.expires_at
            })
            .await,
        Err(Error::Unauthorized)
    ));
    let db = rusqlite::Connection::open(&p).unwrap();
    let generations: i64 = db
        .query_row("SELECT COUNT(*) FROM auth_refresh_tokens", [], |r| r.get(0))
        .unwrap();
    assert_eq!(generations, 2, "one successor and retained consumed hash");
    let baseline: i64 = db
        .query_row("SELECT COUNT(*) FROM ranked_baseline", [], |r| r.get(0))
        .unwrap();
    assert_eq!(baseline, 1, "BR-AS10: existing ranked baseline preserved");
    drop(db);
    drop(store);
    let _ = std::fs::remove_file(p);
}

#[tokio::test]
async fn br_as03_concurrent_consumption_creates_exactly_one_successor() {
    let p = path();
    let store = Store::open(&p).unwrap();
    store.auth(issue(true, 1, 11, 1000)).await.unwrap();
    let rotate = |hash, op| Request::Rotate {
        old_hash: [11; 32],
        new_hash: [hash; 32],
        operation_hash: [op; 32],
        channel: Channel::Native,
        now: 1001,
    };
    let (a, b) = tokio::join!(store.auth(rotate(12, 40)), store.auth(rotate(13, 41)));
    let replies = [a.unwrap(), b.unwrap()];
    assert_eq!(
        replies
            .iter()
            .filter(|r| matches!(r, Reply::Grant { .. }))
            .count(),
        1
    );
    assert_eq!(
        replies
            .iter()
            .filter(|r| matches!(
                r,
                Reply::Consumed {
                    operation_matches: false,
                    ..
                }
            ))
            .count(),
        1
    );
    store
        .auth(Request::Revoke {
            families: vec![[1; 16]],
            now: 1002,
            reason: "test",
        })
        .await
        .unwrap();
    for hash in [11, 12, 13] {
        assert!(matches!(
            store
                .auth(Request::Confirm {
                    hash: [hash; 32],
                    channel: Channel::Native,
                    now: 1002
                })
                .await,
            Err(Error::Unauthorized)
        ));
    }
    drop(store);
    let _ = std::fs::remove_file(p);
}

#[tokio::test]
async fn br_as11_v8_upgrade_and_repeat_preserve_existing_accounts_and_ranked_baseline() {
    let p = path();
    let store = Store::open(&p).unwrap();
    store.register("Existing".into(), [9; 32]).await.unwrap();
    let db = rusqlite::Connection::open(&p).unwrap();
    db.execute_batch("PRAGMA foreign_keys=ON; DROP TABLE auth_refresh_tokens; DROP TABLE auth_sessions; DELETE FROM schema_version WHERE version IN (9,10);").unwrap();
    drop(db);
    drop(store);
    let upgraded = Store::open(&p).unwrap();
    assert!(upgraded.login("existing".into(), [9; 32]).await.is_ok());
    let db = rusqlite::Connection::open(&p).unwrap();
    let version: i64 = db
        .query_row("SELECT MAX(version) FROM schema_version", [], |r| r.get(0))
        .unwrap();
    assert_eq!(version, 10);
    let baseline: i64 = db
        .query_row("SELECT COUNT(*) FROM ranked_baseline", [], |r| r.get(0))
        .unwrap();
    assert_eq!(baseline, 1);
    drop(db);
    drop(upgraded);
    let reopened = Store::open(&p).unwrap();
    assert!(reopened.login("existing".into(), [9; 32]).await.is_ok());
    drop(reopened);
    let _ = std::fs::remove_file(p);
}
