//! Idempotent claim intent and single-use AdminKey rotation; player accounts are unreachable.
use super::*;
pub fn request(db: &mut Connection, address: &str, ip: &str, now: i64) -> Result<(), Error> {
    let address = email::normalize(address).map_err(|_| Error::Invalid)?;
    let tx = db.transaction_with_behavior(TransactionBehavior::Immediate)?;
    service::budget(&tx, &format!("claim:ip:{ip}"), 12, 3600, now)?;
    service::budget(&tx, &format!("claim:email:{address}"), 3, 3600, now)?;
    let row: Option<(i64, String)> = tx
        .query_row(
            "SELECT id,locale FROM operators WHERE email=?1 AND status<>'disabled'",
            [&address],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()?;
    if let Some((id, locale)) = row {
        tx.execute(
            "UPDATE operator_challenges SET consumed=1 WHERE operator_id=?1 AND purpose='claim'",
            [id],
        )?;
        tx.execute("UPDATE mail_outbox SET status='cancelled' WHERE principal_kind='operator' AND principal_id=?1 AND purpose='operator_claim' AND status='pending'", [id])?;
        let _ = mail_outbox::enqueue(
            &tx,
            "operator_claim",
            "operator",
            id,
            &address,
            &locale,
            now,
        )?;
    }
    tx.commit()?;
    Ok(())
}
pub fn complete(
    db: &mut Connection,
    address: &str,
    code: &str,
    ip: &str,
    request: &str,
    now: i64,
) -> Result<String, Error> {
    let tx = db.transaction_with_behavior(TransactionBehavior::Immediate)?;
    service::budget(&tx, &format!("claim:complete:ip:{ip}"), 30, 3600, now)?;
    let address = email::normalize(address).unwrap_or_default();
    let id: i64 = tx
        .query_row(
            "SELECT id FROM operators WHERE email=?1 AND status<>'disabled'",
            [&address],
            |r| r.get(0),
        )
        .optional()?
        .unwrap_or(0);
    if !check_code(&tx, id, "claim", code, None, now)? {
        tx.commit()?;
        return Err(Error::CodeInvalid);
    }
    let (raw, hash) = key::generate().map_err(|_| Error::Unavailable)?;
    tx.execute("UPDATE operators SET key_hash=?2,key_epoch=key_epoch+1,status='active',claimed_unix=?3 WHERE id=?1",params![id,hash.as_slice(),now])?;
    tx.execute("UPDATE operator_sessions SET revoked_unix=?2,revoke_reason='key_rotated' WHERE operator_id=?1 AND revoked_unix IS NULL",params![id,now])?;
    tx.execute("UPDATE mail_outbox SET status='cancelled' WHERE principal_kind='operator' AND principal_id=?1 AND purpose IN ('operator_login','operator_claim') AND status='pending'",[id])?;
    tx.execute(
        "UPDATE operator_challenges SET consumed=1 WHERE operator_id=?1",
        [id],
    )?;
    audit::insert(
        &tx,
        now,
        &audit_entry(
            Some(id),
            &format!("operator:{id}"),
            "key_rotated",
            "operator",
            &id.to_string(),
            request,
        ),
    )?;
    let locale: String = tx.query_row("SELECT locale FROM operators WHERE id=?1", [id], |r| {
        r.get(0)
    })?;
    let _ = mail_outbox::enqueue(
        &tx,
        "operator_key_rotated",
        "operator",
        id,
        &address,
        &locale,
        now,
    )?;
    tx.commit()?;
    Ok(raw)
}
