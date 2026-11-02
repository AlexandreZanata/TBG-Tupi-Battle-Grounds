//! Report queue, frozen evidence and optimistic resolution.
use super::*;
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Resolve {
    outcome: String,
    note: String,
    expected_status: String,
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
        let i: Resolve = parse(b)?;
        if i.expected_status != "open"
            || !matches!(i.outcome.as_str(), "actioned" | "dismissed")
            || !text(&i.note, 500)
        {
            return Err(Error::Invalid);
        }
        let tx = db.savepoint()?;
        let changed=tx.execute("UPDATE reports SET status=?2,resolved_unix=?3,resolved_by=?4,resolution_note=?5 WHERE id=?1 AND status='open'",params![id,i.outcome,c.now,c.principal.id,i.note])?;
        if changed != 1 {
            return Err(Error::Conflict);
        }
        audit::insert(
            &tx,
            c.now,
            &audit(
                c,
                "resolve",
                "report",
                &id.to_string(),
                &i.outcome,
                Some(i.note),
            ),
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
        .unwrap_or_else(|| "open".into());
    if !matches!(status.as_str(), "open" | "actioned" | "dismissed") {
        return Err(Error::Invalid);
    }
    let mut s=db.prepare("SELECT id,created_unix,reporter_id,target_id,reason,context,comment,evidence_json,status,resolution_note,(SELECT name FROM accounts WHERE id=reports.target_id) FROM reports WHERE (?1 IS NULL AND id>?2 AND status=?3) OR id=?1 ORDER BY id LIMIT ?4")?;
    let mut items=s.query_map(params![target,after,status,limit],|r|Ok(json!({"id":r.get::<_,i64>(0)?,"created":r.get::<_,i64>(1)?,"reporter_id":r.get::<_,i64>(2)?,"target_id":r.get::<_,i64>(3)?,"reason":r.get::<_,String>(4)?,"context":r.get::<_,String>(5)?,"comment":r.get::<_,Option<String>>(6)?,"evidence":serde_json::from_str::<Value>(&r.get::<_,String>(7)?).unwrap_or(Value::Null),"status":r.get::<_,String>(8)?,"resolution_note":r.get::<_,Option<String>>(9)?,"target_name":r.get::<_,String>(10)?})))?.collect::<Result<Vec<_>,_>>()?;
    if target.is_none() {
        for item in &mut items {
            if let Some(row) = item.as_object_mut() {
                row.remove("evidence");
            }
        }
    }
    if target.is_some() {
        items.into_iter().next().ok_or(Error::Invalid)
    } else {
        Ok(rows(items))
    }
}
