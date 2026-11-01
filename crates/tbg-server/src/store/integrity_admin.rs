//! P60 §7 offline adjudication: read-only review queries, dry-run plans and
//! their guarded application. A correction rebuilds only the two affected
//! accounts from `ranked_baseline`, replaying each post-cutover entry's
//! frozen first-settlement delta in match-id order with `rating::apply`;
//! later opponents' deltas never change (no recursive Elo recalculation).
//! Decisions are appended to `ranked_decisions`, never rewritten.

use rusqlite::{params, Connection, OptionalExtension, TransactionBehavior};
use serde_json::{json, Value};

use tbg_core::ranked_integrity::{RatingDisposition, RiskState};
use tbg_core::rating::apply;

use super::AccountId;
use crate::ranked::hex;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Invalidate,
    Clear,
    Review,
    Restore,
}

impl Action {
    pub fn parse(s: &str) -> Option<Action> {
        Some(match s {
            "invalidate" => Action::Invalidate,
            "clear" => Action::Clear,
            "review" => Action::Review,
            "restore" => Action::Restore,
            _ => return None,
        })
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Action::Invalidate => "invalidate",
            Action::Clear => "clear",
            Action::Review => "review",
            Action::Restore => "restore",
        }
    }

    /// The only allowed transitions (P60 §4 state machine).
    fn next(self, from: RiskState) -> Option<RiskState> {
        use RiskState::{Invalidated, Normal, Review};
        match (self, from) {
            (Action::Invalidate, Normal | Review) => Some(Invalidated),
            (Action::Clear, Review) => Some(Normal),
            (Action::Review, Normal) => Some(Review),
            (Action::Restore, Invalidated) => Some(Normal),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub enum AdminError {
    Sql(rusqlite::Error),
    UnknownMatch,
    /// Pre-v7 rows are never adjudicated here.
    LegacyMatch,
    BadTransition(RiskState, Action),
    BadInput(&'static str),
    /// The database changed since the plan (revision lease or records).
    StalePlan,
    BadPlan,
}

impl From<rusqlite::Error> for AdminError {
    fn from(e: rusqlite::Error) -> AdminError {
        AdminError::Sql(e)
    }
}

/// An account's competitive projection.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Projection {
    pub id: AccountId,
    pub rating: u16,
    pub peak: u16,
    pub games: u32,
    pub wins: u32,
    pub losses: u32,
    pub draws: u32,
    pub mature: u32,
}

impl Projection {
    fn to_json(&self) -> Value {
        json!({
            "id": self.id, "rating": self.rating, "peak": self.peak, "games": self.games,
            "wins": self.wins, "losses": self.losses, "draws": self.draws, "mature": self.mature,
        })
    }

    fn from_json(v: &Value) -> Option<Projection> {
        let n = |k: &str| v[k].as_u64().and_then(|x| u32::try_from(x).ok());
        Some(Projection {
            id: v["id"].as_i64()?,
            rating: u16::try_from(v["rating"].as_u64()?).ok()?,
            peak: u16::try_from(v["peak"].as_u64()?).ok()?,
            games: n("games")?,
            wins: n("wins")?,
            losses: n("losses")?,
            draws: n("draws")?,
            mature: n("mature")?,
        })
    }
}

/// A dry-run correction: everything `apply` will check and write.
#[derive(Clone, Debug, PartialEq)]
pub struct Plan {
    pub operation_key: String,
    pub action: Action,
    pub match_id: i64,
    pub match_key: String,
    pub reason: String,
    pub actor: String,
    pub decided_unix: i64,
    pub base_revision: i64,
    pub risk_before: RiskState,
    pub risk_after: RiskState,
    pub before: [Projection; 2],
    pub after: [Projection; 2],
}

/// One match waiting for human review.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReviewRow {
    pub match_id: i64,
    pub key: String,
    pub ended_unix: i64,
    pub names: [String; 2],
}

/// Every match currently in Review, oldest first.
pub fn list_review(conn: &Connection) -> rusqlite::Result<Vec<ReviewRow>> {
    let mut stmt = conn.prepare(
        "SELECT m.id, m.match_key, m.ended_unix, a.name, b.name
         FROM ranked_integrity i JOIN ranked_matches m ON m.id = i.match_id
         JOIN accounts a ON a.id = m.a_id JOIN accounts b ON b.id = m.b_id
         WHERE i.risk = 1 ORDER BY m.id",
    )?;
    let rows = stmt.query_map([], |r| {
        Ok(ReviewRow {
            match_id: r.get(0)?,
            key: hex(&r.get::<_, Vec<u8>>(1)?),
            ended_unix: r.get(2)?,
            names: [r.get(3)?, r.get(4)?],
        })
    })?;
    rows.collect()
}

/// The full record of one match for an operator: row, facts, sides, log.
pub fn inspect(conn: &Connection, key: &[u8; 16]) -> Result<Value, AdminError> {
    let (id, mut record): (i64, Value) = conn
        .query_row(
            "SELECT m.id, m.a_id, m.b_id, m.winner, m.end_detail, i.facts_json, i.risk,
                    i.disposition
             FROM ranked_matches m JOIN ranked_integrity i ON i.match_id = m.id
             WHERE m.match_key = ?1",
            params![key.as_slice()],
            |r| {
                let facts: String = r.get(5)?;
                Ok((
                    r.get(0)?,
                    json!({
                        "match_id": r.get::<_, i64>(0)?,
                        "accounts": [r.get::<_, AccountId>(1)?, r.get::<_, AccountId>(2)?],
                        "winner": r.get::<_, Option<i64>>(3)?,
                        "end_detail": r.get::<_, Option<String>>(4)?,
                        "facts": serde_json::from_str::<Value>(&facts).unwrap_or(Value::Null),
                        "risk": r.get::<_, u8>(6)?,
                        "disposition": r.get::<_, u8>(7)?,
                    }),
                ))
            },
        )
        .optional()?
        .ok_or(AdminError::UnknownMatch)?;
    let mut sides = Vec::new();
    let mut stmt = conn.prepare(
        "SELECT seat, original_delta, effective_delta, opponent_mature, low_activity, early_surrender
         FROM ranked_sides WHERE match_id = ?1 ORDER BY seat",
    )?;
    let mut rows = stmt.query(params![id])?;
    while let Some(r) = rows.next()? {
        sides.push(json!({
            "seat": r.get::<_, i64>(0)?, "original_delta": r.get::<_, i64>(1)?,
            "effective_delta": r.get::<_, i64>(2)?, "opponent_mature": r.get::<_, bool>(3)?,
            "low_activity": r.get::<_, bool>(4)?, "early_surrender": r.get::<_, bool>(5)?,
        }));
    }
    let mut decisions = Vec::new();
    let mut stmt = conn.prepare(
        "SELECT decided_unix, actor, action, reason FROM ranked_decisions
         WHERE match_id = ?1 ORDER BY id",
    )?;
    let mut rows = stmt.query(params![id])?;
    while let Some(r) = rows.next()? {
        decisions.push(json!({
            "decided_unix": r.get::<_, i64>(0)?, "actor": r.get::<_, String>(1)?,
            "action": r.get::<_, String>(2)?, "reason": r.get::<_, String>(3)?,
        }));
    }
    record["sides"] = Value::Array(sides);
    record["decisions"] = Value::Array(decisions);
    Ok(record)
}

fn current(conn: &Connection, id: AccountId) -> rusqlite::Result<Projection> {
    conn.query_row(
        "SELECT rating, peak_rating, ranked_games, wins, losses, draws, mature_games
         FROM accounts WHERE id = ?1",
        params![id],
        |r| {
            Ok(Projection {
                id,
                rating: r.get(0)?,
                peak: r.get(1)?,
                games: r.get(2)?,
                wins: r.get(3)?,
                losses: r.get(4)?,
                draws: r.get(5)?,
                mature: r.get(6)?,
            })
        },
    )
}

/// Replay `account` from its baseline with `target`'s risk replaced. An
/// entry counts only when Applied and not Invalidated; a counted entry
/// re-applies its frozen first-settlement delta, so floor and ceiling hold.
fn replay(
    conn: &Connection,
    account: AccountId,
    target: (i64, RiskState),
) -> rusqlite::Result<Projection> {
    let mut p = conn.query_row(
        "SELECT rating, peak_rating, ranked_games, wins, losses, draws, mature_games
         FROM ranked_baseline WHERE account_id = ?1",
        params![account],
        |r| {
            Ok(Projection {
                id: account,
                rating: r.get(0)?,
                peak: r.get(1)?,
                games: r.get(2)?,
                wins: r.get(3)?,
                losses: r.get(4)?,
                draws: r.get(5)?,
                mature: r.get(6)?,
            })
        },
    )?;
    let mut stmt = conn.prepare(
        "SELECT s.match_id, s.seat, s.original_delta, i.risk, i.disposition, m.winner
         FROM ranked_sides s JOIN ranked_integrity i ON i.match_id = s.match_id
         JOIN ranked_matches m ON m.id = s.match_id
         WHERE s.account_id = ?1 ORDER BY s.match_id",
    )?;
    let mut rows = stmt.query(params![account])?;
    while let Some(r) = rows.next()? {
        let match_id: i64 = r.get(0)?;
        let seat: i64 = r.get(1)?;
        let delta: i32 = r.get(2)?;
        let risk = if match_id == target.0 {
            target.1
        } else {
            RiskState::from_code(r.get(3)?).unwrap_or(RiskState::Normal)
        };
        let applied = RatingDisposition::from_code(r.get(4)?) == Some(RatingDisposition::Applied);
        if !applied || risk == RiskState::Invalidated {
            continue;
        }
        let winner: Option<i64> = r.get(5)?;
        p.rating = apply(p.rating, delta);
        p.peak = p.peak.max(p.rating);
        p.games += 1;
        p.mature += 1;
        match winner {
            None => p.draws += 1,
            Some(w) if w == seat => p.wins += 1,
            Some(_) => p.losses += 1,
        }
    }
    Ok(p)
}

/// Build a dry-run plan on any copy (read-only use is enough).
pub fn plan(
    conn: &Connection,
    key: &[u8; 16],
    action: Action,
    reason: &str,
    actor: &str,
    operation_key: String,
    decided_unix: i64,
) -> Result<Plan, AdminError> {
    if reason.is_empty() || reason.len() > 512 {
        return Err(AdminError::BadInput("reason must be 1..=512 bytes"));
    }
    if actor.is_empty() || actor.len() > 64 || actor.chars().any(char::is_control) {
        return Err(AdminError::BadInput(
            "actor must be 1..=64 printable characters",
        ));
    }
    let (match_id, accounts, risk): (i64, [AccountId; 2], u8) = conn
        .query_row(
            "SELECT m.id, m.a_id, m.b_id, i.risk FROM ranked_matches m
             LEFT JOIN ranked_integrity i ON i.match_id = m.id WHERE m.match_key = ?1",
            params![key.as_slice()],
            |r| {
                Ok((
                    r.get(0)?,
                    [r.get(1)?, r.get(2)?],
                    r.get::<_, Option<u8>>(3)?.unwrap_or(255),
                ))
            },
        )
        .optional()?
        .ok_or(AdminError::UnknownMatch)?;
    let from = RiskState::from_code(risk).ok_or(AdminError::LegacyMatch)?;
    let to = action
        .next(from)
        .ok_or(AdminError::BadTransition(from, action))?;
    let base_revision: i64 = conn.query_row(
        "SELECT revision FROM ranked_meta WHERE singleton = 1",
        [],
        |r| r.get(0),
    )?;
    Ok(Plan {
        operation_key,
        action,
        match_id,
        match_key: hex(key),
        reason: reason.to_string(),
        actor: actor.to_string(),
        decided_unix,
        base_revision,
        risk_before: from,
        risk_after: to,
        before: [current(conn, accounts[0])?, current(conn, accounts[1])?],
        after: [
            replay(conn, accounts[0], (match_id, to))?,
            replay(conn, accounts[1], (match_id, to))?,
        ],
    })
}

/// What `apply` did.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Applied {
    Committed,
    /// This operation key is already in the log: nothing changed.
    AlreadyApplied,
}

