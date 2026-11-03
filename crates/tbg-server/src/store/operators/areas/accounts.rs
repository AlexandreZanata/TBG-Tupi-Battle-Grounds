//! Account search, private detail and versioned moderation actions; never edits ratings or keys.
use super::*;
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Action {
    duration_hours: Option<i64>,
    #[serde(default)]
    permanent: bool,
    reason: String,
    note: Option<String>,
    report_id: Option<i64>,
    #[serde(default)]
    block_email: bool,
    expected_version: Option<i64>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Support {
    reason: String,
    note: Option<String>,
}
pub(super) fn execute(
    db: &mut Connection,
    p: &[&str],
    method: &str,
    b: &[u8],
    c: &Context<'_>,
) -> Result<Value, Error> {
    if p.len() == 1 {
        return list(db, c);
    }
    let name = component(p[1]);
    if !tbg_core::account::valid_name_shape(&name) {
        return Err(Error::Invalid);
    }
    let account: i64 = db
        .query_row(
            "SELECT id FROM accounts WHERE name_lower=?1",
            [name.to_ascii_lowercase()],
            |r| r.get(0),
        )
        .optional()?
        .ok_or(Error::Invalid)?;
    if method == "GET" {
        return detail(db, account, c);
    }
    if p.get(2) == Some(&"send-recovery") || p.get(2) == Some(&"revoke-sessions") {
        return support(db, account, p[2], b, c);
    }
    let i: Action = parse(b)?;
    reason(&i.reason)?;
    if i.note.as_deref().is_some_and(|v| !text(v, 500)) {
        return Err(Error::Invalid);
    }
    let action = p.get(2).copied().ok_or(Error::Invalid)?;
    let hours = i.duration_hours.unwrap_or(0);
    let (kind, until) = match action {
        "ban" => {
            if i.permanent {
                if c.principal.role != "owner" {
                    return Err(Error::Forbidden);
                }
                ("ban", None)
            } else {
                if !(1..=365 * 24).contains(&hours) {
                    return Err(Error::Invalid);
                }
                if c.principal.role != "owner" && hours > 30 * 24 {
                    return Err(Error::Forbidden);
                }
                ("ban", Some(c.now + hours * 3600))
            }
        }
        "unban" => ("unban", None),
        "chat" | "emoji" => {
            if !(0..=90 * 24).contains(&hours) {
                return Err(Error::Invalid);
            }
            let kind = match (action, hours) {
                ("chat", 0) => "chat_unrestrict",
                ("chat", _) => "chat_restrict",
                ("emoji", 0) => "emoji_unrestrict",
                _ => "emoji_restrict",
            };
            (kind, (hours > 0).then_some(c.now + hours * 3600))
        }
        _ => return Err(Error::Invalid),
    };
    if i.block_email && c.principal.role != "owner" {
        return Err(Error::Forbidden);
    }
    if i.block_email {
        return ban_with_email(db, account, kind, until, i, c);
    }
    apply(db, account, kind, until, i, c)
}
fn apply(
    db: &mut Connection,
    account: i64,
    kind: &str,
    until: Option<i64>,
    i: Action,
    c: &Context<'_>,
) -> Result<Value, Error> {
    if let Some(report) = i.report_id {
        let target: Option<i64> = db
            .query_row("SELECT target_id FROM reports WHERE id=?1", [report], |r| {
                r.get(0)
            })
            .optional()?;
        if target != Some(account) {
            return Err(Error::Invalid);
        }
    }
    let mut entry = audit(
        c,
        kind,
        "account",
        &account.to_string(),
        &i.reason,
        i.note.clone(),
    );
    entry.before = json!({"version":i.expected_version});
    entry.after = json!({"action":kind,"until":until});
    let state = crate::store::moderation::apply(
        db,
        crate::store::moderation::Action {
            account,
            kind: kind.into(),
            until,
            reason: i.reason,
            note: i.note,
            report: i.report_id,
            expected: i.expected_version.ok_or(Error::Invalid)?,
            audit: entry,
        },
        c.now,
    )
    .map_err(|e| match e {
        crate::store::moderation::Error::Conflict => Error::Conflict,
        crate::store::moderation::Error::Invalid => Error::Invalid,
        _ => Error::Unavailable,
    })?;
    Ok(json!({"version":state.version}))
}
fn ban_with_email(
    db: &mut Connection,
    account: i64,
    kind: &str,
    until: Option<i64>,
    mut i: Action,
    c: &Context<'_>,
) -> Result<Value, Error> {
    // One enclosing savepoint keeps optional email blocking atomic with the ban and audit.
    db.execute_batch("SAVEPOINT ban_and_block")?;
    let result = (|| {
        let email: Option<String> = db
            .query_row(
                "SELECT email FROM recovery_identity WHERE account_id=?1",
                [account],
                |r| r.get(0),
            )
            .optional()?
            .flatten();
        if let Some(email) = email {
            db.execute(
                "INSERT OR IGNORE INTO email_blocks VALUES(?1,?2,?3,?4)",
                params![email, c.now, c.principal.id, i.reason],
            )?;
        }
        i.block_email = false;
        apply(db, account, kind, until, i, c)
    })();
    match result {
        Ok(v) => {
            db.execute_batch("RELEASE ban_and_block")?;
            Ok(v)
        }
        Err(e) => {
            let _ = db.execute_batch("ROLLBACK TO ban_and_block; RELEASE ban_and_block");
            Err(e)
        }
    }
}
fn support(
    db: &mut Connection,
    account: i64,
    action: &str,
    b: &[u8],
    c: &Context<'_>,
) -> Result<Value, Error> {
    let i: Support = parse(b)?;
    reason(&i.reason)?;
    if i.note.as_deref().is_some_and(|v| !text(v, 500)) {
        return Err(Error::Invalid);
    }
    let tx = db.savepoint()?;
    if action == "send-recovery" {
        let row:Option<(String,String)>=tx.query_row("SELECT verified_email,locale FROM recovery_identity WHERE account_id=?1 AND verified_email IS NOT NULL",[account],|r|Ok((r.get(0)?,r.get(1)?))).optional()?;
        if let Some((email, locale)) = row {
            service::budget(&tx, &format!("support:recovery:{account}"), 3, 3600, c.now)?;
            let _ = mail_outbox::enqueue(
                &tx,
                "account_recovery",
                "account",
                account,
                &email,
                &locale,
                c.now,
            )?;
        }
    } else {
        tx.execute("UPDATE auth_sessions SET revoked_at=?2,revoke_reason='operator' WHERE account_id=?1 AND revoked_at IS NULL",params![account,c.now])?;
    }
    audit::insert(
        &tx,
        c.now,
        &audit(
            c,
            action,
            "account",
            &account.to_string(),
            &i.reason,
            i.note,
        ),
    )?;
    tx.commit()?;
    Ok(json!({"ok":true}))
}
fn list(db: &Connection, c: &Context<'_>) -> Result<Value, Error> {
    let (after, limit) = page(c.query)?;
    let prefix = url::form_urlencoded::parse(c.query.as_bytes())
        .find(|(k, _)| k == "prefix")
        .map(|(_, v)| v.to_ascii_lowercase())
        .unwrap_or_default();
    if prefix.len() < 2
        || prefix.len() > 16
        || !prefix
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_')
    {
        return Err(Error::Invalid);
    }
    let mut s=db.prepare("SELECT id,name,rating,ranked_games FROM accounts WHERE id>?1 AND substr(name_lower,1,length(?2))=?2 ORDER BY id LIMIT ?3")?;
    let items=s.query_map(params![after,prefix,limit],|r|Ok(json!({"id":r.get::<_,i64>(0)?,"name":r.get::<_,String>(1)?,"rating":r.get::<_,i64>(2)?,"games":r.get::<_,i64>(3)?})))?.collect::<Result<Vec<_>,_>>()?;
    Ok(rows(items))
}
fn detail(db: &Connection, account: i64, c: &Context<'_>) -> Result<Value, Error> {
    let mut panel = service::panel(db, account, c.now)?;
    panel["id"] = json!(account);
    let state = crate::store::moderation::state(db, account).map_err(|_| Error::Unavailable)?;
    panel["moderation"] = json!({"version":state.version,"ban_until":state.ban_until,"reason":state.reason.map(|r|r.as_str()),"chat_until":state.chat_until,"emoji_until":state.emoji_until});
    if c.principal.role == "owner" {
        let email: Option<String> = db
            .query_row(
                "SELECT email FROM recovery_identity WHERE account_id=?1",
                [account],
                |r| r.get(0),
            )
            .optional()?
            .flatten();
        panel["email"] = json!(email);
    }
    let (rating, games): (i64, i64) = db.query_row(
        "SELECT rating,ranked_games FROM accounts WHERE id=?1",
        [account],
        |r| Ok((r.get(0)?, r.get(1)?)),
    )?;
    panel["rating"] = json!(rating);
    panel["games"] = json!(games);
    let mut s=db.prepare("SELECT id,at_unix,action,reason,note FROM moderation_actions WHERE account_id=?1 ORDER BY id DESC LIMIT 20")?;
    panel["actions"]=json!(s.query_map([account],|r|Ok(json!({"id":r.get::<_,i64>(0)?,"at":r.get::<_,i64>(1)?,"action":r.get::<_,String>(2)?,"reason":r.get::<_,String>(3)?,"note":r.get::<_,Option<String>>(4)?})))?.collect::<Result<Vec<_>,_>>()?);
    let (received,sent):(i64,i64)=db.query_row("SELECT (SELECT COUNT(*) FROM reports WHERE target_id=?1),(SELECT COUNT(*) FROM reports WHERE reporter_id=?1)",[account],|r|Ok((r.get(0)?,r.get(1)?)))?;
    panel["reports"] = json!({"received":received,"sent":sent});
    let mut s=db.prepare("SELECT hex(result_id),mode,ended_unix,outcome_json FROM match_social_records WHERE a_account=?1 OR b_account=?1 ORDER BY ended_unix DESC LIMIT 10")?;
    panel["matches"]=json!(s.query_map([account],|r|Ok(json!({"result_id":r.get::<_,String>(0)?,"mode":r.get::<_,String>(1)?,"ended":r.get::<_,i64>(2)?,"outcome":r.get::<_,String>(3)?})))?.collect::<Result<Vec<_>,_>>()?);
    Ok(panel)
}
