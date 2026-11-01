//! P66 directory, ownership and operator transitions on a real schema.

use rusqlite::{params, Connection};

use super::super::release_admin::{self as admin, AdminError};
use super::*;

fn db() -> Connection {
    let mut conn = Connection::open_in_memory().unwrap();
    super::super::schema::migrate(&mut conn).unwrap();
    for id in 1..=3 {
        conn.execute(
            "INSERT INTO accounts (id, name, name_lower, key_hash, created_at)
             VALUES (?1, ?2, ?3, x'00', '2026-10-31T00:00:00Z')",
            params![id, format!("Player{id}"), format!("player{id}")],
        )
        .unwrap();
    }
    conn
}

fn ident(slot: &str, generation: &str, contract: &str) -> SlotIdentity {
    SlotIdentity {
        slot: slot.into(),
        generation: generation.into(),
        server_build: "test".into(),
        contract: contract.into(),
        endpoint: format!("/ws/slot-{slot}"),
    }
}

fn state(conn: &Connection, slot: &str) -> SlotRow {
    read_directory(conn)
        .unwrap()
        .into_iter()
        .find(|r| r.slot == slot)
        .unwrap()
}

fn beat(conn: &Connection, slot: &str, generation: &str, load: SlotLoad) {
    assert!(heartbeat(conn, slot, generation, load, now_unix()).unwrap());
}

#[test]
fn a_slot_registers_warming_and_a_restart_keeps_its_operator_state() {
    let mut conn = db();
    let row = register(&mut conn, &ident("a", "g1", "c1"), now_unix()).unwrap();
    assert_eq!(row.state, "warming");
    assert!(mark_ready(&conn, "a", "g1").unwrap());
    admin::promote(&mut conn, "a", "test", None).unwrap();
    // A crash restart of the same generation returns to `active`.
    assert_eq!(
        register(&mut conn, &ident("a", "g1", "c1"), now_unix())
            .unwrap()
            .state,
        "active"
    );
    // A different generation in the slot starts over, without authority.
    let fresh = register(&mut conn, &ident("a", "g2", "c1"), now_unix()).unwrap();
    assert_eq!(
        (fresh.state.as_str(), fresh.ranked_authority),
        ("warming", false)
    );
    // The old generation's heartbeat is fenced out.
    assert!(!heartbeat(&conn, "a", "g1", SlotLoad::default(), now_unix()).unwrap());
}

#[test]
fn a_live_session_on_another_slot_keeps_the_account() {
    let mut conn = db();
    register(&mut conn, &ident("a", "ga", "c1"), now_unix()).unwrap();
    register(&mut conn, &ident("b", "gb", "c1"), now_unix()).unwrap();
    assert_eq!(
        claim_owner(&mut conn, 1, "a", "ga", now_unix()).unwrap(),
        Ok(1)
    );
    assert_eq!(
        claim_owner(&mut conn, 1, "b", "gb", now_unix()).unwrap(),
        Err(OwnerConflict::OtherSlot)
    );
    // B releasing never removes A's hold.
    release_owner(&conn, 1, "b").unwrap();
    assert!(owns(&conn, 1, "a", 1).unwrap());
    // Once A stops reporting, B takes over with a higher epoch (the fence).
    let later = now_unix() + STALE_SECS + 5;
    beat(&conn, "b", "gb", SlotLoad::default());
    conn.execute(
        "UPDATE release_slots SET heartbeat_unix = ?1 WHERE slot = 'b'",
        params![later],
    )
    .unwrap();
    assert_eq!(claim_owner(&mut conn, 1, "b", "gb", later).unwrap(), Ok(2));
    assert!(!owns(&conn, 1, "a", 1).unwrap());
    // A retired holder never blocks either.
    assert_eq!(
        claim_owner(&mut conn, 2, "a", "ga", now_unix()).unwrap(),
        Ok(1)
    );
    conn.execute(
        "UPDATE release_slots SET state = 'retired' WHERE slot = 'a'",
        [],
    )
    .unwrap();
    assert_eq!(
        claim_owner(&mut conn, 2, "b", "gb", now_unix()).unwrap(),
        Ok(2)
    );
}

