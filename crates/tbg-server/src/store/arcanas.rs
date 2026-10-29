//! Account Arcana presets (BR-AR08, D-40): one saved complete preset per
//! account, six bytes, validated on read so a bad row is never applied.

use rusqlite::{params, Connection, OptionalExtension};

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

/// The saved preset, if any and still complete.
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
    let ranks: [u8; 6] = blob?.try_into().ok()?;
    ArcanaPreset::complete(ranks).ok()
}
