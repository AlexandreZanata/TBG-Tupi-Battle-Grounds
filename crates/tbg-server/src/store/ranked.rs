//! Ranked results (BR-R03–BR-R06, P60): one transaction reads both
//! ratings, applies the unchanged Elo update with each player's own K,
//! bumps the counters and writes the authoritative v7 record. Rating
//! changes happen only here; a repeated match key never applies twice.

use rusqlite::{params, Connection, OptionalExtension};
use serde_json::json;

use tbg_core::protocol::Profile;
use tbg_core::ranked_integrity::{RatingDisposition, RiskState};
use tbg_core::rating::{apply, delta, Score};
use tbg_core::{Outcome, Seat, FACTIONS, FACTION_COUNT};

use super::{history, AccountId};
use crate::ranked::policy::RankedPolicy;
use crate::ranked::signals::{self, Current};
use crate::ranked::{JournalEntry, RankedFinal, RankedReceipt};

fn score_of(outcome: Option<Outcome>, seat: Seat) -> Score {
    match outcome.and_then(|o| o.winner) {
        None => Score::Draw,
        Some(w) if w == seat => Score::Win,
        Some(_) => Score::Loss,
    }
}

fn factions_json(factions: [u32; FACTION_COUNT]) -> String {
    let shares = FACTIONS
        .iter()
        .map(|f| (f.name().to_string(), json!(factions[f.index()])))
        .collect();
    serde_json::Value::Object(shares).to_string()
}

pub(super) fn count(conn: &Connection) -> rusqlite::Result<i64> {
    conn.query_row("SELECT count(*) FROM ranked_matches", [], |r| r.get(0))
}

/// Public profile for `/api/players/{name}` (BR-P05): the account row,
/// faction purchase shares over all ranked matches, and the last 10 ranked
/// matches newest first. At most three indexed queries; `None` for unknown.
#[derive(Clone, Debug)]
pub struct PlayerStats {
    pub name: String,
    pub rating: u16,
    pub peak_rating: u16,
    pub games: u32,
    pub wins: u32,
    pub losses: u32,
    pub draws: u32,
    /// `YYYY-MM-DD` from `created_at`.
    pub joined_day: String,
    /// Purchase shares in percent, in `Faction` index order.
    pub factions: [f64; FACTION_COUNT],
    pub recent: Vec<RecentMatch>,
}

#[derive(Clone, Debug)]
pub struct RecentMatch {
    /// `YYYY-MM-DD` from `ended_at`.
    pub date: String,
    pub opponent: String,
    /// `W`, `L`, or `D` from this player's side.
    pub result: String,
    /// This player's rating delta.
    pub delta: i32,
    pub reason: String,
    pub duration_s: u64,
}

pub(super) fn player_stats(
    conn: &Connection,
    name_lower: &str,
) -> rusqlite::Result<Option<PlayerStats>> {
    let Some((id, mut stats)) = conn
        .query_row(
            "SELECT id, name, rating, peak_rating, ranked_games, wins, losses, draws,
             substr(created_at, 1, 10) FROM accounts WHERE name_lower = ?1",
            rusqlite::params![name_lower],
            |row| {
                Ok((
                    row.get::<_, AccountId>(0)?,
                    PlayerStats {
                        name: row.get(1)?,
                        rating: row.get(2)?,
                        peak_rating: row.get(3)?,
                        games: row.get(4)?,
                        wins: row.get(5)?,
                        losses: row.get(6)?,
                        draws: row.get(7)?,
                        joined_day: row.get(8)?,
                        factions: [0.0; FACTION_COUNT],
                        recent: Vec::new(),
                    },
                ))
            },
        )
        .optional()?
    else {
        return Ok(None);
    };
    // Faction shares over both sides of every ranked match (one query).
    let mut totals = [0u64; FACTION_COUNT];
    let mut stmt = conn.prepare(
        "SELECT a_factions, b_factions, a_id FROM ranked_matches
         WHERE a_id = ?1 OR b_id = ?1",
    )?;
    let mut rows = stmt.query([id])?;
    while let Some(row) = rows.next()? {
        let (a_json, b_json, a_id): (String, String, AccountId) =
            (row.get(0)?, row.get(1)?, row.get(2)?);
        let mine = if a_id == id { &a_json } else { &b_json };
        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(mine) {
            for f in FACTIONS {
                totals[f.index()] += parsed[f.name()].as_u64().unwrap_or(0);
            }
        }
    }
    let all: u64 = totals.iter().sum();
    if all > 0 {
        for (i, share) in stats.factions.iter_mut().enumerate() {
            *share = (totals[i] as f64 * 1000.0 / all as f64).round() / 10.0;
        }
    }
    // Last 10 with the opponent's name joined in (one query).
    let mut recent = conn.prepare(
        "SELECT substr(m.ended_at, 1, 10), o.name,
         CASE WHEN m.winner IS NULL THEN 'D'
              WHEN (m.a_id = ?1 AND m.winner = 0) OR (m.b_id = ?1 AND m.winner = 1)
              THEN 'W' ELSE 'L' END,
         CASE WHEN m.a_id = ?1 THEN m.a_after - m.a_before ELSE m.b_after - m.b_before END,
         m.reason, m.duration_s
         FROM ranked_matches m
         JOIN accounts o ON o.id = CASE WHEN m.a_id = ?1 THEN m.b_id ELSE m.a_id END
         WHERE m.a_id = ?1 OR m.b_id = ?1 ORDER BY m.id DESC LIMIT 10",
    )?;
    let mut rows = recent.query([id])?;
    while let Some(row) = rows.next()? {
        stats.recent.push(RecentMatch {
            date: row.get(0)?,
            opponent: row.get(1)?,
            result: row.get(2)?,
            delta: row.get(3)?,
            reason: row.get(4)?,
            duration_s: row.get::<_, i64>(5)? as u64,
        });
    }
    Ok(Some(stats))
}