#[test]
fn canary_needs_the_same_contract_and_promotion_drains_the_old_slot() {
    let mut conn = db();
    for (slot, generation, contract) in [("a", "ga", "c1"), ("b", "gb", "c1"), ("c", "gc", "c2")] {
        register(&mut conn, &ident(slot, generation, contract), now_unix()).unwrap();
        mark_ready(&conn, slot, generation).unwrap();
    }
    admin::promote(&mut conn, "a", "test", None).unwrap();
    admin::canary(&mut conn, "b", 5, "test", None).unwrap();
    assert_eq!(
        (
            state(&conn, "b").state.as_str(),
            state(&conn, "b").canary_pct
        ),
        ("canary", 5)
    );
    // BR-UP09: a new contract never canaries against the active one.
    assert_eq!(
        admin::canary(&mut conn, "c", 5, "test", None),
        Err(AdminError::ContractChange)
    );
    assert_eq!(
        admin::canary(&mut conn, "b", 0, "test", None),
        Err(AdminError::BadPercent)
    );
    // A failed canary stops new admissions and keeps running.
    admin::hold(&mut conn, "b", "test", None).unwrap();
    assert_eq!(state(&conn, "b").state, "ready");
    // The scheduled switch to a new contract drains every admitting slot.
    admin::promote(&mut conn, "c", "test", None).unwrap();
    assert_eq!(state(&conn, "a").state, "draining");
    assert_eq!(state(&conn, "c").state, "active");
    // Rolling new admissions back to the live, draining slot.
    admin::rollback(&mut conn, "a", "test", None).unwrap();
    assert_eq!(
        (
            state(&conn, "a").state.as_str(),
            state(&conn, "c").state.as_str()
        ),
        ("active", "draining")
    );
    // An out-of-date expectation is refused.
    let v = state(&conn, "c").version;
    assert_eq!(
        admin::rollback(&mut conn, "c", "test", Some(v - 1)),
        Err(AdminError::Stale)
    );
}

#[test]
fn retirement_and_ranked_handoff_wait_for_work_in_flight() {
    let mut conn = db();
    register(&mut conn, &ident("a", "ga", "c1"), now_unix()).unwrap();
    register(&mut conn, &ident("b", "gb", "c1"), now_unix()).unwrap();
    for (slot, generation) in [("a", "ga"), ("b", "gb")] {
        mark_ready(&conn, slot, generation).unwrap();
    }
    admin::promote(&mut conn, "a", "test", None).unwrap();
    admin::ranked_authority(&mut conn, "a", false, "test").unwrap();
    admin::promote(&mut conn, "b", "test", None).unwrap();
    let busy = SlotLoad {
        matches_running: 1,
        results_pending: 1,
        ranked_inflight: 1,
    };
    beat(&conn, "a", "ga", busy);
    // B cannot take ranked settlement while A still finalizes.
    assert!(!take_ranked_authority(&mut conn, "b", now_unix()).unwrap());
    assert!(matches!(
        admin::ranked_authority(&mut conn, "b", false, "test"),
        Err(AdminError::Busy(_))
    ));
    assert!(matches!(
        admin::mark_retired(&mut conn, "a", "test", None),
        Err(AdminError::Busy(_))
    ));
    assert!(!retire_self(&conn, "a", "ga").unwrap() || state(&conn, "a").results_pending == 0);
    beat(&conn, "a", "ga", SlotLoad::default());
    assert!(take_ranked_authority(&mut conn, "b", now_unix()).unwrap());
    assert!(!state(&conn, "a").ranked_authority);
    admin::mark_retired(&mut conn, "a", "test", None).unwrap();
    assert_eq!(state(&conn, "a").state, "retired");
    let audit: i64 = conn
        .query_row("SELECT COUNT(*) FROM release_audit", [], |r| r.get(0))
        .unwrap();
    assert!(audit >= 8, "every change is audited ({audit})");
}
