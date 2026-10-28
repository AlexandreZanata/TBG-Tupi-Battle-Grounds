//! Ranked results (BR-R03–BR-R06): one transaction reads both ratings,
//! applies the Elo update with each player's own K, bumps the counters,
//! and inserts the `ranked_matches` row. Rating changes happen only here.

use rusqlite::{params, Connection, OptionalExtension};
use serde_json::json;

use tbg_core::protocol::{Profile, RatingResults};
use tbg_core::rating::{apply, delta, Score};
use tbg_core::{Outcome, Seat};

use super::AccountId;

fn score_of(outcome: Option<Outcome>, seat: Seat) -> Score {
    match outcome.and_then(|o| o.winner) {
        None => Score::Draw,
        Some(w) if w == seat => Score::Win,
        Some(_) => Score::Loss,
    }
}

fn factions_json(factions: [u32; 4]) -> String {
    json!({
        "Humans": factions[0],
        "Undead": factions[1],
        "Machines": factions[2],
        "Plants": factions[3],
    })
    .to_string()
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
    /// Purchase shares in percent, Humans/Undead/Machines/Plants order.
    pub factions: [f64; 4],
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

const FACTION_KEYS: [&str; 4] = ["Humans", "Undead", "Machines", "Plants"];

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
                        factions: [0.0; 4],
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
    let mut totals = [0u64; 4];
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
            for (i, key) in FACTION_KEYS.iter().enumerate() {
                totals[i] += parsed[key].as_u64().unwrap_or(0);
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

/// Settle a ranked match. Returns per-seat `(before, after, profile)`.
pub(super) fn ranked_result(
    conn: &mut Connection,
    a: AccountId,
    b: AccountId,
    outcome: Option<Outcome>,
    duration_s: u64,
    factions: [[u32; 4]; 2],
) -> Result<RatingResults, rusqlite::Error> {
    let tx = conn.transaction()?;
    let read = |id: AccountId| -> rusqlite::Result<(u16, u32)> {
        tx.query_row(
            "SELECT rating, ranked_games FROM accounts WHERE id = ?1",
            params![id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
    };
    let ((a_rating, a_games), (b_rating, b_games)) = (read(a)?, read(b)?);
    let a_delta = delta(a_rating, b_rating, a_games, score_of(outcome, Seat::A));
    let b_delta = delta(b_rating, a_rating, b_games, score_of(outcome, Seat::B));
    let (a_after, b_after) = (apply(a_rating, a_delta), apply(b_rating, b_delta));
    let bump = |id: AccountId, after: u16, score: Score| -> rusqlite::Result<Profile> {
        let (win, loss, draw) = match score {
            Score::Win => (1, 0, 0),
            Score::Loss => (0, 1, 0),
            Score::Draw => (0, 0, 1),
        };
        tx.execute(
            "UPDATE accounts SET rating = ?1,
             peak_rating = max(peak_rating, ?1), ranked_games = ranked_games + 1,
             wins = wins + ?2, losses = losses + ?3, draws = draws + ?4 WHERE id = ?5",
            params![after, win, loss, draw, id],
        )?;
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
    };
    let a_profile = bump(a, a_after, score_of(outcome, Seat::A))?;
    let b_profile = bump(b, b_after, score_of(outcome, Seat::B))?;
    let winner: Option<i64> = outcome.and_then(|o| o.winner).map(|s| s.index() as i64);
    let reason = outcome
        .map(|o| format!("{:?}", o.reason))
        .unwrap_or_default();
    tx.execute(
        "INSERT INTO ranked_matches (ended_at, a_id, b_id, winner, reason, duration_s,
         a_before, a_after, b_before, b_after, a_factions, b_factions)
         VALUES (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'), ?1, ?2, ?3, ?4, ?5,
         ?6, ?7, ?8, ?9, ?10, ?11)",
        params![
            a,
            b,
            winner,
            reason,
            duration_s as i64,
            a_rating,
            a_after,
            b_rating,
            b_after,
            factions_json(factions[0]),
            factions_json(factions[1]),
        ],
    )?;
    tx.commit()?;
    Ok([
        (a_rating, a_after, a_profile),
        (b_rating, b_after, b_profile),
    ])
}