/// Apply a plan on the live file of a **stopped** server: same base
/// revision, same current records and the same recomputed projection, all
/// checked inside one immediate transaction that also appends the decision.
pub fn apply_plan(conn: &mut Connection, plan: &Plan) -> Result<Applied, AdminError> {
    let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let done: Option<i64> = tx
        .query_row(
            "SELECT id FROM ranked_decisions WHERE operation_key = ?1",
            params![plan.operation_key],
            |r| r.get(0),
        )
        .optional()?;
    if done.is_some() {
        return Ok(Applied::AlreadyApplied);
    }
    let key = crate::ranked::parse_key(&plan.match_key).ok_or(AdminError::BadPlan)?;
    let fresh = self::plan(
        &tx,
        &key,
        plan.action,
        &plan.reason,
        &plan.actor,
        plan.operation_key.clone(),
        plan.decided_unix,
    )?;
    if fresh != *plan {
        return Err(AdminError::StalePlan);
    }
    for p in &plan.after {
        tx.execute(
            "UPDATE accounts SET rating = ?1, peak_rating = ?2, ranked_games = ?3, wins = ?4,
             losses = ?5, draws = ?6, mature_games = ?7 WHERE id = ?8",
            params![p.rating, p.peak, p.games, p.wins, p.losses, p.draws, p.mature, p.id],
        )?;
    }
    tx.execute(
        "UPDATE ranked_integrity SET risk = ?1 WHERE match_id = ?2",
        params![plan.risk_after.code(), plan.match_id],
    )?;
    // Projection of the side rows: zero while invalidated, the frozen
    // first-settlement delta otherwise (NoChange entries stay zero).
    tx.execute(
        "UPDATE ranked_sides SET effective_delta =
           CASE WHEN ?1 = 2 OR (SELECT disposition FROM ranked_integrity WHERE match_id = ?2) = 1
           THEN 0 ELSE original_delta END
         WHERE match_id = ?2",
        params![plan.risk_after.code(), plan.match_id],
    )?;
    let side = |ps: &[Projection; 2], risk: RiskState| {
        json!({"risk": risk.code(), "accounts": ps.each_ref().map(Projection::to_json)}).to_string()
    };
    tx.execute(
        "INSERT INTO ranked_decisions (operation_key, match_id, decided_unix, actor, action,
         reason, before_json, after_json) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            plan.operation_key,
            plan.match_id,
            plan.decided_unix,
            plan.actor,
            plan.action.as_str(),
            plan.reason,
            side(&plan.before, plan.risk_before),
            side(&plan.after, plan.risk_after),
        ],
    )?;
    tx.execute(
        "UPDATE ranked_meta SET revision = revision + 1 WHERE singleton = 1",
        [],
    )?;
    tx.commit()?;
    Ok(Applied::Committed)
}

