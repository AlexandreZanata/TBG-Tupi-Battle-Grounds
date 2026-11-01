//! P66 release directory and account ownership (D-92, P66 §6.2–§6.4):
//! which generation each fixed slot runs, which slots may admit new
//! matches, which one settles ranked results, and which slot holds each
//! signed-in account. Every server process on the host shares these rows
//! through the same SQLite file; admin changes run in IMMEDIATE
//! transactions so two promoters can never assign one slot twice.

use rusqlite::{params, Connection, OptionalExtension, TransactionBehavior};

use super::AccountId;

/// A heartbeat older than this marks a slot process as gone.
pub const STALE_SECS: i64 = 15;

/// Lifecycle of one slot (P66 contracts §2); `failed` is never ready.
pub const STATES: [&str; 7] = [
    "warming", "ready", "canary", "active", "draining", "retired", "failed",
];

/// What a server process says about itself when it registers.
#[derive(Clone, Debug)]
pub struct SlotIdentity {
    pub slot: String,
    pub generation: String,
    pub server_build: String,
    pub contract: String,
    pub endpoint: String,
}

/// One directory row as read back.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SlotRow {
    pub slot: String,
    pub generation: String,
    pub server_build: String,
    pub contract: String,
    pub endpoint: String,
    pub state: String,
    pub canary_pct: u8,
    pub ranked_authority: bool,
    pub matches_running: i64,
    pub results_pending: i64,
    pub ranked_inflight: i64,
    pub heartbeat_unix: i64,
    pub version: i64,
}

impl SlotRow {
    pub fn fresh(&self, now: i64) -> bool {
        now - self.heartbeat_unix <= STALE_SECS
    }

    /// May this slot start new matches?
    pub fn admits(&self) -> bool {
        matches!(self.state.as_str(), "active" | "canary")
    }
}

/// Live load a heartbeat reports.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SlotLoad {
    pub matches_running: i64,
    pub results_pending: i64,
    pub ranked_inflight: i64,
}

pub fn now_unix() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs() as i64)
}

const COLS: &str = "slot, generation, server_build, contract, endpoint, state, canary_pct, \
     ranked_authority, matches_running, results_pending, ranked_inflight, heartbeat_unix, version";

fn row(r: &rusqlite::Row<'_>) -> rusqlite::Result<SlotRow> {
    Ok(SlotRow {
        slot: r.get(0)?,
        generation: r.get(1)?,
        server_build: r.get(2)?,
        contract: r.get(3)?,
        endpoint: r.get(4)?,
        state: r.get(5)?,
        canary_pct: r.get::<_, i64>(6)?.clamp(0, 100) as u8,
        ranked_authority: r.get::<_, i64>(7)? != 0,
        matches_running: r.get(8)?,
        results_pending: r.get(9)?,
        ranked_inflight: r.get(10)?,
        heartbeat_unix: r.get(11)?,
        version: r.get(12)?,
    })
}

pub fn read_directory(conn: &Connection) -> rusqlite::Result<Vec<SlotRow>> {
    let mut stmt = conn.prepare(&format!("SELECT {COLS} FROM release_slots ORDER BY slot"))?;
    let rows = stmt.query_map([], row)?.collect();
    rows
}

fn audit(
    conn: &Connection,
    actor: &str,
    action: &str,
    slot: &str,
    detail: &str,
) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO release_audit (at_unix, actor, action, slot, detail) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![now_unix(), actor, action, slot, detail],
    )?;
    Ok(())
}

/// A process takes its slot as `warming`. A restart of the same generation
/// returns to the state it had (a crash must not silently retire a slot);
/// a different generation always starts `warming` with no ranked authority.
pub fn register(conn: &mut Connection, id: &SlotIdentity, now: i64) -> rusqlite::Result<SlotRow> {
    let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let previous: Option<SlotRow> = tx
        .query_row(
            &format!("SELECT {COLS} FROM release_slots WHERE slot = ?1"),
            params![id.slot],
            row,
        )
        .optional()?;
    let same = previous
        .as_ref()
        .is_some_and(|p| p.generation == id.generation);
    let (state, pct, authority) = match &previous {
        Some(p) if same && p.state != "retired" && p.state != "failed" => (
            p.state.clone(),
            i64::from(p.canary_pct),
            i64::from(p.ranked_authority),
        ),
        _ => ("warming".to_string(), 0, 0),
    };
    tx.execute(
        "INSERT INTO release_slots (slot, generation, server_build, contract, endpoint, state,
           canary_pct, ranked_authority, matches_running, results_pending, ranked_inflight,
           heartbeat_unix, version)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 0, 0, 0, ?9, 1)
         ON CONFLICT(slot) DO UPDATE SET generation = ?2, server_build = ?3, contract = ?4,
           endpoint = ?5, state = ?6, canary_pct = ?7, ranked_authority = ?8,
           matches_running = 0, results_pending = 0, ranked_inflight = 0,
           heartbeat_unix = ?9, version = version + 1",
        params![
            id.slot,
            id.generation,
            id.server_build,
            id.contract,
            id.endpoint,
            state,
            pct,
            authority,
            now
        ],
    )?;
    // A replaced slot's former sessions are gone with its process.
    tx.execute(
        "DELETE FROM account_owner WHERE slot = ?1",
        params![id.slot],
    )?;
    let detail = format!(
        "generation {} build {} contract {}",
        id.generation, id.server_build, id.contract
    );
    audit(&tx, "server", "register", &id.slot, &detail)?;
    let stored = tx.query_row(
        &format!("SELECT {COLS} FROM release_slots WHERE slot = ?1"),
        params![id.slot],
        row,
    )?;
    tx.commit()?;
    Ok(stored)
}

