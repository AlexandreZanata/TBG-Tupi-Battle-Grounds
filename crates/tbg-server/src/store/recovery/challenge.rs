//! Single-use account code verification with committed guessing budgets.
use super::Error;
use crate::identity::codes;
use rusqlite::{params, Connection, OptionalExtension};

/// Call inside the caller's transaction. Even false results must be committed.
pub fn check(
    db: &Connection,
    account: i64,
    purpose: &str,
    code: &str,
    now: i64,
) -> Result<bool, Error> {
    let canonical = codes::parse(code).unwrap_or_default();
    let supplied = codes::digest(account, purpose, &canonical);
    let row:Option<(Vec<u8>,bool)>=db.query_row("SELECT c.digest,c.consumed=0 AND c.attempts<5 AND c.expires_unix>?3 AND c.epoch=i.epoch AND i.compromised=0 FROM recovery_challenges c JOIN recovery_identity i ON i.account_id=c.account_id WHERE c.account_id=?1 AND c.purpose=?2",params![account,purpose,now],|r|Ok((r.get(0)?,r.get(1)?))).optional()?;
    let (digest, active) = row.unwrap_or((vec![0; 32], false));
    let valid = codes::equal(&digest, &supplied) && active && !canonical.is_empty();
    db.execute("UPDATE recovery_challenges SET attempts=MIN(attempts+1,5),consumed=CASE WHEN ?3 THEN 1 ELSE consumed END WHERE account_id=?1 AND purpose=?2 AND consumed=0",params![account,purpose,valid])?;
    Ok(valid)
}
