//! Ranked results (BR-R03–BR-R06): one transaction reads both ratings,
//! applies the Elo update with each player's own K, bumps the counters,
//! and inserts the `ranked_matches` row. Rating changes happen only here.

use rusqlite::{params, Connection};
use serde_json::json;

use tbg_core::protocol::Profile;
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

/// Settle a ranked match. Returns per-seat `(before, after, profile)`.
pub(super) fn ranked_result(
    conn: &mut Connection,
    a: AccountId,
    b: AccountId,
    outcome: Option<Outcome>,
    duration_s: u64,
    factions: [[u32; 4]; 2],
) -> Result<[(u16, u16, Profile); 2], rusqlite::Error> {
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
