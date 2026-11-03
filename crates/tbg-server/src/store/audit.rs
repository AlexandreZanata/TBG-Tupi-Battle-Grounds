//! Atomic append-only privileged audit rows; callers own the surrounding transaction.
use rusqlite::{params, Connection};
pub struct Entry {
    pub operator: Option<i64>,
    pub actor: String,
    pub action: String,
    pub kind: String,
    pub target: String,
    pub reason: Option<String>,
    pub note: Option<String>,
    pub before: serde_json::Value,
    pub after: serde_json::Value,
    pub request: String,
}
pub fn insert(db: &Connection, now: i64, entry: &Entry) -> rusqlite::Result<()> {
    db.execute("INSERT INTO admin_audit(at_unix,operator_id,actor,action,target_kind,target_id,reason_code,note,before_json,after_json,request_id) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11)",params![now,entry.operator,entry.actor,entry.action,entry.kind,entry.target,entry.reason,entry.note,entry.before.to_string(),entry.after.to_string(),entry.request])?;
    Ok(())
}
