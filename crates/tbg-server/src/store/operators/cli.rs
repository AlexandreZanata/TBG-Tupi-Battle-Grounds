//! Audited break-glass operator mutations with no game-account credential access.
use super::*;
pub fn execute(
    db: &mut Connection,
    command: &str,
    operator: Option<i64>,
    email: Option<&str>,
    actor: &str,
    now: i64,
) -> Result<Option<String>, Error> {
    let tx = db.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let id = match operator {
        Some(id) => id,
        None => tx.query_row("SELECT id FROM operators WHERE role='owner'", [], |r| {
            r.get(0)
        })?,
    };
    let mut result = None;
    match command {
        "claim-code" => {
            let code = crate::identity::codes::generate().map_err(|_| Error::Unavailable)?;
            let canonical = crate::identity::codes::parse(&code).ok_or(Error::Unavailable)?;
            tx.execute("INSERT INTO operator_challenges(operator_id,purpose,digest,key_epoch,expires_unix) SELECT id,'claim',?2,key_epoch,?3 FROM operators WHERE id=?1 AND status<>'disabled' ON CONFLICT(operator_id,purpose) DO UPDATE SET digest=?2,key_epoch=excluded.key_epoch,expires_unix=?3,attempts=0,consumed=0",params![id,codes::digest(id,"operator:claim",&canonical).as_slice(),now+86400])?;
            result = Some(code);
        }
        "revoke-sessions" => {}
        "disable" | "enable" => {
            tx.execute(
                "UPDATE operators SET status=?2,disabled_unix=?3 WHERE id=?1",
                params![
                    id,
                    if command == "disable" {
                        "disabled"
                    } else {
                        "active"
                    },
                    if command == "disable" {
                        Some(now)
                    } else {
                        None
                    }
                ],
            )?;
        }
        "set-owner-email" => {
            let address =
                email::normalize(email.ok_or(Error::Invalid)?).map_err(|_| Error::Invalid)?;
            tx.execute("UPDATE operators SET email=?2,key_hash=NULL,key_epoch=key_epoch+1,status='pending_claim' WHERE id=?1 AND role='owner'",params![id,address])?;
        }
        _ => return Err(Error::Invalid),
    }
    if command != "claim-code" {
        tx.execute("UPDATE operator_sessions SET revoked_unix=?2,revoke_reason='cli' WHERE operator_id=?1 AND revoked_unix IS NULL",params![id,now])?;
    }
    let request = key::random::<16>().map_err(|_| Error::Unavailable)?;
    audit::insert(
        &tx,
        now,
        &audit_entry(
            Some(id),
            actor,
            command,
            "operator",
            &id.to_string(),
            &request,
        ),
    )?;
    tx.commit()?;
    Ok(result)
}