impl Plan {
    pub fn to_json(&self) -> Value {
        json!({
            "format": 1,
            "operation_key": self.operation_key, "action": self.action.as_str(),
            "match_id": self.match_id, "match_key": self.match_key,
            "reason": self.reason, "actor": self.actor, "decided_unix": self.decided_unix,
            "base_revision": self.base_revision,
            "risk_before": self.risk_before.code(), "risk_after": self.risk_after.code(),
            "before": self.before.each_ref().map(Projection::to_json),
            "after": self.after.each_ref().map(Projection::to_json),
        })
    }

    pub fn from_json(v: &Value) -> Option<Plan> {
        if v["format"].as_u64()? != 1 {
            return None;
        }
        let risk = |k: &str| RiskState::from_code(u8::try_from(v[k].as_u64()?).ok()?);
        let pair = |k: &str| -> Option<[Projection; 2]> {
            Some([
                Projection::from_json(v[k].get(0)?)?,
                Projection::from_json(v[k].get(1)?)?,
            ])
        };
        Some(Plan {
            operation_key: v["operation_key"].as_str()?.to_string(),
            action: Action::parse(v["action"].as_str()?)?,
            match_id: v["match_id"].as_i64()?,
            match_key: v["match_key"].as_str()?.to_string(),
            reason: v["reason"].as_str()?.to_string(),
            actor: v["actor"].as_str()?.to_string(),
            decided_unix: v["decided_unix"].as_i64()?,
            base_revision: v["base_revision"].as_i64()?,
            risk_before: risk("risk_before")?,
            risk_after: risk("risk_after")?,
            before: pair("before")?,
            after: pair("after")?,
        })
    }
}