/// `warming` becomes `ready` once the process can serve (store and ranked
/// recovery done). Other states are operator decisions and stay.
pub fn mark_ready(conn: &Connection, slot: &str, generation: &str) -> rusqlite::Result<bool> {
    let n = conn.execute(
        "UPDATE release_slots SET state = 'ready', version = version + 1
         WHERE slot = ?1 AND generation = ?2 AND state = 'warming'",
        params![slot, generation],
    )?;
    if n > 0 {
        audit(conn, "server", "ready", slot, generation)?;
    }
    Ok(n > 0)
}

/// Report load and liveness; `false` when another generation now holds the
/// slot, so this process must stop admitting (fail closed).
pub fn heartbeat(
    conn: &Connection,
    slot: &str,
    generation: &str,
    load: SlotLoad,
    now: i64,
) -> rusqlite::Result<bool> {
    let n = conn.execute(
        "UPDATE release_slots SET heartbeat_unix = ?3, matches_running = ?4,
           results_pending = ?5, ranked_inflight = ?6
         WHERE slot = ?1 AND generation = ?2",
        params![
            slot,
            generation,
            now,
            load.matches_running,
            load.results_pending,
            load.ranked_inflight
        ],
    )?;
    conn.execute(
        "UPDATE account_owner SET heartbeat_unix = ?2 WHERE slot = ?1",
        params![slot, now],
    )?;
    Ok(n > 0)
}

/// The server's own exit: a drained slot with nothing pending is retired.
pub fn retire_self(conn: &Connection, slot: &str, generation: &str) -> rusqlite::Result<bool> {
    let n = conn.execute(
        "UPDATE release_slots SET state = 'retired', ranked_authority = 0,
           canary_pct = 0, version = version + 1
         WHERE slot = ?1 AND generation = ?2 AND matches_running = 0 AND results_pending = 0",
        params![slot, generation],
    )?;
    if n > 0 {
        audit(conn, "server", "retire", slot, generation)?;
    }
    Ok(n > 0)
}

/// Take ranked authority when the holder is gone or idle (P66 §6.2): only
/// one slot ever settles ranked results, so their sequence never forks.
pub fn take_ranked_authority(
    conn: &mut Connection,
    slot: &str,
    now: i64,
) -> rusqlite::Result<bool> {
    let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let rows = read_directory(&tx)?;
    let Some(me) = rows.iter().find(|r| r.slot == slot) else {
        return Ok(false);
    };
    if me.ranked_authority || me.state != "active" || !me.fresh(now) {
        return Ok(me.ranked_authority);
    }
    let holder_busy = rows
        .iter()
        .any(|r| r.slot != slot && r.ranked_authority && r.fresh(now) && r.ranked_inflight > 0);
    if holder_busy {
        return Ok(false);
    }
    tx.execute("UPDATE release_slots SET ranked_authority = 0, version = version + 1 WHERE ranked_authority = 1", [])?;
    tx.execute(
        "UPDATE release_slots SET ranked_authority = 1, version = version + 1 WHERE slot = ?1",
        params![slot],
    )?;
    audit(
        &tx,
        "server",
        "ranked-authority",
        slot,
        "handoff from an idle or absent holder",
    )?;
    tx.commit()?;
    Ok(true)
}

/// Why an account may not sign in on this slot now.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OwnerConflict {
    /// A live session holds the account on another slot.
    OtherSlot,
}

/// Claim `account` for this slot (P66 §6.2): refused while a fresh session
/// on another live slot holds it; a stale or retired holder is replaced
/// with a higher epoch (the fence a stale writer cannot pass).
pub fn claim_owner(
    conn: &mut Connection,
    account: AccountId,
    slot: &str,
    generation: &str,
    now: i64,
) -> rusqlite::Result<Result<i64, OwnerConflict>> {
    let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let holder: Option<(String, i64, i64)> = tx
        .query_row(
            "SELECT o.slot, o.epoch, COALESCE(s.heartbeat_unix, 0) * (s.state != 'retired')
             FROM account_owner o LEFT JOIN release_slots s ON s.slot = o.slot
             WHERE o.account_id = ?1",
            params![account],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .optional()?;
    if let Some((other, _, beat)) = &holder {
        if other != slot && now - beat <= STALE_SECS {
            return Ok(Err(OwnerConflict::OtherSlot));
        }
    }
    let epoch = holder.map_or(1, |(_, e, _)| e + 1);
    tx.execute(
        "INSERT INTO account_owner (account_id, slot, generation, epoch, heartbeat_unix)
         VALUES (?1, ?2, ?3, ?4, ?5)
         ON CONFLICT(account_id) DO UPDATE SET slot = ?2, generation = ?3, epoch = ?4,
           heartbeat_unix = ?5",
        params![account, slot, generation, epoch, now],
    )?;
    tx.commit()?;
    Ok(Ok(epoch))
}

/// Release only this slot's hold; another slot's newer claim is untouched.
pub fn release_owner(conn: &Connection, account: AccountId, slot: &str) -> rusqlite::Result<()> {
    conn.execute(
        "DELETE FROM account_owner WHERE account_id = ?1 AND slot = ?2",
        params![account, slot],
    )?;
    Ok(())
}

/// Does this slot still hold `account` at `epoch`?
pub fn owns(
    conn: &Connection,
    account: AccountId,
    slot: &str,
    epoch: i64,
) -> rusqlite::Result<bool> {
    conn.query_row(
        "SELECT COUNT(*) FROM account_owner WHERE account_id = ?1 AND slot = ?2 AND epoch = ?3",
        params![account, slot, epoch],
        |r| r.get::<_, i64>(0),
    )
    .map(|n| n > 0)
}

#[cfg(test)]
mod tests;
