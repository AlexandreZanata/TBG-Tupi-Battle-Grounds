//! Operator changes to the release directory (P66 §6.2–§6.4): canary,
//! promotion, the scheduled contract switch, draining, rollback of new
//! admissions and retirement. Each change is one IMMEDIATE transaction
//! that validates the current rows, bumps their versions and appends an
//! audit line; nothing here stops a process or touches a running match.

use rusqlite::{params, Connection, TransactionBehavior};

use super::release::{now_unix, read_directory, SlotRow};

/// Why an operator change was refused.
#[derive(Debug, PartialEq, Eq)]
pub enum AdminError {
    UnknownSlot,
    /// The slot's process has not reported recently.
    NotLive,
    WrongState(String),
    /// A percentage canary across contracts would split the public queue
    /// (BR-UP09): a new contract switches with `promote` instead.
    ContractChange,
    BadPercent,
    /// Work is still in flight; retirement or ranked handoff must wait.
    Busy(String),
    /// The caller's `expect` version is out of date.
    Stale,
    Db(String),
}

impl From<rusqlite::Error> for AdminError {
    fn from(e: rusqlite::Error) -> Self {
        AdminError::Db(e.to_string())
    }
}

type Tx<'a> = rusqlite::Transaction<'a>;

fn find<'a>(rows: &'a [SlotRow], slot: &str) -> Result<&'a SlotRow, AdminError> {
    rows.iter()
        .find(|r| r.slot == slot)
        .ok_or(AdminError::UnknownSlot)
}

fn set_state(tx: &Tx<'_>, slot: &str, state: &str, pct: u8) -> rusqlite::Result<()> {
    tx.execute(
        "UPDATE release_slots SET state = ?2, canary_pct = ?3, version = version + 1 WHERE slot = ?1",
        params![slot, state, pct],
    )?;
    Ok(())
}

fn audit(tx: &Tx<'_>, actor: &str, action: &str, slot: &str, detail: &str) -> rusqlite::Result<()> {
    tx.execute(
        "INSERT INTO release_audit (at_unix, actor, action, slot, detail) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![now_unix(), actor, action, slot, detail],
    )?;
    Ok(())
}

/// Open the transaction and check the optional `expect` version.
fn begin<'a>(
    conn: &'a mut Connection,
    slot: &str,
    expect: Option<i64>,
) -> Result<(Tx<'a>, Vec<SlotRow>), AdminError> {
    let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let rows = read_directory(&tx)?;
    let target = find(&rows, slot)?;
    if expect.is_some_and(|v| v != target.version) {
        return Err(AdminError::Stale);
    }
    Ok((tx, rows))
}

fn live(row: &SlotRow, now: i64) -> Result<(), AdminError> {
    if row.fresh(now) {
        Ok(())
    } else {
        Err(AdminError::NotLive)
    }
}

/// Same-contract canary: `pct` percent of new eligible admissions.
pub fn canary(
    conn: &mut Connection,
    slot: &str,
    pct: u8,
    actor: &str,
    expect: Option<i64>,
) -> Result<(), AdminError> {
    if !(1..=99).contains(&pct) {
        return Err(AdminError::BadPercent);
    }
    let now = now_unix();
    let (tx, rows) = begin(conn, slot, expect)?;
    let target = find(&rows, slot)?;
    live(target, now)?;
    if !matches!(target.state.as_str(), "ready" | "canary") {
        return Err(AdminError::WrongState(target.state.clone()));
    }
    let active = rows.iter().find(|r| r.state == "active" && r.slot != slot);
    if active.is_some_and(|a| a.contract != target.contract) {
        return Err(AdminError::ContractChange);
    }
    set_state(&tx, slot, "canary", pct)?;
    audit(
        &tx,
        actor,
        "canary",
        slot,
        &format!("{pct}% of new admissions"),
    )?;
    tx.commit()?;
    Ok(())
}

/// Make `slot` the one active generation; every other admitting slot
/// drains. Used for the full promotion after a canary and for the
/// scheduled contract switch (BR-UP09). Running matches never move.
pub fn promote(
    conn: &mut Connection,
    slot: &str,
    actor: &str,
    expect: Option<i64>,
) -> Result<(), AdminError> {
    switch_to(conn, slot, actor, expect, &["ready", "canary"], "promote")
}

/// Roll new admissions back to a live, still-running slot (P66 §6.2): the
/// slot that was active drains; matches on both keep running to the end.
pub fn rollback(
    conn: &mut Connection,
    slot: &str,
    actor: &str,
    expect: Option<i64>,
) -> Result<(), AdminError> {
    switch_to(
        conn,
        slot,
        actor,
        expect,
        &["draining", "ready", "canary"],
        "rollback",
    )
}

