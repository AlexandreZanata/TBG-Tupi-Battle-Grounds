//! Account Arcana presets (BR-AR08, D-40/P40): one saved complete preset per
//! account, fifteen bytes, validated on read so a bad row is never applied;
//! legacy six-byte rows migrate in place before their first use.

use rusqlite::{params, Connection, OptionalExtension};

use tbg_core::arcana::NODES;
use tbg_core::ArcanaPreset;

use super::AccountId;

pub(super) fn save(
    conn: &Connection,
    account: AccountId,
    preset: ArcanaPreset,
) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE accounts SET arcanas = ?2 WHERE id = ?1",
        params![account, preset.ranks.as_slice()],
    )?;
    Ok(())
}

/// The saved preset, if any and still complete. A schema-1 six-byte row is
/// migrated in place to the twelve-point tree and saved before it is used.
pub(super) fn load(conn: &Connection, account: AccountId) -> Option<ArcanaPreset> {
    let blob: Option<Vec<u8>> = conn
        .query_row(
            "SELECT arcanas FROM accounts WHERE id = ?1",
            params![account],
            |r| r.get(0),
        )
        .optional()
        .ok()
        .flatten()
        .flatten();
    let blob = blob?;
    if blob.len() == 6 {
        let old: [u8; 6] = blob.try_into().ok()?;
        let preset = ArcanaPreset::migrate_legacy(old);
        save(conn, account, preset).ok()?;
        return Some(preset);
    }
    let ranks: [u8; NODES] = blob.try_into().ok()?;
    ArcanaPreset::complete(ranks).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn conn() -> Connection {
        let conn = Connection::open_in_memory().expect("memory db");
        conn.execute_batch(
            "CREATE TABLE accounts(id INTEGER PRIMARY KEY, arcanas BLOB);
             INSERT INTO accounts(id, arcanas) VALUES (1, X'020002000200');",
        )
        .expect("fixture");
        conn
    }

    #[test]
    fn a_six_byte_row_migrates_in_place_to_the_twelve_point_tree() {
        let conn = conn();
        assert_eq!(load(&conn, 1), Some(ArcanaPreset::DEFAULT));
        let blob: Vec<u8> = conn
            .query_row("SELECT arcanas FROM accounts WHERE id = 1", [], |r| {
                r.get(0)
            })
            .expect("row");
        assert_eq!(blob.len(), NODES);
        assert_eq!(
            ArcanaPreset::complete(blob.try_into().expect("fifteen")).ok(),
            Some(ArcanaPreset::DEFAULT)
        );
    }

    #[test]
    fn a_saved_preset_round_trips() {
        let conn = conn();
        let preset =
            ArcanaPreset::complete([2, 2, 2, 2, 0, 0, 1, 1, 1, 0, 0, 0, 0, 0, 0]).expect("valid");
        save(&conn, 1, preset).expect("save");
        assert_eq!(load(&conn, 1), Some(preset));
    }
}
