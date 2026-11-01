//! P60 §6: the one bounded history statement per settlement. Two indexed
//! range arms over `ranked_side_history` (one per account), each stopped at
//! `LIMIT + 1` rows, joined to the match outcome and integrity status.

use rusqlite::{named_params, Transaction};

use tbg_core::ranked_integrity::{RatingDisposition, RiskState};

use super::AccountId;
use crate::ranked::signals::HistoryRow;

/// Rows read at most; one more means overflow (fail-open, recorded).
pub const LIMIT: usize = 10_000;

const HISTORY: &str = "SELECT h.match_id, h.seat, h.account_id, h.opponent_id, h.opponent_mature,
       h.effective_delta, h.low_activity, h.early_surrender, m.winner, i.risk, i.disposition
FROM (
  SELECT * FROM (
    SELECT * FROM ranked_sides
     WHERE account_id=:a AND ended_unix>:cutoff AND ended_unix<=:now
       AND match_id<:sequence
     ORDER BY ended_unix, match_id LIMIT 10001
  )
  UNION ALL
  SELECT * FROM (
    SELECT * FROM ranked_sides
     WHERE account_id=:b AND ended_unix>:cutoff AND ended_unix<=:now
       AND match_id<:sequence
     ORDER BY ended_unix, match_id LIMIT 10001
  )
) AS h
JOIN ranked_matches m ON m.id=h.match_id
JOIN ranked_integrity i ON i.match_id=h.match_id
ORDER BY h.ended_unix, h.match_id, h.seat LIMIT 10001";

/// Earlier sides of both accounts in `(now - window, now]` below `sequence`.
/// `None` on overflow or a read error: callers record the gap, never guess.
pub(super) fn load(
    tx: &Transaction,
    accounts: [AccountId; 2],
    now: i64,
    window_secs: i64,
    sequence: i64,
) -> Option<Vec<HistoryRow>> {
    let read = || -> rusqlite::Result<Vec<HistoryRow>> {
        let mut stmt = tx.prepare_cached(HISTORY)?;
        let mut rows = stmt.query(named_params! {
            ":a": accounts[0], ":b": accounts[1],
            ":cutoff": now - window_secs, ":now": now, ":sequence": sequence,
        })?;
        let mut out = Vec::new();
        while let Some(row) = rows.next()? {
            if out.len() == LIMIT {
                return Err(rusqlite::Error::QueryReturnedMoreThanOneRow);
            }
            let seat: i64 = row.get(1)?;
            let winner: Option<i64> = row.get(8)?;
            out.push(HistoryRow {
                match_id: row.get(0)?,
                account: row.get(2)?,
                opponent: row.get(3)?,
                opponent_mature: row.get(4)?,
                effective_delta: row.get(5)?,
                low_activity: row.get(6)?,
                early_surrender: row.get(7)?,
                won: winner.map(|w| w == seat),
                risk: RiskState::from_code(row.get(9)?).unwrap_or(RiskState::Normal),
                disposition: RatingDisposition::from_code(row.get(10)?)
                    .unwrap_or(RatingDisposition::Applied),
            });
        }
        Ok(out)
    };
    match read() {
        Ok(rows) => Some(rows),
        Err(rusqlite::Error::QueryReturnedMoreThanOneRow) => {
            tracing::warn!("ranked history over {LIMIT} rows; budgets not evaluated");
            None
        }
        Err(e) => {
            tracing::error!("ranked history read failed: {e}; budgets not evaluated");
            None
        }
    }
}
