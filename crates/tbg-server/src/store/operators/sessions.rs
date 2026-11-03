//! Hash-only operator sessions, emailed second factor and fresh-key reauthentication.
use super::*;
pub struct Granted {
    pub token: String,
    pub csrf: String,
    pub principal: Principal,
}
pub fn csrf(token: &str) -> String {
    key::hash(&format!("admin-csrf:{token}"))
        .iter()
        .map(|v| format!("{v:02x}"))
        .collect()
}
pub fn start(
    db: &mut Connection,
    raw: &str,
    attempt: &str,
    ip: &str,
    now: i64,
    second_factor: bool,
) -> Result<Option<Granted>, Error> {
    let tx = db.transaction_with_behavior(TransactionBehavior::Immediate)?;
    service::budget(&tx, &format!("admin:start:ip:{ip}"), 30, 3600, now)?;
    let id = key_owner(&tx, raw)?;
    if let Some(id) = id {
        service::budget(&tx, &format!("admin:start:operator:{id}"), 10, 3600, now)?;
        if !second_factor {
            let grant = issue(&tx, id, now)?;
            tx.commit()?;
            crate::metrics::p75::add(&crate::metrics::p75::SIGNINS);
            return Ok(Some(grant));
        }
        let (email, locale, epoch): (String, String, i64) = tx.query_row(
            "SELECT email,locale,key_epoch FROM operators WHERE id=?1",
            [id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )?;
        tx.execute("UPDATE operator_challenges SET consumed=1,attempt_hash=NULL WHERE purpose='login' AND attempt_hash=?1 AND operator_id<>?2", params![key::hash(attempt).as_slice(),id])?;
        tx.execute("INSERT INTO operator_challenges(operator_id,purpose,digest,key_epoch,expires_unix,consumed,attempt_hash) VALUES(?1,'login',zeroblob(32),?2,?3,1,?4) ON CONFLICT(operator_id,purpose) DO UPDATE SET digest=zeroblob(32),key_epoch=?2,expires_unix=?3,consumed=1,attempt_hash=?4,attempts=0",params![id,epoch,now+600,key::hash(attempt).as_slice()])?;
        tx.execute("UPDATE mail_outbox SET status='cancelled' WHERE purpose='operator_login' AND principal_kind='operator' AND principal_id=?1 AND status='pending'",[id])?;
        let _ = mail_outbox::enqueue(&tx, "operator_login", "operator", id, &email, &locale, now)?;
    }
    tx.commit()?;
    if id.is_none() {
        crate::metrics::p75::add(&crate::metrics::p75::SIGNIN_FAILURES);
    }
    Ok(None)
}
pub fn verify(
    db: &mut Connection,
    attempt: &str,
    code: &str,
    ip: &str,
    now: i64,
) -> Result<Granted, Error> {
    let tx = db.transaction_with_behavior(TransactionBehavior::Immediate)?;
    service::budget(&tx, &format!("admin:verify:ip:{ip}"), 30, 3600, now)?;
    let id: i64 = tx
        .query_row(
            "SELECT operator_id FROM operator_challenges WHERE purpose='login' AND attempt_hash=?1",
            [key::hash(attempt).as_slice()],
            |r| r.get(0),
        )
        .optional()?
        .unwrap_or(0);
    if !check_code(&tx, id, "login", code, Some(attempt), now)? {
        tx.commit()?;
        crate::metrics::p75::add(&crate::metrics::p75::SIGNIN_FAILURES);
        return Err(Error::CodeInvalid);
    }
    let granted = issue(&tx, id, now)?;
    tx.commit()?;
    crate::metrics::p75::add(&crate::metrics::p75::SIGNINS);
    Ok(granted)
}
fn issue(db: &Connection, id: i64, now: i64) -> Result<Granted, Error> {
    let token = key::random::<32>().map_err(|_| Error::Unavailable)?;
    let csrf = csrf(&token);
    let (email, role, epoch): (String, String, i64) = db.query_row(
        "SELECT email,role,key_epoch FROM operators WHERE id=?1 AND status='active'",
        [id],
        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
    )?;
    db.execute("UPDATE operator_sessions SET revoked_unix=?2,revoke_reason='session_limit' WHERE token_hash IN (SELECT token_hash FROM operator_sessions WHERE operator_id=?1 AND revoked_unix IS NULL ORDER BY created_unix DESC,rowid DESC LIMIT -1 OFFSET 2)",params![id,now])?;
    db.execute("INSERT INTO operator_sessions(token_hash,operator_id,csrf_hash,key_epoch,created_unix,last_seen_unix,absolute_expires_unix,reauth_unix) VALUES(?1,?2,?3,?4,?5,?5,?6,?5)",params![key::hash(&token).as_slice(),id,key::hash(&csrf).as_slice(),epoch,now,now+8*3600])?;
    let request = key::random::<16>().map_err(|_| Error::Unavailable)?;
    audit::insert(
        db,
        now,
        &audit_entry(
            Some(id),
            &format!("operator:{id}"),
            "signin",
            "operator",
            &id.to_string(),
            &request,
        ),
    )?;
    let locale: String = db.query_row("SELECT locale FROM operators WHERE id=?1", [id], |r| {
        r.get(0)
    })?;
    let _ = mail_outbox::enqueue(
        db,
        "operator_login_notice",
        "operator",
        id,
        &email,
        &locale,
        now,
    )?;
    Ok(Granted {
        token,
        csrf,
        principal: Principal {
            id,
            role,
            email_masked: service::mask(&email),
            epoch,
            reauth: now,
            idle: now + 1800,
            absolute: now + 8 * 3600,
        },
    })
}
pub fn authenticate(
    db: &Connection,
    token: &str,
    csrf_header: Option<&str>,
    now: i64,
) -> Result<Principal, Error> {
    if token.len() != 64 || !token.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(Error::Unauthorized);
    }
    let row:Option<(Principal,Vec<u8>)>=db.query_row("SELECT o.id,o.role,o.email,o.key_epoch,s.reauth_unix,s.last_seen_unix,s.absolute_expires_unix,s.csrf_hash FROM operator_sessions s JOIN operators o ON o.id=s.operator_id WHERE s.token_hash=?1 AND s.revoked_unix IS NULL AND o.status='active' AND s.key_epoch=o.key_epoch AND s.last_seen_unix>?2 AND s.absolute_expires_unix>?3",params![key::hash(token).as_slice(),now-1800,now],|r|Ok((Principal{id:r.get(0)?,role:r.get(1)?,email_masked:service::mask(&r.get::<_,String>(2)?),epoch:r.get(3)?,reauth:r.get(4)?,idle:r.get::<_,i64>(5)?+1800,absolute:r.get(6)?},r.get(7)?))).optional()?;
    let Some((mut p, stored)) = row else {
        return Err(Error::Unauthorized);
    };
    if csrf_header.is_some_and(|v| !codes::equal(&key::hash(v), &stored)) {
        return Err(Error::Forbidden);
    }
    db.execute(
        "UPDATE operator_sessions SET last_seen_unix=?2 WHERE token_hash=?1",
        params![key::hash(token).as_slice(), now],
    )?;
    p.idle = now + 1800;
    Ok(p)
}
pub fn reauth(
    db: &mut Connection,
    token: &str,
    raw: &str,
    request: &str,
    now: i64,
) -> Result<(), Error> {
    let tx = db.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let principal = authenticate(&tx, token, None, now)?;
    service::budget(
        &tx,
        &format!("admin:reauth:{}", principal.id),
        10,
        3600,
        now,
    )?;
    if key_owner(&tx, raw)? != Some(principal.id) {
        tx.commit()?;
        return Err(Error::Unauthorized);
    }
    tx.execute(
        "UPDATE operator_sessions SET reauth_unix=?2 WHERE token_hash=?1",
        params![key::hash(token).as_slice(), now],
    )?;
    audit::insert(
        &tx,
        now,
        &audit_entry(
            Some(principal.id),
            &format!("operator:{}", principal.id),
            "reauth",
            "operator",
            &principal.id.to_string(),
            request,
        ),
    )?;
    tx.commit()?;
    Ok(())
}
pub fn logout(db: &mut Connection, token: &str, request: &str, now: i64) -> Result<(), Error> {
    let tx = db.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let principal = authenticate(&tx, token, None, now)?;
    tx.execute(
        "UPDATE operator_sessions SET revoked_unix=?2,revoke_reason='logout' WHERE token_hash=?1",
        params![key::hash(token).as_slice(), now],
    )?;
    audit::insert(
        &tx,
        now,
        &audit_entry(
            Some(principal.id),
            &format!("operator:{}", principal.id),
            "logout",
            "operator",
            &principal.id.to_string(),
            request,
        ),
    )?;
    tx.commit()?;
    Ok(())
}

/// S04: a process restart ends console authority without affecting player sessions.
pub fn restart(db: &mut Connection, now: i64) -> Result<(), Error> {
    let tx = db.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let changed = tx.execute("UPDATE operator_sessions SET revoked_unix=?1,revoke_reason='restart' WHERE revoked_unix IS NULL",[now])?;
    if changed > 0 {
        let request = key::random::<16>().map_err(|_| Error::Unavailable)?;
        audit::insert(
            &tx,
            now,
            &audit_entry(
                None,
                "startup",
                "sessions_revoked",
                "operator",
                "all",
                &request,
            ),
        )?;
    }
    tx.commit()?;
    Ok(())
}
