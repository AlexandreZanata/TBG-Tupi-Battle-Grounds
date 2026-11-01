//! Resumable, bounded import of verified Ranked account links; never infer Casual ownership.

use super::{Envelope, Error, HistorySide};
use rusqlite::{params, Connection};
use tbg_core::protocol::{MatchMode, MatchOrigin, PersonalMatchSummary};

/// Runs on the existing store worker only when the explicit history rollout is enabled.
pub fn batch(conn: &mut Connection) -> Result<bool, Error> {
    conn.execute("UPDATE player_history_meta SET backfill_target=MAX(backfill_target,COALESCE((SELECT MAX(id) FROM ranked_matches),0)) WHERE singleton=1 AND backfill_target<COALESCE((SELECT MAX(id) FROM ranked_matches),0)",[])?;
    let after: i64 = conn.query_row(
        "SELECT backfill_id FROM player_history_meta WHERE singleton=1",
        [],
        |r| r.get(0),
    )?;
    type Row = (
        i64,
        Option<Vec<u8>>,
        i64,
        [i64; 2],
        [String; 2],
        Option<u8>,
        String,
        u64,
        Option<u32>,
        bool,
    );
    let rows: Vec<Row> = {
        let mut stmt = conn.prepare("SELECT m.id,m.match_key,COALESCE(m.ended_unix,unixepoch(m.ended_at)),m.a_id,m.b_id,a.name,b.name,m.winner,m.reason,m.duration_s,m.combat_ticks,m.history_expected FROM ranked_matches m JOIN accounts a ON a.id=m.a_id JOIN accounts b ON b.id=m.b_id WHERE m.id>?1 AND m.id <= (SELECT backfill_target FROM player_history_meta WHERE singleton=1) ORDER BY m.id LIMIT 100")?;
        let collected = stmt
            .query_map([after], |r| {
                Ok((
                    r.get(0)?,
                    r.get(1)?,
                    r.get(2)?,
                    [r.get(3)?, r.get(4)?],
                    [r.get(5)?, r.get(6)?],
                    r.get(7)?,
                    r.get(8)?,
                    u64::try_from(r.get::<_, i64>(9)?)
                        .map_err(|_| rusqlite::Error::InvalidQuery)?,
                    r.get(10)?,
                    r.get(11)?,
                ))
            })?
            .collect::<rusqlite::Result<_>>()?;
        collected
    };
    if rows.is_empty() {
        return Ok(false);
    }
    for (id, key, ended, accounts, names, winner, reason, duration, combat, expected) in rows {
        let ranked_key = key.and_then(|v| <[u8; 16]>::try_from(v).ok());
        let history_key = ranked_key.map_or_else(
            || format!("legacy-ranked:{id}"),
            |k| format!("ranked:{}", crate::ranked::hex(&k)),
        );
        let exists: bool = conn.query_row("SELECT EXISTS(SELECT 1 FROM player_match_history WHERE history_key=?1 OR ranked_row_id=?2)",params![history_key,id],|r| r.get(0))?;
        if !expected && !exists && accounts[0] != accounts[1] {
            let sides = [0, 1].map(|seat| HistorySide {
                entry_id: crate::names::random_bytes::<16>(),
                account: Some(accounts[seat]),
                name: names[seat].clone(),
                summary: PersonalMatchSummary {
                    version: 1,
                    quality: "result_only".into(),
                    mode: MatchMode::Ranked,
                    origin: MatchOrigin::LegacyRanked,
                    ended_unix: ended,
                    arena: None,
                    ruleset_version: None,
                    source_hash: None,
                    combat_ticks: combat,
                    tick_hz: tbg_core::content::TICK_HZ as u16,
                    wall_ms: duration.checked_mul(1000),
                    outcome: if winner.is_none() {
                        "draw"
                    } else if winner == Some(seat as u8) {
                        "win"
                    } else {
                        "loss"
                    }
                    .into(),
                    reason: reason.clone(),
                    purchases: None,
                    champions: None,
                    evolutions: None,
                    fortifications: None,
                    augments: None,
                    stronghold: None,
                },
            });
            let result = super::insert(
                conn,
                &Envelope {
                    format: 1,
                    key: history_key.clone(),
                    result_id: None,
                    ranked_key,
                    ranked_row: Some(id),
                    winner,
                    sides,
                },
            );
            if let Err(error) = result {
                let same_source: bool=conn.query_row("SELECT EXISTS(SELECT 1 FROM player_match_history h JOIN player_match_history_sides a ON a.history_key=h.history_key AND a.seat=0 JOIN player_match_history_sides b ON b.history_key=h.history_key AND b.seat=1 WHERE h.history_key=?1 AND a.account_id=?2 AND b.account_id=?3)",params![history_key,accounts[0],accounts[1]],|r|r.get(0))?;
                if error != Error::Conflict || !same_source {
                    return Err(error);
                }
            }
        }
        // A crash before this advances at most one already-idempotent row again.
        conn.execute(
            "UPDATE player_history_meta SET backfill_id=?1 WHERE singleton=1",
            [id],
        )?;
    }
    Ok(true)
}
