//! Durable secret-free mail intents, transactional dispatch leases and bounded retries.

use crate::identity::codes;
use rusqlite::{params, Connection, OptionalExtension, TransactionBehavior};

pub struct Delivery {
    pub id: i64,
    pub purpose: String,
    pub to: String,
    pub locale: String,
    pub code: Option<String>,
}

pub fn enqueue(
    conn: &Connection,
    purpose: &str,
    kind: &str,
    id: i64,
    email: &str,
    locale: &str,
    now: i64,
) -> rusqlite::Result<bool> {
    let changed=conn.execute("INSERT INTO mail_outbox(purpose,principal_kind,principal_id,to_email,locale,created_unix,not_before_unix) SELECT ?1,?2,?3,?4,?5,?6,?6 WHERE (SELECT COUNT(*) FROM mail_outbox WHERE status IN ('pending','sending'))<2000",params![purpose,kind,id,email,locale,now])?;
    Ok(changed == 1)
}

pub fn lease(conn: &mut Connection, holder: &str, now: i64) -> rusqlite::Result<Option<Delivery>> {
    let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
    tx.execute("INSERT INTO mail_dispatch_lease VALUES(1,?1,?2) ON CONFLICT(singleton) DO UPDATE SET holder=?1,until_unix=?2 WHERE until_unix<=?3 OR holder=?1",params![holder,now+30,now])?;
    let mine: bool = tx.query_row(
        "SELECT holder=?1 AND until_unix>?2 FROM mail_dispatch_lease WHERE singleton=1",
        params![holder, now],
        |r| r.get(0),
    )?;
    if !mine {
        return Ok(None);
    }
    let row:Option<(i64,String,String,i64,String,String)>=tx.query_row("SELECT id,purpose,principal_kind,principal_id,to_email,locale FROM mail_outbox WHERE (status='pending' AND not_before_unix<=?1) OR (status='sending' AND lease_until_unix<=?1) ORDER BY id LIMIT 1",[now],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?,r.get(5)?))).optional()?;
    let Some((id, purpose, kind, principal, to, locale)) = row else {
        tx.commit()?;
        return Ok(None);
    };
    let proof = match purpose.as_str() {
        "account_verify" => Some(("verify", 86400)),
        "account_recovery" => Some(("rotate", 900)),
        "account_change_old" => Some(("change_old", 86400)),
        "account_change_new" => Some(("change_new", 86400)),
        "operator_claim" => Some(("claim", 86400)),
        "operator_login" => Some(("login", 600)),
        _ => None,
    };
    let mut code = None;
    if let Some((bound, ttl)) = proof {
        let epoch: Option<i64> = if kind == "account" {
            let field = match bound {
                "verify" => "email",
                "change_new" => "pending_email",
                _ => "verified_email",
            };
            tx.query_row(&format!("SELECT epoch FROM recovery_identity WHERE account_id=?1 AND {field}=?2 AND compromised=0"),params![principal,to],|r|r.get(0)).optional()?
        } else {
            tx.query_row(
                "SELECT key_epoch FROM operators WHERE id=?1 AND email=?2 AND status<>'disabled'",
                params![principal, to],
                |r| r.get(0),
            )
            .optional()?
        };
        let Some(epoch) = epoch else {
            tx.execute(
                "UPDATE mail_outbox SET status='cancelled' WHERE id=?1",
                [id],
            )?;
            tx.commit()?;
            return Ok(None);
        };
        let raw = codes::generate().map_err(|_| rusqlite::Error::InvalidQuery)?;
        let canonical = codes::parse(&raw).ok_or(rusqlite::Error::InvalidQuery)?;
        // Domain separation keeps operator proofs unusable as player proofs.
        let domain = if kind == "operator" {
            format!("operator:{bound}")
        } else {
            bound.into()
        };
        let digest = codes::digest(principal, &domain, &canonical);
        if kind == "account" {
            tx.execute("INSERT INTO recovery_challenges(account_id,purpose,digest,email,epoch,expires_unix) VALUES(?1,?2,?3,?4,?5,?6) ON CONFLICT(account_id,purpose) DO UPDATE SET digest=?3,email=?4,epoch=?5,expires_unix=?6,attempts=0,consumed=0",params![principal,bound,digest.as_slice(),to,epoch,now+ttl])?;
        } else {
            tx.execute("INSERT INTO operator_challenges(operator_id,purpose,digest,key_epoch,expires_unix) VALUES(?1,?2,?3,?4,?5) ON CONFLICT(operator_id,purpose) DO UPDATE SET digest=?3,key_epoch=?4,expires_unix=?5,attempts=0,consumed=0",params![principal,bound,digest.as_slice(),epoch,now+ttl])?;
        }
        code = Some(raw);
    }
    tx.execute("UPDATE mail_outbox SET status='sending',lease_owner=?2,lease_until_unix=?3,attempts=MIN(attempts+1,8) WHERE id=?1",params![id,holder,now+30])?;
    tx.commit()?;
    Ok(Some(Delivery {
        id,
        purpose,
        to,
        locale,
        code,
    }))
}

pub fn finish(
    conn: &Connection,
    id: i64,
    holder: &str,
    now: i64,
    error: Option<&str>,
) -> rusqlite::Result<()> {
    let attempts: i64 = conn.query_row(
        "SELECT attempts FROM mail_outbox WHERE id=?1 AND status='sending' AND lease_owner=?2",
        params![id, holder],
        |r| r.get(0),
    )?;
    let delay = match attempts {
        0 | 1 => 60,
        2 => 300,
        3 => 1800,
        4 => 7200,
        _ => 21600,
    };
    let status = if error.is_none() {
        "sent"
    } else if attempts >= 8 {
        "failed"
    } else {
        "pending"
    };
    conn.execute("UPDATE mail_outbox SET status=?3,last_error=?4,sent_unix=?5,not_before_unix=?6,lease_until_unix=NULL WHERE id=?1 AND lease_owner=?2",params![id,holder,status,error,error.is_none().then_some(now),now+delay])?;
    Ok(())
}

pub fn prune(conn: &Connection, now: i64) -> rusqlite::Result<()> {
    conn.execute("DELETE FROM mail_outbox WHERE id IN (SELECT id FROM mail_outbox WHERE status IN ('sent','cancelled','failed') AND created_unix<?1 LIMIT 256)",[now-30*86400])?;
    Ok(())
}