/// Why a settlement did not commit. `Sql` is retried by the finalizer;
/// the others are quarantined: retrying cannot change them.
#[derive(Debug)]
pub enum SettleError {
    Sql(rusqlite::Error),
    /// Same match key, different envelope.
    ConflictingDuplicate,
    /// Both seats froze the same account: an internal invariant failure.
    SameAccount,
}

impl From<rusqlite::Error> for SettleError {
    fn from(e: rusqlite::Error) -> SettleError {
        SettleError::Sql(e)
    }
}

/// Integrity policy revision written with every record (P60 §4).
pub const POLICY_VERSION: i64 = 1;
/// Maturity (P60 §2): account age and previously Applied results.
const MATURE_AGE_SECS: i64 = 7 * 24 * 3600;
const MATURE_GAMES: u32 = 20;
/// An early voluntary Surrender (S3): before 15 combat seconds.
const EARLY_SURRENDER_TICKS: u32 = 15 * tbg_core::content::TICK_HZ;

struct Side {
    rating: u16,
    games: u32,
    mature_games: u32,
    created_unix: i64,
}

fn read_side(tx: &rusqlite::Transaction, id: AccountId) -> rusqlite::Result<Side> {
    tx.query_row(
        "SELECT rating, ranked_games, mature_games,
                CAST(strftime('%s', created_at) AS INTEGER) FROM accounts WHERE id = ?1",
        params![id],
        |row| {
            Ok(Side {
                rating: row.get(0)?,
                games: row.get(1)?,
                mature_games: row.get(2)?,
                created_unix: row.get(3)?,
            })
        },
    )
}

fn read_profile(tx: &rusqlite::Transaction, id: AccountId) -> rusqlite::Result<Profile> {
    tx.query_row(
        "SELECT name, rating, ranked_games, wins, losses, draws FROM accounts WHERE id = ?1",
        params![id],
        |row| {
            Ok(Profile {
                name: row.get(0)?,
                rating: row.get(1)?,
                ranked_games: row.get(2)?,
                wins: row.get(3)?,
                losses: row.get(4)?,
                draws: row.get(5)?,
            })
        },
    )
}

/// Highest committed ranked sequence (0 when none): the finalizer's
/// startup high-water mark and the login freshness stamp.
pub(super) fn high_water(conn: &Connection) -> rusqlite::Result<i64> {
    conn.query_row("SELECT COALESCE(MAX(id), 0) FROM ranked_matches", [], |r| {
        r.get(0)
    })
}

