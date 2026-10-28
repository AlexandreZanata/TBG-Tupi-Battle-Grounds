//! Account rows: register and login inside the single writer thread.
//! `BadKey` is identical for unknown name and wrong secret — same error,
//! same code path, no name oracle beyond `NameTaken` at registration.

use rusqlite::{params, Connection, ErrorCode};

use tbg_core::protocol::{AccountError, Profile};

use super::AccountId;

/// Compare 32-byte hashes with no early return (constant shape, not just
/// constant time — the loop always folds all 32 bytes).
fn hashes_equal(a: &[u8], b: &[u8]) -> bool {
    if a.len() != 32 || b.len() != 32 {
        return false;
    }
    let mut diff = 0u8;
    for i in 0..32 {
        diff |= a[i] ^ b[i];
    }
    diff == 0
}

fn profile_from_row(row: &rusqlite::Row) -> rusqlite::Result<(AccountId, Profile)> {
    let id: AccountId = row.get(0)?;
    let profile = Profile {
        name: row.get(1)?,
        rating: row.get(2)?,
        ranked_games: row.get(3)?,
        wins: row.get(4)?,
        losses: row.get(5)?,
        draws: row.get(6)?,
    };
    Ok((id, profile))
}

const SELECT_COLS: &str = "id, name, rating, ranked_games, wins, losses, draws";

/// Insert a new account; the UNIQUE `name_lower` violation is `NameTaken`.
pub(super) fn register(
    conn: &mut Connection,
    name: &str,
    name_lower: &str,
    key_hash: &[u8; 32],
) -> Result<(AccountId, Profile), AccountError> {
    let tx = conn.transaction().map_err(|_| AccountError::Unavailable)?;
    let taken = tx
        .execute(
            "INSERT INTO accounts (name, name_lower, key_hash, created_at, last_login_at)
             VALUES (?1, ?2, ?3, strftime('%Y-%m-%dT%H:%M:%SZ', 'now'), strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))",
            params![name, name_lower, key_hash.as_slice()],
        )
        .map_err(|e| match e {
            rusqlite::Error::SqliteFailure(err, _) if err.code == ErrorCode::ConstraintViolation => {
                AccountError::NameTaken
            }
            _ => AccountError::Unavailable,
        })?;
    if taken != 1 {
        return Err(AccountError::Unavailable);
    }
    let id = tx.last_insert_rowid();
    let row = tx
        .query_row(
            &format!("SELECT {SELECT_COLS} FROM accounts WHERE id = ?1"),
            params![id],
            profile_from_row,
        )
        .map_err(|_| AccountError::Unavailable)?;
    tx.commit().map_err(|_| AccountError::Unavailable)?;
    Ok(row)
}

/// Look up by `name_lower`; unknown name and wrong secret both read `BadKey`.
pub(super) fn login(
    conn: &Connection,
    name_lower: &str,
    key_hash: &[u8; 32],
) -> Result<(AccountId, Profile), AccountError> {
    let (id, stored, profile) = conn
        .query_row(
            &format!("SELECT {SELECT_COLS}, key_hash FROM accounts WHERE name_lower = ?1"),
            params![name_lower],
            |row| {
                let (id, profile) = profile_from_row(row)?;
                let stored: Vec<u8> = row.get(7)?;
                Ok((id, stored, profile))
            },
        )
        .map_err(|_| AccountError::BadKey)?;
    if !hashes_equal(&stored, key_hash) {
        return Err(AccountError::BadKey);
    }
    conn.execute(
        "UPDATE accounts SET last_login_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now') WHERE id = ?1",
        params![id],
    )
    .map_err(|_| AccountError::Unavailable)?;
    Ok((id, profile))
}
