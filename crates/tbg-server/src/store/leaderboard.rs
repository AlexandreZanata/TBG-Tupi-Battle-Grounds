//! Leaderboard snapshot (P13, BR-R07): every ranked account ordered by
//! rating DESC, wins DESC, name_lower ASC. Readers (HTTP, WS) clone the
//! `Arc` — no SQL per request. Rank is the 1-based position in that order.

use std::collections::HashMap;

use rusqlite::Connection;

/// One ranked account in the global order.
#[derive(Clone, Debug)]
pub struct LeaderRow {
    pub rank: u32,
    pub name: String,
    pub rating: u16,
    pub games: u32,
    pub wins: u32,
    pub losses: u32,
    pub draws: u32,
}

impl LeaderRow {
    /// Win rate in percent with one decimal (0.0 before any game).
    pub fn win_rate(&self) -> f64 {
        if self.games == 0 {
            0.0
        } else {
            (f64::from(self.wins) * 1000.0 / f64::from(self.games)).round() / 10.0
        }
    }
}

/// The published snapshot: ordered rows plus a name → rank lookup.
#[derive(Clone, Debug, Default)]
pub struct Snapshot {
    pub rows: Vec<LeaderRow>,
    pub total: usize,
    /// Time the underlying data last changed (latest row, ISO-8601 UTC).
    pub updated_at: String,
    by_name: HashMap<String, u32>,
}

impl Snapshot {
    /// 1-based rank, or `None` for guests and accounts with no ranked game.
    pub fn rank_of(&self, name_lower: &str) -> Option<u32> {
        self.by_name.get(name_lower).copied()
    }
}

/// Rebuild from the table index; only accounts with ≥ 1 ranked game.
pub(super) fn build(conn: &Connection) -> rusqlite::Result<Snapshot> {
    let updated_at: String = conn.query_row(
        "SELECT COALESCE(MAX(ended_at), strftime('%Y-%m-%dT%H:%M:%SZ', 'now')) FROM ranked_matches",
        [],
        |r| r.get(0),
    )?;
    let mut stmt = conn.prepare(
        "SELECT name, name_lower, rating, ranked_games, wins, losses, draws FROM accounts
         WHERE ranked_games >= 1 ORDER BY rating DESC, wins DESC, name_lower ASC",
    )?;
    let mut rows = Vec::new();
    let mut by_name = HashMap::new();
    let mut rank = 0u32;
    let mapped = stmt.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, u16>(2)?,
            row.get::<_, u32>(3)?,
            row.get::<_, u32>(4)?,
            row.get::<_, u32>(5)?,
            row.get::<_, u32>(6)?,
        ))
    })?;
    for entry in mapped {
        let (name, name_lower, rating, games, wins, losses, draws) = entry?;
        rank += 1;
        by_name.insert(name_lower, rank);
        rows.push(LeaderRow {
            rank,
            name,
            rating,
            games,
            wins,
            losses,
            draws,
        });
    }
    let total = rows.len();
    Ok(Snapshot {
        rows,
        total,
        updated_at,
        by_name,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Exact order and ranks with rating, wins, and name tiebreaks.
    #[test]
    fn order_and_tiebreaks() {
        let mut conn = Connection::open_in_memory().unwrap();
        crate::store::schema::migrate(&mut conn).unwrap();
        for name in ["ann", "bob", "cid", "dan", "eli", "fay"] {
            conn.execute(
                "INSERT INTO accounts (name, name_lower, key_hash, created_at)
                 VALUES (?1, ?1, zeroblob(32), '2026-01-01T00:00:00Z')",
                [name],
            )
            .unwrap();
        }
        // Sculpt: ann/cid 1020 (tie → name), bob/eli/fay 1000 (wins → bob),
        // dan 960. All deltas are exact halves (no rounding risk).
        let set = |id: i64, rating: u16, games: u32, wins: u32, losses: u32| {
            conn.execute(
                "UPDATE accounts SET rating = ?1, ranked_games = ?2, wins = ?3,
                 losses = ?4 WHERE id = ?5",
                rusqlite::params![rating, games, wins, losses, id],
            )
            .unwrap();
        };
        set(1, 1020, 2, 1, 0); // ann
        set(2, 1000, 2, 1, 1); // bob
        set(3, 1020, 2, 1, 0); // cid
        set(4, 960, 2, 0, 2); // dan
        set(5, 1000, 1, 0, 0); // eli
        set(6, 1000, 1, 0, 0); // fay
        let snap = build(&conn).unwrap();
        let order: Vec<(&str, u32)> = snap
            .rows
            .iter()
            .map(|r| (r.name.as_str(), r.rank))
            .collect();
        assert_eq!(
            order,
            [
                ("ann", 1),
                ("cid", 2),
                ("bob", 3),
                ("eli", 4),
                ("fay", 5),
                ("dan", 6)
            ]
        );
        assert_eq!(snap.total, 6);
        assert_eq!(snap.rank_of("cid"), Some(2));
        assert_eq!(snap.rank_of("nobody"), None);
    }
}
