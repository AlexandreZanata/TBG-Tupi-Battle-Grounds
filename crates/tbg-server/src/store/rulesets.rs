//! P61 §3 ruleset registry: each revision's immutable catalog is stored
//! once at startup, outside per-match writes. A known revision with a
//! different hash or catalog is a startup error, never an overwrite.

use rusqlite::{params, Connection, OptionalExtension};

pub(super) fn register(
    conn: &Connection,
    version: u32,
    source_hash: &str,
    catalog_json: &str,
) -> rusqlite::Result<()> {
    let stored: Option<(String, String)> = conn
        .query_row(
            "SELECT source_hash, catalog_json FROM ranked_rulesets WHERE version = ?1",
            params![version],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()?;
    match stored {
        Some((hash, catalog)) if hash == source_hash && catalog == catalog_json => Ok(()),
        Some(_) => {
            tracing::error!(
                "ruleset revision {version} is registered with a different hash or catalog; \
                 append a new RULESET_VERSION instead of reusing it"
            );
            Err(rusqlite::Error::InvalidQuery)
        }
        None => conn
            .execute(
                "INSERT INTO ranked_rulesets (version, source_hash, catalog_json)
                 VALUES (?1, ?2, ?3)",
                params![version, source_hash, catalog_json],
            )
            .map(|_| ()),
    }
}
