//! Audited kill switches, blocked terms and exact-address registration blocks.
use super::*;
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Setting {
    value: u8,
    reason: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Term {
    term: String,
    reason: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Block {
    email: String,
    reason: String,
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
        return list(db, p[0], c);
    }
    let tx = db.savepoint()?;
    let (action, kind, target, why, event) = match p[0] {
        "settings" => {
            let i: Setting = parse(b)?;
            reason(&i.reason)?;
            if i.value > 1 {
                return Err(Error::Invalid);
            }
            let key = p.get(1).ok_or(Error::Invalid)?;
            let changed=tx.execute("UPDATE admin_settings SET value=?2,updated_unix=?3,operator_id=?4 WHERE key=?1 AND value<>?2",params![key,i.value,c.now,c.principal.id])?;
            if changed != 1 {
                return Err(Error::Conflict);
            }
            ("setting", "setting", key.to_string(), i.reason, "settings")
        }
        "chat" => {
            let (term, why) = if method == "POST" {
                let i: Term = parse(b)?;
                (i.term, i.reason)
            } else {
                let i: Reason = parse(b)?;
                (component(p.get(2).ok_or(Error::Invalid)?), i.reason)
            };
            reason(&why)?;
            let term = crate::moderation::terms::fold(&term);
            if !(2..=32).contains(&term.chars().count()) || !text(&term, 32) {
                return Err(Error::Invalid);
            }
            let changed = if method == "POST" {
                let count: i64 =
                    tx.query_row("SELECT COUNT(*) FROM chat_blocked_terms", [], |r| r.get(0))?;
                if count >= 4096 {
                    return Err(Error::Limited);
                }
                tx.execute(
                    "INSERT OR IGNORE INTO chat_blocked_terms VALUES(?1,?2,?3)",
                    params![term, c.now, c.principal.id],
                )?
            } else {
                tx.execute("DELETE FROM chat_blocked_terms WHERE term=?1", [&term])?
            };
            if changed != 1 {
                return Err(Error::Conflict);
            }
            ("term", "term", term, why, "terms")
        }
        "email-blocks" => {
            let (address, why) = if method == "POST" {
                let i: Block = parse(b)?;
                (i.email, i.reason)
            } else {
                let i: Reason = parse(b)?;
                (component(p.get(1).ok_or(Error::Invalid)?), i.reason)
            };
            reason(&why)?;
            let address = email::normalize(&address).map_err(|_| Error::Invalid)?;
            let changed = if method == "POST" {
                tx.execute(
                    "INSERT OR IGNORE INTO email_blocks VALUES(?1,?2,?3,?4)",
                    params![address, c.now, c.principal.id, why],
                )?
            } else {
                tx.execute("DELETE FROM email_blocks WHERE email=?1", [&address])?
            };
            if changed != 1 {
                return Err(Error::Conflict);
            }
            let target = key::hash(&address)
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect();
            ("email_block", "email_block", target, why, "settings")
        }
        _ => return Err(Error::Invalid),
    };
    crate::store::moderation::event(&tx, event, None, &json!({"target":target}), c.now)?;
    audit::insert(&tx, c.now, &audit(c, action, kind, &target, &why, None))?;
    tx.commit()?;
    Ok(json!({"ok":true}))
}
fn list(db: &Connection, area: &str, c: &Context<'_>) -> Result<Value, Error> {
    if area == "settings" {
        let mut s = db.prepare("SELECT key,value FROM admin_settings ORDER BY key")?;
        let items = s
            .query_map([], |r| {
                Ok(json!({"key":r.get::<_,String>(0)?,"value":r.get::<_,i64>(1)?}))
            })?
            .collect::<Result<Vec<_>, _>>()?;
        return Ok(json!({"items":items}));
    }
    let (after, limit) = page(c.query)?;
    let sql = if area == "chat" {
        "SELECT rowid,term,created_unix FROM chat_blocked_terms WHERE rowid>?1 ORDER BY rowid LIMIT ?2"
    } else {
        "SELECT rowid,email,created_unix FROM email_blocks WHERE rowid>?1 ORDER BY rowid LIMIT ?2"
    };
    let mut s = db.prepare(sql)?;
    let items=s.query_map(params![after,limit],|r|Ok(json!({"id":r.get::<_,i64>(0)?,"value":r.get::<_,String>(1)?,"created":r.get::<_,i64>(2)?})))?.collect::<Result<Vec<_>,_>>()?;
    Ok(rows(items))
}