/// Settle one journaled ranked result (BR-R03–R06, P60 §6/§7) in one
/// transaction: match, integrity and two side rows, both accounts and the
/// metadata row. A repeated key returns the committed receipt unchanged.
/// Shadow mode: the existing Elo always applies; nothing is gated yet.
pub(super) fn settle(
    conn: &mut Connection,
    entry: &JournalEntry,
    policy: &RankedPolicy,
) -> Result<RankedReceipt, SettleError> {
    let r = &entry.result;
    if r.accounts[0] == r.accounts[1] {
        return Err(SettleError::SameAccount);
    }
    let tx = conn.transaction()?;
    if let Some(receipt) = existing(&tx, r)? {
        return Ok(receipt);
    }
    let (last_utc, revision): (i64, i64) = tx.query_row(
        "SELECT last_utc, revision FROM ranked_meta WHERE singleton = 1",
        [],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )?;
    // Clock rollback never moves the record clock backwards (P60 §2).
    let t = entry.ended_unix.max(last_utc);
    let sides = [
        read_side(&tx, r.accounts[0])?,
        read_side(&tx, r.accounts[1])?,
    ];
    let scores = [
        score_of(Some(r.outcome), Seat::A),
        score_of(Some(r.outcome), Seat::B),
    ];
    let base: [i32; 2] = std::array::from_fn(|i| {
        let (own, opp) = (&sides[i], &sides[1 - i]);
        let d = delta(own.rating, opp.rating, own.games, scores[i]);
        i32::from(apply(own.rating, d)) - i32::from(own.rating)
    });
    let mature = sides
        .each_ref()
        .map(|s| t - s.created_unix >= MATURE_AGE_SECS && s.mature_games >= MATURE_GAMES);
    let early: [bool; 2] = r.seats.each_ref().map(|s| {
        s.surrender_tick
            .is_some_and(|tick| tick < EARLY_SURRENDER_TICKS)
    });
    // Shadow (P60 §4): one bounded history read before any write; budgets
    // and signals are recorded as `would_*`, the baseline Elo applies.
    let history = history::load(&tx, r.accounts, t, policy.window_secs, entry.completion_seq);
    let eval = signals::evaluate(
        history.as_deref(),
        &Current {
            accounts: r.accounts,
            base,
            winner: r.outcome.winner.map(|s| s.index()),
            mature,
            low_activity: r.seats.each_ref().map(|s| s.low_activity()),
            early_surrender: early,
            reached_running: r.reached_running,
        },
        &policy.budget(),
    );
    let after: [u16; 2] = std::array::from_fn(|i| apply(sides[i].rating, base[i]));
    for (i, account) in r.accounts.iter().enumerate() {
        let (win, loss, draw) = match scores[i] {
            Score::Win => (1, 0, 0),
            Score::Loss => (0, 1, 0),
            Score::Draw => (0, 0, 1),
        };
        tx.execute(
            "UPDATE accounts SET rating = ?1, peak_rating = max(peak_rating, ?1),
             ranked_games = ranked_games + 1, mature_games = mature_games + 1,
             wins = wins + ?2, losses = losses + ?3, draws = draws + ?4 WHERE id = ?5",
            params![after[i], win, loss, draw, account],
        )?;
    }
    let winner: Option<i64> = r.outcome.winner.map(|s| s.index() as i64);
    tx.execute(
        "INSERT INTO ranked_matches (id, ended_at, a_id, b_id, winner, reason, duration_s,
         a_before, a_after, b_before, b_after, a_factions, b_factions, a_arcanas, b_arcanas,
         match_key, ruleset_version, ruleset_source_hash, ended_unix, end_detail, combat_ticks)
         VALUES (?1, strftime('%Y-%m-%dT%H:%M:%SZ', ?2, 'unixepoch'), ?3, ?4, ?5, ?6, ?7,
         ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?2, ?19, ?20)",
        params![
            entry.completion_seq,
            t,
            r.accounts[0],
            r.accounts[1],
            winner,
            format!("{:?}", r.outcome.reason),
            (r.wall_ms / 1000) as i64,
            sides[0].rating,
            after[0],
            sides[1].rating,
            after[1],
            factions_json(r.factions[0]),
            factions_json(r.factions[1]),
            r.arcanas[0].ranks.as_slice(),
            r.arcanas[1].ranks.as_slice(),
            r.key.as_slice(),
            r.ruleset_version,
            format!("{:016x}", r.ruleset_source_hash),
            r.end_detail.as_str(),
            r.combat_ticks,
        ],
    )?;
    let facts = json!({
        "v": 1,
        "base": base,
        "mature": mature,
        "reached_running": r.reached_running,
        "seats": r.seats.each_ref().map(|s| json!({
            "purchases": s.purchases, "evolutions": s.evolutions,
            "fortifications": s.fortifications, "augments": s.augments_applied,
            "reconnects": s.reconnects, "surrender_tick": s.surrender_tick,
            "final_hp": s.final_hp, "final_max_hp": s.final_max_hp,
            "damage": null,
        })),
        "history": history.as_ref().map_or_else(|| json!("unavailable"), |h| json!(h.len())),
        "policy": {
            "pair_cap": policy.pair_cap, "new_source_cap": policy.new_source_cap,
            "window_secs": policy.window_secs, "signals": signals::SIGNAL_VERSION,
        },
        "budget": {
            "pair": eval.facts.pair_positive, "immature": eval.facts.immature_positive,
            "opponent_immature": eval.facts.opponent_immature,
        },
        "would": { "bits": eval.would_bits, "applied": eval.would_applied },
        "signals": {
            "s1": eval.signals.s1_relationship, "s2": eval.signals.s2_transfer,
            "s3": eval.signals.s3_behavior, "s4": eval.signals.s4_newcomers,
            "pair_games": eval.signals.pair_games, "donor_games": eval.signals.donor_games,
        },
    })
    .to_string();
    tx.execute(
        "INSERT INTO ranked_integrity (match_id, policy_version, mode, risk, disposition,
         reason_bits, facts_json) VALUES (?1, ?2, 'shadow', ?3, ?4, 0, ?5)",
        params![
            entry.completion_seq,
            POLICY_VERSION,
            eval.risk.code(),
            RatingDisposition::Applied.code(),
            facts,
        ],
    )?;
    for (i, seat) in r.seats.iter().enumerate() {
        tx.execute(
            "INSERT INTO ranked_sides (match_id, seat, account_id, opponent_id, ended_unix,
             opponent_mature, original_delta, effective_delta, low_activity, early_surrender)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?7, ?8, ?9)",
            params![
                entry.completion_seq,
                i as i64,
                r.accounts[i],
                r.accounts[1 - i],
                t,
                mature[1 - i],
                base[i],
                seat.low_activity(),
                early[i],
            ],
        )?;
    }
    tx.execute(
        "UPDATE ranked_meta SET last_utc = ?1, revision = ?2 WHERE singleton = 1",
        params![t, revision + 1],
    )?;
    let profiles = [
        read_profile(&tx, r.accounts[0])?,
        read_profile(&tx, r.accounts[1])?,
    ];
    tx.commit()?;
    let [pa, pb] = profiles;
    Ok(RankedReceipt {
        key: r.key,
        sequence: entry.completion_seq,
        accounts: r.accounts,
        results: [
            (sides[0].rating, after[0], pa),
            (sides[1].rating, after[1], pb),
        ],
        disposition: RatingDisposition::Applied,
        risk: eval.risk,
        reason_bits: 0,
    })
}

