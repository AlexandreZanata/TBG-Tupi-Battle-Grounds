//! Same-account key replacement, with revocation and notice in one transaction.
use super::{challenge, email, outbox, service, Error};
use rusqlite::{params, Connection, OptionalExtension, TransactionBehavior};
use sha2::{Digest, Sha256};
pub struct Completion {
    pub account: i64,
    pub name: String,
    pub key: String,
    pub hash: [u8; 32],
}

pub fn request(
    db: &mut Connection,
    name: &str,
    address: &str,
    locale: &str,
    ip: &str,
    now: i64,
) -> Result<(), Error> {
    let email = email::normalize(address)?;
    if !matches!(locale, "en" | "pt-BR") {
        return Err(Error::Denied);
    }
    let tx = db.transaction_with_behavior(TransactionBehavior::Immediate)?;
    service::budget(&tx, &format!("recovery:ip:{ip}"), 12, 3600, now)?;
    service::budget(
        &tx,
        &format!("recovery:name:{}", name.to_ascii_lowercase()),
        3,
        3600,
        now,
    )?;
    service::budget(&tx, &format!("destination:{email}"), 3, 3600, now)?;
    let account:Option<i64>=tx.query_row("SELECT a.id FROM accounts a JOIN recovery_identity i ON a.id=i.account_id WHERE a.name_lower=?1 AND i.verified_email=?2 AND i.compromised=0",params![name.to_ascii_lowercase(),email],|r|r.get(0)).optional()?;
    if let Some(account) = account {
        // A newly requested code replaces any outstanding dispatch for this purpose.
        tx.execute("UPDATE mail_outbox SET status='cancelled' WHERE principal_kind='account' AND principal_id=?1 AND purpose='account_recovery' AND status='pending'",[account])?;
        let _ = outbox::enqueue(
            &tx,
            "account_recovery",
            "account",
            account,
            &email,
            locale,
            now,
        )?;
    }
    tx.commit()?;
    Ok(())
}
pub fn complete(
    db: &mut Connection,
    name: &str,
    code: &str,
    now: i64,
) -> Result<Completion, Error> {
    let tx = db.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let row: Option<(i64, String)> = tx
        .query_row(
            "SELECT id,name FROM accounts WHERE name_lower=?1",
            [name.to_ascii_lowercase()],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()?;
    let (account, name) = row.unwrap_or((0, String::new()));
    if !challenge::check(&tx, account, "rotate", code, now)? {
        tx.commit()?;
        return Err(Error::Denied);
    }
    let mut random = [0; 10];
    getrandom::fill(&mut random).map_err(|_| Error::Unavailable)?;
    let secret = tbg_core::account::encode_secret(&random);
    let key = tbg_core::account::format_key(&name, &secret);
    let hash: [u8; 32] = Sha256::digest(secret).into();
    let (email, locale): (String, String) = tx.query_row(
        "SELECT verified_email,locale FROM recovery_identity WHERE account_id=?1",
        [account],
        |r| Ok((r.get(0)?, r.get(1)?)),
    )?;
    tx.execute(
        "UPDATE accounts SET key_hash=?2 WHERE id=?1",
        params![account, hash.as_slice()],
    )?;
    tx.execute(
        "UPDATE recovery_identity SET epoch=epoch+1,updated_unix=?2 WHERE account_id=?1",
        params![account, now],
    )?;
    tx.execute(
        "UPDATE recovery_challenges SET consumed=1 WHERE account_id=?1",
        [account],
    )?;
    tx.execute("INSERT INTO recovery_revocations SELECT account_id,epoch,?2 FROM recovery_identity WHERE account_id=?1",params![account,now])?;
    tx.execute("UPDATE auth_sessions SET revoked_at=?2,revoke_reason='recovered' WHERE account_id=?1 AND revoked_at IS NULL",params![account,now])?;
    service::audit(&tx, account, now, "rotated")?;
    let _ = outbox::enqueue(
        &tx,
        "account_key_rotated",
        "account",
        account,
        &email,
        &locale,
        now,
    )?;
    tx.commit()?;
    Ok(Completion {
        account,
        name,
        key,
        hash,
    })
}
