//! Versioned restriction projection and atomic moderation history/feed writes.
use super::audit;
use rusqlite::{params, Connection, OptionalExtension};
use serde_json::json;
use tbg_core::protocol::SuspensionReason;
pub const PERMANENT: i64 = 253402300799;
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct State {
    pub ban_until: Option<i64>,
    pub reason: Option<SuspensionReason>,
    pub chat_until: Option<i64>,
    pub emoji_until: Option<i64>,
    pub version: i64,
}
impl State {
    pub fn banned(&self, now: i64) -> bool {
        self.ban_until.is_some_and(|v| v > now)
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    Unavailable,
    Conflict,
    Invalid,
}
impl From<rusqlite::Error> for Error {
    fn from(_: rusqlite::Error) -> Self {
        Self::Unavailable
    }
}
pub fn state(db: &Connection, account: i64) -> Result<State, Error> {
    Ok(db.query_row("SELECT ban_until_unix,ban_reason,chat_until_unix,emoji_until_unix,version FROM account_moderation WHERE account_id=?1",[account],|r|{let reason:Option<String>=r.get(1)?;Ok(State{ban_until:r.get(0)?,reason:reason.as_deref().and_then(SuspensionReason::parse),chat_until:r.get(2)?,emoji_until:r.get(3)?,version:r.get(4)?})}).optional()?.unwrap_or_default())
}
pub struct Action {
    pub account: i64,
    pub kind: String,
    pub until: Option<i64>,
    pub reason: String,
    pub note: Option<String>,
    pub report: Option<i64>,
    pub expected: i64,
    pub audit: audit::Entry,
}
pub fn apply(db: &mut Connection, action: Action, now: i64) -> Result<State, Error> {
    if db.is_autocommit() {
        let tx = db.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
        let next = write(&tx, action, now)?;
        tx.commit()?;
        Ok(next)
    } else {
        let tx = db.savepoint()?;
        let next = write(&tx, action, now)?;
        tx.commit()?;
        Ok(next)
    }
}
fn write(db: &Connection, action: Action, now: i64) -> Result<State, Error> {
    if SuspensionReason::parse(&action.reason).is_none()
        || action.note.as_ref().is_some_and(|s| s.len() > 1024)
    {
        return Err(Error::Invalid);
    }
    let tx = db;
    let before = state(tx, action.account)?;
    if before.version != action.expected {
        return Err(Error::Conflict);
    }
    let mut next = before;
    next.version += 1;
    match action.kind.as_str() {
        "ban" => {
            next.ban_until = Some(action.until.unwrap_or(PERMANENT));
            next.reason = SuspensionReason::parse(&action.reason);
        }
        "unban" => {
            next.ban_until = None;
            next.reason = None;
        }
        "chat_restrict" | "chat_unrestrict" => next.chat_until = action.until,
        "emoji_restrict" | "emoji_unrestrict" => next.emoji_until = action.until,
        "revoke_sessions" | "send_recovery" => {}
        _ => return Err(Error::Invalid),
    }
    tx.execute("INSERT INTO account_moderation(account_id,ban_until_unix,ban_reason,chat_until_unix,emoji_until_unix,version,updated_unix) VALUES(?1,?2,?3,?4,?5,?6,?7) ON CONFLICT(account_id) DO UPDATE SET ban_until_unix=?2,ban_reason=?3,chat_until_unix=?4,emoji_until_unix=?5,version=?6,updated_unix=?7",params![action.account,next.ban_until,next.reason.map(SuspensionReason::as_str),next.chat_until,next.emoji_until,next.version,now])?;
    tx.execute("INSERT INTO moderation_actions(at_unix,operator_id,account_id,action,until_unix,reason,note,report_id) VALUES(?1,?2,?3,?4,?5,?6,?7,?8)",params![now,action.audit.operator,action.account,action.kind,action.until,action.reason,action.note,action.report])?;
    let kind = match action.kind.as_str() {
        "ban" | "unban" => action.kind.as_str(),
        "chat_restrict" | "chat_unrestrict" => "chat",
        "emoji_restrict" | "emoji_unrestrict" => "emoji",
        _ => "settings",
    };
    event(
        tx,
        kind,
        Some(action.account),
        &json!({"version":next.version}),
        now,
    )?;
    if action.kind == "ban" || action.kind == "revoke_sessions" {
        tx.execute("UPDATE auth_sessions SET revoked_at=?2,revoke_reason=?3 WHERE account_id=?1 AND revoked_at IS NULL",params![action.account,now,if action.kind=="ban"{"banned"}else{"operator"}])?;
    }
    audit::insert(tx, now, &action.audit)?;
    Ok(next)
}
pub fn event(
    db: &Connection,
    kind: &str,
    account: Option<i64>,
    payload: &serde_json::Value,
    now: i64,
) -> rusqlite::Result<()> {
    db.execute(
        "INSERT INTO moderation_events(at_unix,kind,account_id,payload_json) VALUES(?1,?2,?3,?4)",
        params![now, kind, account, payload.to_string()],
    )?;
    Ok(())
}
pub mod feed;
