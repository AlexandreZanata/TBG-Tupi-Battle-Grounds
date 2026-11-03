//! Separate operator principals and session/challenge lifecycle on the writer thread.
use super::{
    audit, mail_outbox,
    recovery::{email, service},
};
use crate::{admin::key, identity::codes};
use rusqlite::{params, Connection, OptionalExtension, TransactionBehavior};
use serde_json::{json, Value};
pub mod claim;
pub mod cli;
pub mod sessions;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    CodeInvalid,
    Unauthorized,
    Forbidden,
    Invalid,
    Conflict,
    Reauth,
    Limited,
    Unavailable,
}
impl From<rusqlite::Error> for Error {
    fn from(_: rusqlite::Error) -> Self {
        Self::Unavailable
    }
}
impl From<super::recovery::Error> for Error {
    fn from(e: super::recovery::Error) -> Self {
        if e == super::recovery::Error::RateLimited {
            Self::Limited
        } else {
            Self::Unavailable
        }
    }
}
#[derive(Clone)]
pub struct Principal {
    pub id: i64,
    pub role: String,
    pub email_masked: String,
    pub epoch: i64,
    pub reauth: i64,
    pub idle: i64,
    pub absolute: i64,
}
impl Principal {
    pub fn dto(&self, csrf: &str) -> Value {
        json!({"operator":{"id":self.id,"role":self.role,"email_masked":self.email_masked},"csrf":csrf,"idle_expires":self.idle,"absolute_expires":self.absolute})
    }
}
pub fn audit_entry(
    operator: Option<i64>,
    actor: &str,
    action: &str,
    kind: &str,
    target: &str,
    request: &str,
) -> audit::Entry {
    audit::Entry {
        operator,
        actor: actor.into(),
        action: action.into(),
        kind: kind.into(),
        target: target.into(),
        reason: None,
        note: None,
        before: json!({}),
        after: json!({}),
        request: request.into(),
    }
}
pub fn bootstrap(db: &mut Connection, address: &str, now: i64) -> Result<(), Error> {
    let address = email::normalize(address).map_err(|_| Error::Invalid)?;
    let tx = db.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let owner: Option<(i64, String, String)> = tx
        .query_row(
            "SELECT id,email,status FROM operators WHERE role='owner'",
            [],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .optional()?;
    let (id, status) = if let Some((id, email, status)) = owner {
        if email != address {
            tracing::warn!(
                "configured bootstrap address differs from existing Owner; keeping the Owner"
            );
        }
        (id, status)
    } else {
        tx.execute("INSERT INTO operators(email,role,status,created_unix,origin) VALUES(?1,'owner','pending_claim',?2,'bootstrap')",params![address,now])?;
        let id = tx.last_insert_rowid();
        let request = key::random::<16>().map_err(|_| Error::Unavailable)?;
        audit::insert(
            &tx,
            now,
            &audit_entry(
                Some(id),
                "bootstrap",
                "bootstrap",
                "operator",
                &id.to_string(),
                &request,
            ),
        )?;
        (id, "pending_claim".into())
    };
    if status == "pending_claim" {
        let last:Option<i64>=tx.query_row("SELECT MAX(created_unix) FROM mail_outbox WHERE purpose='operator_claim' AND principal_id=?1 AND principal_kind='operator'",[id],|r|r.get(0))?;
        if last.is_none_or(|last| now - last >= 86400) {
            let (email, locale): (String, String) = tx.query_row(
                "SELECT email,locale FROM operators WHERE id=?1",
                [id],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )?;
            let _ =
                mail_outbox::enqueue(&tx, "operator_claim", "operator", id, &email, &locale, now)?;
        }
    }
    tx.commit()?;
    Ok(())
}

pub fn check_code(
    db: &Connection,
    id: i64,
    purpose: &str,
    code: &str,
    attempt: Option<&str>,
    now: i64,
) -> Result<bool, Error> {
    let canonical = codes::parse(code).unwrap_or_default();
    let hash = codes::digest(id, &format!("operator:{purpose}"), &canonical);
    let row:Option<(Vec<u8>,bool,Option<Vec<u8>>)>=db.query_row("SELECT c.digest,c.consumed=0 AND c.attempts<5 AND c.expires_unix>?3 AND c.key_epoch=o.key_epoch AND o.status<>'disabled',c.attempt_hash FROM operator_challenges c JOIN operators o ON o.id=c.operator_id WHERE operator_id=?1 AND purpose=?2",params![id,purpose,now],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?))).optional()?;
    let (stored, active, bound) = row.unwrap_or((vec![0; 32], false, None));
    let attempt_ok = match (attempt, bound) {
        (Some(a), Some(h)) => codes::equal(&key::hash(a), &h),
        (None, None) => true,
        _ => false,
    };
    let valid = codes::equal(&stored, &hash) && active && !canonical.is_empty() && attempt_ok;
    db.execute("UPDATE operator_challenges SET attempts=MIN(attempts+1,5),consumed=CASE WHEN ?3 THEN 1 ELSE consumed END WHERE operator_id=?1 AND purpose=?2 AND consumed=0",params![id,purpose,valid])?;
    Ok(valid)
}
pub fn key_owner(db: &Connection, raw: &str) -> Result<Option<i64>, Error> {
    let hash = key::parse(raw).unwrap_or([0; 32]);
    let row: Option<(i64, Vec<u8>)> = db
        .query_row(
            "SELECT id,key_hash FROM operators WHERE key_hash=?1 AND status='active'",
            [hash.as_slice()],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()?;
    let (id, stored) = row.unwrap_or((0, vec![0; 32]));
    Ok((codes::equal(&hash, &stored) && id > 0).then_some(id))
}

/// OP-07: bounded maintenance, outside all match ticks.
pub fn maintenance(db: &Connection, now: i64) -> rusqlite::Result<()> {
    db.execute("DELETE FROM operator_sessions WHERE rowid IN (SELECT rowid FROM operator_sessions WHERE absolute_expires_unix<=?1 OR revoked_unix<?2 LIMIT 256)",params![now,now-86400])?;
    db.execute("DELETE FROM operator_challenges WHERE rowid IN (SELECT rowid FROM operator_challenges WHERE expires_unix<=?1 LIMIT 256)",[now])?;
    Ok(())
}
pub mod areas;