/// The committed receipt for this key, or a conflict when the stored
/// envelope differs. `None` when the key is new.
fn existing(
    tx: &rusqlite::Transaction,
    r: &RankedFinal,
) -> Result<Option<RankedReceipt>, SettleError> {
    let row = tx
        .query_row(
            "SELECT m.id, m.a_id, m.b_id, m.winner, m.end_detail, m.combat_ticks,
                    m.a_before, m.a_after, m.b_before, m.b_after,
                    i.risk, i.disposition, i.reason_bits
             FROM ranked_matches m JOIN ranked_integrity i ON i.match_id = m.id
             WHERE m.match_key = ?1",
            params![r.key.as_slice()],
            |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    [row.get::<_, AccountId>(1)?, row.get::<_, AccountId>(2)?],
                    row.get::<_, Option<i64>>(3)?,
                    row.get::<_, Option<String>>(4)?,
                    row.get::<_, Option<u32>>(5)?,
                    [row.get::<_, u16>(6)?, row.get::<_, u16>(7)?],
                    [row.get::<_, u16>(8)?, row.get::<_, u16>(9)?],
                    (
                        row.get::<_, u8>(10)?,
                        row.get::<_, u8>(11)?,
                        row.get::<_, u32>(12)?,
                    ),
                ))
            },
        )
        .optional()?;
    let Some((id, accounts, winner, detail, ticks, a, b, (risk, disp, bits))) = row else {
        return Ok(None);
    };
    let same = accounts == r.accounts
        && winner == r.outcome.winner.map(|s| s.index() as i64)
        && detail.as_deref() == Some(r.end_detail.as_str())
        && ticks == Some(r.combat_ticks);
    if !same {
        return Err(SettleError::ConflictingDuplicate);
    }
    let profiles = [
        read_profile(tx, accounts[0])?,
        read_profile(tx, accounts[1])?,
    ];
    let [pa, pb] = profiles;
    Ok(Some(RankedReceipt {
        key: r.key,
        sequence: id,
        accounts,
        results: [(a[0], a[1], pa), (b[0], b[1], pb)],
        disposition: RatingDisposition::from_code(disp).unwrap_or(RatingDisposition::Applied),
        risk: RiskState::from_code(risk).unwrap_or(RiskState::Normal),
        reason_bits: bits,
    }))
}
