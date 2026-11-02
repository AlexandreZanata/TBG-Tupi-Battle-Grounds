//! P66 admission decisions: exact contracts only, same-contract canary by
//! bucket, update-required for a contract the directory no longer admits.

use super::*;

fn row(slot: &str, state: &str, contract: &str, pct: u8, beat: i64) -> SlotRow {
    SlotRow {
        slot: slot.into(),
        generation: format!("g{slot}"),
        server_build: "test".into(),
        contract: contract.into(),
        endpoint: format!("/ws/slot-{slot}"),
        state: state.into(),
        canary_pct: pct,
        ranked_authority: false,
        matches_running: 0,
        results_pending: 0,
        ranked_inflight: 0,
        heartbeat_unix: beat,
        version: 1,
    }
}

fn own() -> SlotIdentity {
    SlotIdentity {
        slot: "a".into(),
        generation: "ga".into(),
        server_build: "test".into(),
        contract: "c1".into(),
        endpoint: "/ws/slot-a".into(),
    }
}

fn slot_of(d: Decision) -> String {
    match d {
        Decision::Admit { slot, .. } => slot,
        other => panic!("not admitted: {other:?}"),
    }
}

#[test]
fn a_single_server_admits_only_its_own_contract() {
    let admit = decide(&[], None, "c1", "c1", 0, 100);
    assert_eq!(slot_of(admit), "");
    assert_eq!(
        decide(&[], None, "c1", "c0", 0, 100),
        Decision::UpdateRequired {
            contract: "c1".into()
        }
    );
}

#[test]
fn same_contract_canary_takes_low_buckets_only() {
    let rows = [
        row("a", "active", "c1", 0, 100),
        row("b", "canary", "c1", 25, 100),
    ];
    assert_eq!(
        slot_of(decide(&rows, Some(&own()), "c1", "c1", 10, 100)),
        "b"
    );
    assert_eq!(
        slot_of(decide(&rows, Some(&own()), "c1", "c1", 25, 100)),
        "a"
    );
    assert_eq!(
        slot_of(decide(&rows, Some(&own()), "c1", "c1", 99, 100)),
        "a"
    );
}

#[test]
fn after_a_contract_switch_old_clients_must_update_and_nothing_stale_admits() {
    let rows = [
        row("a", "draining", "c1", 0, 100),
        row("b", "active", "c2", 0, 100),
    ];
    assert_eq!(
        decide(&rows, Some(&own()), "c1", "c1", 0, 100),
        Decision::UpdateRequired {
            contract: "c2".into()
        }
    );
    assert_eq!(
        slot_of(decide(&rows, Some(&own()), "c1", "c2", 0, 100)),
        "b"
    );
    // A slot that stopped reporting never receives new players.
    let silent = [row("b", "active", "c2", 0, 0)];
    assert_eq!(
        decide(&silent, Some(&own()), "c1", "c2", 0, 100),
        Decision::Unavailable
    );
    // Draining only: new play pauses (the safe-drain fallback).
    let paused = [row("a", "draining", "c1", 0, 100)];
    assert_eq!(
        decide(&paused, Some(&own()), "c1", "c1", 0, 100),
        Decision::Unavailable
    );
}

/// A slot that has just started answers from the database instead of its empty view, so a player is
/// never told "unavailable" while the other slot admits (release into the slot the proxy tries first).
#[tokio::test]
async fn a_slot_that_has_not_read_the_directory_yet_reads_it_instead_of_turning_players_away() {
    let dir = std::env::temp_dir().join(format!("tbg-admission-view-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let store = std::sync::Arc::new(Store::open(&dir.join("db.sqlite3")).unwrap());
    let other = SlotIdentity {
        slot: "b".into(),
        generation: "gb".into(),
        server_build: "test".into(),
        contract: "c1".into(),
        endpoint: "/ws/slot-b".into(),
    };
    store
        .release(move |conn| {
            crate::store::release::register(conn, &other, now_unix()).unwrap();
            conn.execute("UPDATE release_slots SET state = 'active'", [])
                .unwrap();
        })
        .await
        .unwrap();
    let empty = std::sync::Arc::new(Vec::new());

    let rows = directory_view(empty.clone(), true, Some(&store)).await;
    assert_eq!(
        slot_of(decide(&rows, Some(&own()), "c1", "c1", 0, now_unix())),
        "b"
    );
    let alone = directory_view(empty.clone(), true, None).await;
    assert_eq!(
        decide(&alone, Some(&own()), "c1", "c1", 0, now_unix()),
        Decision::Unavailable
    );
    let single = directory_view(empty, false, Some(&store)).await;
    assert!(
        single.is_empty(),
        "a single server answers for itself without a directory"
    );
    std::fs::remove_dir_all(dir).ok();
}
