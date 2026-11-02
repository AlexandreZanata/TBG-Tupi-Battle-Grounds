//! Console area dispatch with explicit projections and per-action transactions.
use super::*;
use serde::Deserialize;
mod accounts;
mod controls;
mod emoji;
mod operators;
mod reads;
mod reports;
pub struct Context<'a> {
    pub principal: &'a Principal,
    pub request: &'a str,
    pub now: i64,
    pub query: &'a str,
    pub system: Value,
}
pub fn execute(
    db: &mut Connection,
    path: &str,
    method: &str,
    bytes: &[u8],
    context: &Context<'_>,
) -> Result<Value, Error> {
    let parts: Vec<_> = path.split('/').collect();
    match parts.first().copied().unwrap_or("") {
        "accounts" => accounts::execute(db, &parts, method, bytes, context),
        "reports" => reports::execute(db, &parts, method, bytes, context),
        "emoji" => emoji::execute(db, &parts, method, bytes, context),
        "settings" | "chat" | "email-blocks" => {
            controls::execute(db, &parts, method, bytes, context)
        }
        "operators" => operators::execute(db, &parts, method, bytes, context),
        "overview" | "audit" | "system" => reads::execute(db, path, context),
        _ => Err(Error::Invalid),
    }
}
fn parse<T: serde::de::DeserializeOwned>(bytes: &[u8]) -> Result<T, Error> {
    serde_json::from_slice(bytes).map_err(|_| Error::Invalid)
}
fn page(query: &str) -> Result<(i64, i64), Error> {
    let mut after = 0;
    let mut limit = 20;
    for (k, v) in url::form_urlencoded::parse(query.as_bytes()) {
        match k.as_ref() {
            "cursor" => after = v.parse().map_err(|_| Error::Invalid)?,
            "limit" => limit = v.parse().map_err(|_| Error::Invalid)?,
            _ => {}
        }
    }
    if after < 0 || limit < 1 {
        return Err(Error::Invalid);
    }
    Ok((after, limit.min(50)))
}
fn rows(items: Vec<Value>) -> Value {
    let next = items.last().and_then(|r| r.get("id")).cloned();
    json!({"items":items,"cursor":next})
}
fn audit(
    c: &Context<'_>,
    action: &str,
    kind: &str,
    target: &str,
    reason: &str,
    note: Option<String>,
) -> audit::Entry {
    let mut entry = audit_entry(
        Some(c.principal.id),
        &format!("operator:{}", c.principal.id),
        action,
        kind,
        target,
        c.request,
    );
    entry.reason = Some(reason.into());
    entry.note = note;
    entry
}
fn text(s: &str, max: usize) -> bool {
    s.chars().count() <= max
        && s.chars()
            .all(|c| (' '..='~').contains(&c) || ('\u{a0}'..='\u{ff}').contains(&c))
}
fn reason(s: &str) -> Result<(), Error> {
    if s.is_empty() || s.len() > 32 || !s.bytes().all(|b| b.is_ascii_lowercase() || b == b'_') {
        Err(Error::Invalid)
    } else {
        Ok(())
    }
}
fn component(raw: &str) -> String {
    url::form_urlencoded::parse(format!("v={raw}").as_bytes())
        .next()
        .map(|(_, v)| v.into_owned())
        .unwrap_or_default()
}
fn id(raw: Option<&&str>) -> Result<i64, Error> {
    raw.and_then(|v| v.parse().ok())
        .filter(|n| *n > 0)
        .ok_or(Error::Invalid)
}

/// Recheck authorization after acquiring SQLite's writer lock, closing disable/rotation races.
pub fn authorized(
    db: &mut Connection,
    path: &str,
    method: &str,
    bytes: &[u8],
    c: &Context<'_>,
    token: &str,
    csrf: Option<&str>,
) -> Result<Value, Error> {
    db.execute_batch("BEGIN IMMEDIATE")?;
    let result = (|| {
        let principal = super::sessions::authenticate(
            db,
            token,
            if method == "GET" { None } else { csrf },
            c.now,
        )?;
        let route = crate::admin::policy::find(method, path).ok_or(Error::Invalid)?;
        if !crate::admin::policy::allowed(&principal.role, route.capability) {
            return Err(Error::Forbidden);
        }
        if route.sensitive && c.now - principal.reauth >= 300 {
            return Err(Error::Reauth);
        }
        let context = Context {
            principal: &principal,
            request: c.request,
            now: c.now,
            query: c.query,
            system: c.system.clone(),
        };
        execute(db, path, method, bytes, &context)
    })();
    match result {
        Ok(value) => {
            db.execute_batch("COMMIT")?;
            if method == "POST" && path.ends_with("/approve") {
                crate::metrics::p75::add(&crate::metrics::p75::APPROVALS);
            }
            if method == "POST" && path.starts_with("accounts/") && path.ends_with("/ban") {
                crate::metrics::p75::add(&crate::metrics::p75::BANS);
            }
            Ok(value)
        }
        Err(e) => {
            let _ = db.execute_batch("ROLLBACK");
            Err(e)
        }
    }
}

/// Authorization is checked by the dedicated operator router before this store-only read.
pub fn private_sheet(db: &Connection, asset: i64) -> Result<Vec<u8>, Error> {
    db.query_row("SELECT sheet FROM emoji_assets WHERE id=?1", [asset], |r| {
        r.get::<_, Option<Vec<u8>>>(0)
    })
    .optional()?
    .flatten()
    .ok_or(Error::Invalid)
}