fn switch_to(
    conn: &mut Connection,
    slot: &str,
    actor: &str,
    expect: Option<i64>,
    allowed: &[&str],
    action: &str,
) -> Result<(), AdminError> {
    let now = now_unix();
    let (tx, rows) = begin(conn, slot, expect)?;
    let target = find(&rows, slot)?;
    live(target, now)?;
    if !allowed.contains(&target.state.as_str()) {
        return Err(AdminError::WrongState(target.state.clone()));
    }
    for other in rows.iter().filter(|r| r.slot != slot && r.admits()) {
        set_state(&tx, &other.slot, "draining", 0)?;
        audit(
            &tx,
            actor,
            "drain",
            &other.slot,
            &format!("{action} of {slot}"),
        )?;
    }
    set_state(&tx, slot, "active", 0)?;
    audit(
        &tx,
        actor,
        action,
        slot,
        &format!("contract {}", target.contract),
    )?;
    tx.commit()?;
    Ok(())
}

/// Stop new admissions on `slot` (the safe-drain fallback when it is the
/// only admitting slot: new play pauses, running matches finish).
pub fn drain(
    conn: &mut Connection,
    slot: &str,
    actor: &str,
    expect: Option<i64>,
) -> Result<(), AdminError> {
    let (tx, rows) = begin(conn, slot, expect)?;
    let target = find(&rows, slot)?;
    if matches!(target.state.as_str(), "retired" | "failed" | "draining") {
        return Err(AdminError::WrongState(target.state.clone()));
    }
    set_state(&tx, slot, "draining", 0)?;
    audit(&tx, actor, "drain", slot, "operator")?;
    tx.commit()?;
    Ok(())
}

/// Stop a canary without promoting it: back to `ready`, its running
/// matches continue (canary rollback of new admissions).
pub fn hold(
    conn: &mut Connection,
    slot: &str,
    actor: &str,
    expect: Option<i64>,
) -> Result<(), AdminError> {
    let (tx, rows) = begin(conn, slot, expect)?;
    let target = find(&rows, slot)?;
    if target.state != "canary" {
        return Err(AdminError::WrongState(target.state.clone()));
    }
    set_state(&tx, slot, "ready", 0)?;
    audit(&tx, actor, "hold", slot, "canary stopped")?;
    tx.commit()?;
    Ok(())
}

/// The retirement gate (P66 §6.2 step 5, BR-UP06): drained, nothing
/// running, nothing pending, no ranked authority. `Ok` lists nothing.
pub fn retire_blockers(row: &SlotRow) -> Vec<String> {
    let mut why = Vec::new();
    if !matches!(row.state.as_str(), "draining" | "ready" | "retired") {
        why.push(format!("state is {}", row.state));
    }
    if row.matches_running > 0 {
        why.push(format!("{} matches running", row.matches_running));
    }
    if row.results_pending > 0 {
        why.push(format!("{} results pending", row.results_pending));
    }
    if row.ranked_inflight > 0 {
        why.push(format!("{} ranked results in flight", row.ranked_inflight));
    }
    if row.ranked_authority {
        why.push("holds ranked authority".into());
    }
    why
}

/// Record that a stopped slot was retired after its gate passed.
pub fn mark_retired(
    conn: &mut Connection,
    slot: &str,
    actor: &str,
    expect: Option<i64>,
) -> Result<(), AdminError> {
    let (tx, rows) = begin(conn, slot, expect)?;
    let target = find(&rows, slot)?;
    let why = retire_blockers(target);
    if !why.is_empty() {
        return Err(AdminError::Busy(why.join(", ")));
    }
    set_state(&tx, slot, "retired", 0)?;
    tx.execute("DELETE FROM account_owner WHERE slot = ?1", params![slot])?;
    audit(&tx, actor, "retire", slot, &target.generation)?;
    tx.commit()?;
    Ok(())
}

/// Hand ranked settlement to `slot` (P66 §6.2): refused while the holder
/// still has ranked results in flight, unless it is gone and `force`.
pub fn ranked_authority(
    conn: &mut Connection,
    slot: &str,
    force: bool,
    actor: &str,
) -> Result<(), AdminError> {
    let now = now_unix();
    let (tx, rows) = begin(conn, slot, None)?;
    let target = find(&rows, slot)?;
    live(target, now)?;
    if let Some(holder) = rows.iter().find(|r| r.ranked_authority && r.slot != slot) {
        let gone = !holder.fresh(now);
        if holder.ranked_inflight > 0 && !(gone && force) {
            return Err(AdminError::Busy(format!(
                "{} has {} ranked results in flight",
                holder.slot, holder.ranked_inflight
            )));
        }
    }
    tx.execute("UPDATE release_slots SET ranked_authority = 0, version = version + 1 WHERE ranked_authority = 1", [])?;
    tx.execute(
        "UPDATE release_slots SET ranked_authority = 1, version = version + 1 WHERE slot = ?1",
        params![slot],
    )?;
    audit(
        &tx,
        actor,
        "ranked-authority",
        slot,
        if force { "forced" } else { "handoff" },
    )?;
    tx.commit()?;
    Ok(())
}
