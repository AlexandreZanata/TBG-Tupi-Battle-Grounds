//! Transactional registration identity and authenticated email lifecycle.
use super::{challenge, email, outbox, Error};
use rusqlite::{params, Connection, OptionalExtension, TransactionBehavior};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

#[derive(Clone)]
pub struct Enrollment {
    pub email: String,
    pub locale: String,
}

pub fn validate(db: &Connection, address: &str, locale: &str) -> Result<Enrollment, Error> {
    let address = email::normalize(address).map_err(|_| Error::EmailInvalid)?;
    if !matches!(locale, "en" | "pt-BR") {
        return Err(Error::EmailInvalid);
    }
    let blocked: bool = db.query_row(
        "SELECT EXISTS(SELECT 1 FROM email_blocks WHERE email=?1)",
        [&address],
        |r| r.get(0),
    )?;
    if blocked {
        return Err(Error::EmailBlocked);
    }
    let count: i64 = db.query_row(
        "SELECT COUNT(*) FROM recovery_identity WHERE email=?1 OR pending_email=?1",
        [&address],
        |r| r.get(0),
    )?;
    if count >= 5 {
        return Err(Error::EmailLimit);
    }
    Ok(Enrollment {
        email: address,
        locale: locale.into(),
    })
}
pub fn registration(db: &Connection, address: &str, locale: &str) -> Result<Enrollment, Error> {
    let enabled: bool = db.query_row(
        "SELECT value FROM admin_settings WHERE key='registration_enabled'",
        [],
        |r| r.get(0),
    )?;
    if !enabled {
        return Err(Error::RegistrationClosed);
    }
    validate(db, address, locale)
}
pub fn enroll(db: &Connection, account: i64, input: &Enrollment, now: i64) -> Result<(), Error> {
    db.execute(
        "INSERT INTO recovery_identity(account_id,email,locale,updated_unix) VALUES(?1,?2,?3,?4)",
        params![account, input.email, input.locale, now],
    )?;
    audit(db, account, now, "email_registered")?;
    let _ = outbox::enqueue(
        db,
        "account_verify",
        "account",
        account,
        &input.email,
        &input.locale,
        now,
    )?;
    Ok(())
}
pub fn audit(db: &Connection, account: i64, now: i64, action: &str) -> Result<(), Error> {
    db.execute("INSERT INTO recovery_audit(account_id,at_unix,epoch,action) SELECT account_id,?2,epoch,?3 FROM recovery_identity WHERE account_id=?1",params![account,now,action])?;
    Ok(())
}
pub fn mask(address: &str) -> String {
    let Some((local, domain)) = address.split_once('@') else {
        return String::new();
    };
    let first = local.chars().next().unwrap_or('*');
    let last = if local.len() > 1 {
        local.chars().last().unwrap_or('*').to_string()
    } else {
        String::new()
    };
    format!("{first}***{last}@{domain}")
}
pub fn panel(db: &Connection, account: i64, now: i64) -> Result<Value, Error> {
    let name: String = db.query_row("SELECT name FROM accounts WHERE id=?1", [account], |r| {
        r.get(0)
    })?;
    let row: Option<(Option<String>, Option<String>, Option<i64>)> = db
        .query_row(
            "SELECT email,verified_email,pending_unix FROM recovery_identity WHERE account_id=?1",
            [account],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .optional()?;
    let (email, verified, hold) = row.unwrap_or_default();
    let state = if verified.is_some() {
        "verified"
    } else if email.is_some() {
        "unverified"
    } else {
        "none"
    };
    let moderation =
        super::super::moderation::state(db, account).map_err(|_| Error::Unavailable)?;
    let enabled = |key: &str| {
        db.query_row(
            "SELECT value FROM admin_settings WHERE key=?1",
            [key],
            |r| r.get::<_, bool>(0),
        )
    };
    let can_chat = !moderation.banned(now)
        && verified.is_some()
        && !moderation.chat_until.is_some_and(|until| until > now)
        && enabled("chat_enabled")?;
    let can_upload = !moderation.banned(now)
        && verified.is_some()
        && !moderation.emoji_until.is_some_and(|until| until > now)
        && enabled("emoji_uploads_enabled")?;
    Ok(
        json!({"name":name,"email_masked":email.as_deref().map(mask),"email_state":state,"pending_change":hold.map(|until|json!({"until":until})),"chat_until":moderation.chat_until,"emoji_until":moderation.emoji_until,"can_chat":can_chat,"can_upload":can_upload}),
    )
}
pub fn verify(db: &mut Connection, account: i64, code: &str, now: i64) -> Result<(), Error> {
    let tx = db.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let valid = challenge::check(&tx, account, "verify", code, now)?;
    if valid {
        tx.execute("UPDATE recovery_identity SET verified_email=email,verified_unix=?2,updated_unix=?2 WHERE account_id=?1",params![account,now])?;
        audit(&tx, account, now, "verified")?;
    }
    tx.commit()?;
    if valid {
        Ok(())
    } else {
        Err(Error::Denied)
    }
}

/// DB-backed budgets, shared by both slots, hashed scopes only.
pub fn budget(
    db: &Connection,
    scope: &str,
    limit: i64,
    window: i64,
    now: i64,
) -> Result<(), Error> {
    db.execute("DELETE FROM recovery_dispatch_limits WHERE rowid IN (SELECT rowid FROM recovery_dispatch_limits WHERE issued<=?1 LIMIT 256)",[now-86400])?;
    let hash = Sha256::digest(scope.as_bytes());
    let (used,total):(i64,i64)=db.query_row("SELECT (SELECT COUNT(*) FROM recovery_dispatch_limits WHERE bucket=?1 AND issued>?2),(SELECT COUNT(*) FROM recovery_dispatch_limits)",params![hash.as_slice(),now-window],|r|Ok((r.get(0)?,r.get(1)?)))?;
    if used >= limit || total >= 65536 {
        return Err(Error::RateLimited);
    }
    db.execute(
        "INSERT INTO recovery_dispatch_limits VALUES(?1,?2)",
        params![hash.as_slice(), now],
    )?;
    Ok(())
}
pub fn resend(db: &mut Connection, account: i64, ip: &str, now: i64) -> Result<(), Error> {
    let tx = db.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let (email,locale):(String,String)=tx.query_row("SELECT email,locale FROM recovery_identity WHERE account_id=?1 AND verified_email IS NULL",[account],|r|Ok((r.get(0)?,r.get(1)?))).optional()?.ok_or(Error::Denied)?;
    budget(&tx, &format!("verify:account:{account}"), 3, 3600, now)?;
    budget(&tx, &format!("destination:{email}"), 3, 3600, now)?;
    budget(&tx, &format!("mail:ip:{ip}"), 12, 3600, now)?;
    challenge::consume(&tx, account, "verify")?;
    let _ = outbox::enqueue(
        &tx,
        "account_verify",
        "account",
        account,
        &email,
        &locale,
        now,
    )?;
    tx.commit()?;
    Ok(())
}
fn proof(db: &Connection, account: i64, key: &str) -> Result<(), Error> {
    let (name, secret) = tbg_core::account::parse_key(key).ok_or(Error::Denied)?;
    let hash: [u8; 32] = Sha256::digest(secret).into();
    let row: Option<(i64, Vec<u8>)> = db
        .query_row(
            "SELECT id,key_hash FROM accounts WHERE name_lower=?1",
            [name.to_ascii_lowercase()],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()?;
    let (id, stored) = row.unwrap_or((0, vec![0; 32]));
    if id != account || !crate::identity::codes::equal(&stored, &hash) {
        return Err(Error::Denied);
    }
    Ok(())
}
pub fn change(
    db: &mut Connection,
    account: i64,
    key: &str,
    email: &str,
    locale: &str,
    lost_old: bool,
    now: i64,
) -> Result<&'static str, Error> {
    let tx = db.transaction_with_behavior(TransactionBehavior::Immediate)?;
    proof(&tx, account, key)?;
    let input = validate(&tx, email, locale)?;
    let old: Option<String> = tx
        .query_row(
            "SELECT verified_email FROM recovery_identity WHERE account_id=?1",
            [account],
            |r| r.get(0),
        )
        .optional()?
        .flatten();
    if let Some(old) = old {
        if old == input.email {
            return Err(Error::Conflict);
        }
        budget(&tx, &format!("change:{account}"), 3, 3600, now)?;
        challenge::consume(&tx, account, "change_old")?;
        challenge::consume(&tx, account, "change_new")?;
        tx.execute("UPDATE mail_outbox SET status='cancelled' WHERE principal_kind='account' AND principal_id=?1 AND purpose IN ('account_change_old','account_change_new') AND status='pending'",[account])?;
        // 0 denotes a routine change; a positive timestamp is the lost-mailbox hold deadline.
        tx.execute("UPDATE recovery_identity SET pending_email=?2,pending_unix=?3,locale=?4,updated_unix=?5 WHERE account_id=?1",params![account,input.email,if lost_old{now+7*86400}else{0},locale,now])?;
        let _ = outbox::enqueue(
            &tx,
            "account_change_old",
            "account",
            account,
            &old,
            locale,
            now,
        )?;
        let _ = outbox::enqueue(
            &tx,
            "account_change_new",
            "account",
            account,
            &input.email,
            locale,
            now,
        )?;
        audit(&tx, account, now, "change_requested")?;
        tx.commit()?;
        Ok(if lost_old { "hold" } else { "approve_old" })
    } else {
        tx.execute(
            "DELETE FROM recovery_challenges WHERE account_id=?1",
            [account],
        )?;
        tx.execute("INSERT INTO recovery_identity(account_id,email,locale,updated_unix) VALUES(?1,?2,?3,?4) ON CONFLICT(account_id) DO UPDATE SET email=?2,locale=?3,updated_unix=?4",params![account,input.email,locale,now])?;
        audit(&tx, account, now, "email_registered")?;
        let _ = outbox::enqueue(
            &tx,
            "account_verify",
            "account",
            account,
            &input.email,
            locale,
            now,
        )?;
        tx.commit()?;
        Ok("verify_new")
    }
}
pub fn confirm(
    db: &mut Connection,
    account: i64,
    old: &str,
    new: &str,
    now: i64,
) -> Result<bool, Error> {
    let tx = db.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let hold: Option<i64> = tx.query_row(
        "SELECT pending_unix FROM recovery_identity WHERE account_id=?1",
        [account],
        |r| r.get(0),
    )?;
    let Some(hold) = hold else {
        return Err(Error::Denied);
    };
    let new_ok = challenge::proof(&tx, account, "change_new", new, now)?;
    let old_ok = if hold > 0 {
        true
    } else {
        challenge::proof(&tx, account, "change_old", old, now)?
    };
    if !new_ok || !old_ok {
        tx.commit()?;
        return Err(Error::Denied);
    }
    challenge::consume(&tx, account, "change_new")?;
    if hold > now {
        tx.commit()?;
        return Ok(false);
    }
    apply_change(&tx, account, now)?;
    tx.commit()?;
    Ok(true)
}
fn apply_change(db: &Connection, account: i64, now: i64) -> Result<(), Error> {
    let (old, new, locale): (String, String, String) = db.query_row(
        "SELECT verified_email,pending_email,locale FROM recovery_identity WHERE account_id=?1",
        [account],
        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
    )?;
    db.execute("UPDATE recovery_identity SET email=pending_email,verified_email=pending_email,verified_unix=?2,pending_email=NULL,pending_unix=NULL,updated_unix=?2 WHERE account_id=?1",params![account,now])?;
    db.execute(
        "UPDATE recovery_challenges SET consumed=1 WHERE account_id=?1",
        [account],
    )?;
    audit(db, account, now, "email_changed")?;
    for address in [old, new] {
        let _ = outbox::enqueue(
            db,
            "account_email_changed",
            "account",
            account,
            &address,
            &locale,
            now,
        )?;
    }
    Ok(())
}
pub fn cancel(db: &mut Connection, account: i64, old: Option<&str>, now: i64) -> Result<(), Error> {
    let tx = db.transaction_with_behavior(TransactionBehavior::Immediate)?;
    if let Some(code) = old {
        if !challenge::check(&tx, account, "change_old", code, now)? {
            tx.commit()?;
            return Err(Error::Denied);
        }
    }
    tx.execute("UPDATE recovery_identity SET pending_email=NULL,pending_unix=NULL,updated_unix=?2 WHERE account_id=?1",params![account,now])?;
    tx.execute("DELETE FROM recovery_challenges WHERE account_id=?1 AND purpose IN ('change_old','change_new')",[account])?;
    audit(&tx, account, now, "change_cancelled")?;
    tx.commit()?;
    Ok(())
}
pub fn maintenance(db: &mut Connection, now: i64) -> Result<(), Error> {
    let tx = db.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let ids: Vec<i64> = {
        let mut s=tx.prepare("SELECT i.account_id FROM recovery_identity i JOIN recovery_challenges c ON c.account_id=i.account_id AND c.purpose='change_new' WHERE i.pending_unix>0 AND i.pending_unix<=?1 AND c.consumed=1 AND c.epoch=i.epoch LIMIT 256")?;
        let rows = s.query_map([now], |r| r.get(0))?;
        rows.collect::<Result<_, _>>()?
    };
    for id in ids {
        apply_change(&tx, id, now)?;
    }
    tx.commit()?;
    Ok(())
}
