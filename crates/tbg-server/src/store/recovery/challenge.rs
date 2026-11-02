//! Single-use purpose/email/epoch-bound proofs with committed guessing budgets.
use super::Error;
use crate::identity::codes;
use rusqlite::{params, Connection, OptionalExtension};

/// Call inside a transaction and commit false results to retain the guessing budget.
pub fn check(
    db: &Connection,
    account: i64,
    purpose: &str,
    code: &str,
    now: i64,
) -> Result<bool, Error> {
    let valid = proof(db, account, purpose, code, now)?;
    if valid {
        consume(db, account, purpose)?;
    }
    Ok(valid)
}

/// Checking both email-change proofs must not consume the valid one when the other is mistyped.
pub fn proof(
    db: &Connection,
    account: i64,
    purpose: &str,
    code: &str,
    now: i64,
) -> Result<bool, Error> {
    let canonical = codes::parse(code).unwrap_or_default();
    let supplied = codes::digest(account, purpose, &canonical);
    let row: Option<(Vec<u8>, bool)> = db.query_row(
        "SELECT c.digest,c.consumed=0 AND c.attempts<5 AND c.expires_unix>?3 AND c.epoch=i.epoch AND i.compromised=0 AND c.email=CASE c.purpose WHEN 'verify' THEN i.email WHEN 'change_new' THEN i.pending_email ELSE i.verified_email END FROM recovery_challenges c JOIN recovery_identity i ON i.account_id=c.account_id WHERE c.account_id=?1 AND c.purpose=?2",
        params![account,purpose,now], |r| Ok((r.get(0)?,r.get(1)?))).optional()?;
    let (digest, active) = row.unwrap_or((vec![0; 32], false));
    let valid = codes::equal(&digest, &supplied) && active && !canonical.is_empty();
    if !valid {
        db.execute("UPDATE recovery_challenges SET attempts=MIN(attempts+1,5) WHERE account_id=?1 AND purpose=?2 AND consumed=0", params![account,purpose])?;
    }
    Ok(valid)
}
pub fn consume(db: &Connection, account: i64, purpose: &str) -> Result<(), Error> {
    db.execute(
        "UPDATE recovery_challenges SET consumed=1 WHERE account_id=?1 AND purpose=?2",
        params![account, purpose],
    )?;
    Ok(())
}
