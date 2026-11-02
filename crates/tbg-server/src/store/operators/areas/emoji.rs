//! Premoderation review transitions; public ids are assigned once and never reused.
use super::*;
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Review {
    reason: Option<String>,
}
pub(super) fn execute(
    db: &mut Connection,
    p: &[&str],
    method: &str,
    b: &[u8],
    c: &Context<'_>,
) -> Result<Value, Error> {
    if method == "POST" {
        let id = id(p.get(1))?;
        let i: Review = parse(b)?;
        let action = p.get(2).copied().ok_or(Error::Invalid)?;
        let why = i.reason.unwrap_or_else(|| action.into());
        reason(&why)?;
        let tx = db.savepoint()?;
        let (from, to) = match action {
            "approve" => ("pending", "approved"),
            "reject" => ("pending", "rejected"),
            "revoke" => ("approved", "revoked"),
            _ => return Err(Error::Invalid),
        };
        let public = if action == "approve" {
            let mut b = [0; 16];
            getrandom::fill(&mut b).map_err(|_| Error::Unavailable)?;
            Some(b.to_vec())
        } else {
            None
        };
        let changed=tx.execute("UPDATE emoji_assets SET status=?2,public_id=COALESCE(public_id,?3),reviewed_unix=?4,reviewer_id=?5,decision_reason=?6 WHERE id=?1 AND status=?7",params![id,to,public,c.now,c.principal.id,why,from])?;
        if changed != 1 {
            return Err(Error::Conflict);
        }
        if action == "revoke" {
            tx.execute("UPDATE account_emoji_slots SET standard_id=slot,emoji_id=NULL,updated_unix=?2 WHERE emoji_id=?1",params![id,c.now])?;
        }
        if action != "reject" {
            crate::store::moderation::event(
                &tx,
                if action == "approve" {
                    "emoji_approved"
                } else {
                    "emoji_revoked"
                },
                None,
                &json!({"id":id}),
                c.now,
            )?;
        }
        audit::insert(
            &tx,
            c.now,
            &audit(c, action, "emoji", &id.to_string(), &why, None),
        )?;
        tx.commit()?;
        return Ok(json!({"ok":true}));
    }
    let (after, limit) = page(c.query)?;
    let target = if p.len() > 1 {
        Some(id(p.get(1))?)
    } else {
        None
    };
    let status = url::form_urlencoded::parse(c.query.as_bytes())
        .find(|(k, _)| k == "status")
        .map(|(_, v)| v.into_owned())
        .unwrap_or_else(|| "pending".into());
    let mut s=db.prepare("SELECT e.id,e.label,e.status,e.owner_account_id,a.name,e.frames,e.frame_px,e.columns,e.delays_json,e.decision_reason,e.created_unix FROM emoji_assets e JOIN accounts a ON a.id=e.owner_account_id WHERE (?1 IS NULL AND e.id>?2 AND e.status=?3) OR e.id=?1 ORDER BY e.id LIMIT ?4")?;
    let items=s.query_map(params![target,after,status,limit],|r|Ok(json!({"id":r.get::<_,i64>(0)?,"label":r.get::<_,String>(1)?,"status":r.get::<_,String>(2)?,"owner_id":r.get::<_,i64>(3)?,"owner":r.get::<_,String>(4)?,"frames":r.get::<_,i64>(5)?,"frame_px":r.get::<_,i64>(6)?,"columns":r.get::<_,i64>(7)?,"delays_ms":serde_json::from_str::<Value>(&r.get::<_,String>(8)?).unwrap_or(Value::Null),"reason":r.get::<_,Option<String>>(9)?,"created":r.get::<_,i64>(10)?})))?.collect::<Result<Vec<_>,_>>()?;
    if target.is_some() {
        items.into_iter().next().ok_or(Error::Invalid)
    } else {
        Ok(rows(items))
    }
}
