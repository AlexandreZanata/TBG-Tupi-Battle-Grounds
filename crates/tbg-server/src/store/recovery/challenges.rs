//! Purpose-bound single-use challenge persistence and bounded issuance.

use super::proof::{digest, random_hex};
use super::{AccountId, Error};
use rusqlite::{params, OptionalExtension, Transaction};

pub(super) fn issue(
    tx: &Transaction<'_>,
    account: AccountId,
    purpose: &str,
    email: &str,
    now: i64,
    ttl: i64,
) -> Result<String, Error> {
    tx.execute(
        "DELETE FROM recovery_challenges WHERE expires<=?1 OR consumed=1",
        [now],
    )?;
    let count: usize =
        tx.query_row("SELECT COUNT(*) FROM recovery_challenges", [], |r| r.get(0))?;
    if count >= 128 {
        return Err(Error::Unavailable);
    }
    let token = random_hex()?;
    let expires = now.checked_add(ttl).ok_or(Error::Unavailable)?;
    tx.execute(
        "INSERT INTO recovery_challenges(digest,account_id,purpose,email,epoch,expires)
         SELECT ?1,account_id,?3,?4,epoch,?5 FROM recovery_identity WHERE account_id=?2",
        params![
            digest(token.as_bytes()).as_slice(),
            account,
            purpose,
            email,
            expires
        ],
    )?;
    Ok(token)
}

pub(super) fn challenge(
    tx: &Transaction<'_>,
    token: &str,
    purpose: &str,
    now: i64,
) -> Result<(AccountId, String, i64), Error> {
    if token.len() != 64 || !token.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(Error::Denied);
    }
    tx.query_row(
        "SELECT c.account_id,c.email,c.epoch FROM recovery_challenges c
         JOIN recovery_identity i ON c.account_id=i.account_id
         WHERE c.digest=?1 AND c.purpose=?2 AND c.consumed=0 AND c.expires>?3
         AND c.epoch=i.epoch AND i.compromised=0
         AND (c.purpose='verify' OR c.email=i.verified_email)",
        params![digest(token.as_bytes()).as_slice(), purpose, now],
        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
    )
    .optional()?
    .ok_or(Error::Denied)
}

pub(super) fn consume(tx: &Transaction<'_>, account: AccountId) -> Result<(), Error> {
    tx.execute(
        "UPDATE recovery_challenges SET consumed=1 WHERE account_id=?1",
        [account],
    )?;
    Ok(())
}
