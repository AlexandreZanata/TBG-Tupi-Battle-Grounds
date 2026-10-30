//! Account Stronghold kind (BR-SK01, D-46): one small integer per account,
//! validated on read so an unknown or lab-only value is never applied.

use rusqlite::{params, Connection, OptionalExtension};

use tbg_core::StrongholdKind;

use super::AccountId;

pub(super) fn save(
    conn: &Connection,
    account: AccountId,
    kind: StrongholdKind,
) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE accounts SET stronghold = ?2 WHERE id = ?1",
        params![account, kind.index() as i64],
    )?;
    Ok(())
}

/// The saved kind, if any and still selectable.
pub(super) fn load(conn: &Connection, account: AccountId) -> Option<StrongholdKind> {
    let value: Option<i64> = conn
        .query_row(
            "SELECT stronghold FROM accounts WHERE id = ?1",
            params![account],
            |r| r.get(0),
        )
        .optional()
        .ok()
        .flatten()
        .flatten();
    let kind = StrongholdKind::from_index(usize::try_from(value?).ok()?)?;
    kind.selectable().then_some(kind)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn saved_kind_round_trips_and_bad_rows_are_ignored() {
        let conn = Connection::open_in_memory().expect("memory db");
        conn.execute_batch(
            "CREATE TABLE accounts(id INTEGER PRIMARY KEY, stronghold INTEGER);
             INSERT INTO accounts(id) VALUES (1), (2), (3);
             UPDATE accounts SET stronghold = 99 WHERE id = 2;
             UPDATE accounts SET stronghold = 0 WHERE id = 3;",
        )
        .expect("fixture");
        assert_eq!(load(&conn, 1), None, "never chosen");
        save(&conn, 1, StrongholdKind::Artillery).expect("save");
        assert_eq!(load(&conn, 1), Some(StrongholdKind::Artillery));
        assert_eq!(load(&conn, 2), None, "unknown value");
        assert_eq!(load(&conn, 3), None, "Classic is never a player's");
    }
}
