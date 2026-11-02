//! Bounded moderation feed replay and startup projection loaded on the store writer.
use super::*;
use std::collections::{BTreeMap, BTreeSet};
pub fn read(db: &Connection, after: i64) -> rusqlite::Result<Vec<(i64, String, Option<i64>)>> {
    let mut s = db.prepare(
        "SELECT id,kind,account_id FROM moderation_events WHERE id>?1 ORDER BY id LIMIT 256",
    )?;
    s.query_map([after], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))?
        .collect()
}
pub(crate) fn snapshot(db: &Connection) -> rusqlite::Result<crate::moderation::cache::Snapshot> {
    let mut snapshot = crate::moderation::cache::Snapshot::default();
    let mut s=db.prepare("SELECT m.account_id,a.name_lower FROM account_moderation m JOIN accounts a ON a.id=m.account_id ORDER BY m.account_id LIMIT 65537")?;
    for row in s.query_map([], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?)))? {
        let (id, name) = row?;
        snapshot.states.insert(
            id,
            state(db, id).map_err(|_| rusqlite::Error::InvalidQuery)?,
        );
        snapshot.names.insert(name, id);
    }
    if snapshot.states.len() > 65536 {
        return Err(rusqlite::Error::InvalidQuery);
    }
    snapshot.settings = {
        let mut s = db.prepare("SELECT key,value FROM admin_settings")?;
        s.query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?
            .collect::<Result<BTreeMap<_, _>, _>>()?
    };
    snapshot.terms = {
        let mut s = db.prepare("SELECT term FROM chat_blocked_terms ORDER BY term LIMIT 4096")?;
        s.query_map([], |r| r.get(0))?
            .collect::<Result<BTreeSet<_>, _>>()?
    };
    snapshot.cursor = db.query_row(
        "SELECT COALESCE(MAX(id),0) FROM moderation_events",
        [],
        |r| r.get(0),
    )?;
    Ok(snapshot)
}
pub fn prune(db: &Connection, now: i64) -> rusqlite::Result<()> {
    db.execute("DELETE FROM moderation_events WHERE id IN (SELECT id FROM moderation_events WHERE at_unix<?1 LIMIT 256)",[now-86400])?;
    Ok(())
}
