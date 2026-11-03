//! Secret-free system/overview cards and the Owner-only audit projection.
use super::*;
pub(super) fn execute(db: &Connection, path: &str, c: &Context<'_>) -> Result<Value, Error> {
    if path == "audit" {
        let (after, limit) = page(c.query)?;
        let q: std::collections::BTreeMap<_, _> =
            url::form_urlencoded::parse(c.query.as_bytes()).collect();
        let kind = q.get("target_kind").map(|s| s.as_ref());
        let target = q.get("target_id").map(|s| s.as_ref());
        let operator = q.get("operator").and_then(|s| s.parse::<i64>().ok());
        let mut s=db.prepare("SELECT id,at_unix,actor,action,target_kind,target_id,reason_code,note,before_json,after_json,request_id FROM admin_audit WHERE id>?1 AND (?3 IS NULL OR target_kind=?3) AND (?4 IS NULL OR target_id=?4) AND (?5 IS NULL OR operator_id=?5) ORDER BY id LIMIT ?2")?;
        let items=s.query_map(params![after,limit,kind,target,operator],|r|Ok(json!({"id":r.get::<_,i64>(0)?,"at":r.get::<_,i64>(1)?,"actor":r.get::<_,String>(2)?,"action":r.get::<_,String>(3)?,"target_kind":r.get::<_,String>(4)?,"target_id":r.get::<_,String>(5)?,"reason":r.get::<_,Option<String>>(6)?,"note":r.get::<_,Option<String>>(7)?,"before":r.get::<_,String>(8)?,"after":r.get::<_,String>(9)?,"request_id":r.get::<_,String>(10)?})))?.collect::<Result<Vec<_>,_>>()?;
        return Ok(rows(items));
    }
    let (pending, failed): (i64, i64) = db.query_row(
        "SELECT SUM(status IN ('pending','sending')),SUM(status='failed') FROM mail_outbox",
        [],
        |r| {
            Ok((
                r.get::<_, Option<i64>>(0)?.unwrap_or(0),
                r.get::<_, Option<i64>>(1)?.unwrap_or(0),
            ))
        },
    )?;
    let last:Option<String>=db.query_row("SELECT last_error FROM mail_outbox WHERE last_error IS NOT NULL ORDER BY id DESC LIMIT 1",[],|r|r.get(0)).optional()?.flatten();
    let mut out = c.system.clone();
    let latest: i64 = db.query_row(
        "SELECT COALESCE(MAX(id),0) FROM moderation_events",
        [],
        |r| r.get(0),
    )?;
    let cursor = out["feed_cursor"].as_i64().unwrap_or(0);
    if let Some(object) = out.as_object_mut() {
        object.remove("feed_cursor");
    }
    out["feed_lag"] = json!(latest.saturating_sub(cursor));
    out["feed_lag_unit"] = json!("events");
    out["outbox"] = json!({"pending":pending,"failed":failed,"last_error":last});
    if path == "overview" {
        let (reports,emoji,bans):(i64,i64,i64)=db.query_row("SELECT (SELECT COUNT(*) FROM reports WHERE status='open'),(SELECT COUNT(*) FROM emoji_assets WHERE status='pending'),(SELECT COUNT(*) FROM account_moderation WHERE ban_until_unix>?1)",[c.now],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?)))?;
        out["open_reports"] = json!(reports);
        out["pending_emojis"] = json!(emoji);
        out["active_bans"] = json!(bans);
        let mut s = db.prepare(
            "SELECT slot,state,matches_running FROM release_slots ORDER BY slot LIMIT 26",
        )?;
        out["slots"]=json!(s.query_map([],|r|Ok(json!({"slot":r.get::<_,String>(0)?,"state":r.get::<_,String>(1)?,"matches":r.get::<_,i64>(2)?})))?.collect::<Result<Vec<_>,_>>()?);
    }
    Ok(out)
}
