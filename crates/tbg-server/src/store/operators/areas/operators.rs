//! Owner-only moderator invitations and enable/disable; the Owner cannot be disabled on web.
use super::*;
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Invite {
    email: String,
    role: String,
    reason: Option<String>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Reason {
    reason: String,
}
pub(super) fn execute(
    db: &mut Connection,
    p: &[&str],
    method: &str,
    b: &[u8],
    c: &Context<'_>,
) -> Result<Value, Error> {
    if method == "GET" {
        let (after, limit) = page(c.query)?;
        let mut s = db.prepare(
            "SELECT id,email,role,status,key_epoch FROM operators WHERE id>?1 ORDER BY id LIMIT ?2",
        )?;
        return Ok(rows(s.query_map(params![after,limit],|r|Ok(json!({"id":r.get::<_,i64>(0)?,"email_masked":service::mask(&r.get::<_,String>(1)?),"role":r.get::<_,String>(2)?,"status":r.get::<_,String>(3)?,"key_epoch":r.get::<_,i64>(4)?})))?.collect::<Result<Vec<_>,_>>()?));
    }
    let tx = db.savepoint()?;
    let (id, action, why) = if p.len() == 1 {
        let i: Invite = parse(b)?;
        if i.role != "moderator" {
            return Err(Error::Forbidden);
        }
        let email = email::normalize(&i.email).map_err(|_| Error::Invalid)?;
        let count: i64 = tx.query_row("SELECT COUNT(*) FROM operators", [], |r| r.get(0))?;
        if count >= 1000 {
            return Err(Error::Limited);
        }
        tx.execute("INSERT INTO operators(email,role,status,created_unix,origin) VALUES(?1,'moderator','pending_claim',?2,?3)",params![email,c.now,format!("invite:{}",c.principal.id)])?;
        let id = tx.last_insert_rowid();
        let _ = mail_outbox::enqueue(&tx, "operator_claim", "operator", id, &email, "en", c.now)?;
        (id, "invite", i.reason.unwrap_or_else(|| "invited".into()))
    } else {
        let id = id(p.get(1))?;
        let i: Reason = parse(b)?;
        let action = p.get(2).copied().ok_or(Error::Invalid)?;
        let status = match action {
            "disable" => "disabled",
            "enable" => "active",
            _ => return Err(Error::Invalid),
        };
        let changed=tx.execute("UPDATE operators SET status=CASE WHEN ?2='active' AND key_hash IS NULL THEN 'pending_claim' ELSE ?2 END,disabled_unix=?3 WHERE id=?1 AND role='moderator' AND status<>?2",params![id,status,if status=="disabled"{Some(c.now)}else{None}])?;
        if changed != 1 {
            return Err(Error::Conflict);
        }
        tx.execute("UPDATE operator_sessions SET revoked_unix=?2,revoke_reason='operator_disabled' WHERE operator_id=?1 AND revoked_unix IS NULL",params![id,c.now])?;
        (id, action, i.reason)
    };
    reason(&why)?;
    audit::insert(
        &tx,
        c.now,
        &audit(c, action, "operator", &id.to_string(), &why, None),
    )?;
    tx.commit()?;
    Ok(json!({"id":id}))
}
