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
