//! Bounded sliding-window dispatch admission for isolated recovery fixtures.

use super::proof::digest;
use super::Error;
use rusqlite::{params, Transaction};
use std::net::IpAddr;

pub(super) fn admit(
    tx: &Transaction<'_>,
    name: &str,
    email: &str,
    ip: IpAddr,
    now: i64,
) -> Result<(), Error> {
    // Proposed fixture bounds: cap storage as well as per-destination dispatch.
    let start = now.checked_sub(3600).ok_or(Error::Unavailable)?;
    tx.execute(
        "DELETE FROM recovery_dispatch_limits WHERE issued<=?1",
        [start],
    )?;
    let count: usize = tx.query_row("SELECT COUNT(*) FROM recovery_dispatch_limits", [], |r| {
        r.get(0)
    })?;
    if count > 253 {
        return Err(Error::Unavailable);
    }
    for (scope, value, limit) in [
        ("account", name.to_string(), 3),
        ("destination", email.to_string(), 3),
        ("ip", ip.to_string(), 12),
    ] {
        let bucket = digest(format!("{scope}:{value}").as_bytes());
        let count: usize = tx.query_row(
            "SELECT COUNT(*) FROM recovery_dispatch_limits WHERE bucket=?1",
            [bucket.as_slice()],
            |r| r.get(0),
        )?;
        if count >= limit {
            return Err(Error::Unavailable);
        }
        tx.execute(
            "INSERT INTO recovery_dispatch_limits VALUES (?1,?2)",
            params![bucket.as_slice(), now],
        )?;
    }
    Ok(())
}
