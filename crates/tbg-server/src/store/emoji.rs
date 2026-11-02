//! Private upload library, ownership-checked slots and approved-only public sheet reads.
use crate::emoji::normalize::Sheet;
use rusqlite::{params, Connection, OptionalExtension, TransactionBehavior};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    TooLarge,
    Unsupported,
    Image(&'static str),
    Invalid,
    Unverified,
    Restricted,
    Disabled,
    QueueFull,
    Limited,
    NotFound,
    Unavailable,
}
impl From<rusqlite::Error> for Error {
    fn from(_: rusqlite::Error) -> Self {
        Self::Unavailable
    }
}
#[derive(Deserialize, Serialize)]
#[serde(untagged, deny_unknown_fields)]
pub enum Slot {
    Standard { standard: u8 },
    Custom { custom: String },
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Slots {
    pub slots: Vec<Slot>,
}
pub fn allowed(db: &Connection, account: i64, now: i64) -> Result<(), Error> {
    if super::moderation::state(db, account)
        .map_err(|_| Error::Unavailable)?
        .emoji_until
        .is_some_and(|u| u > now)
    {
        return Err(Error::Restricted);
    }
    let verified:bool=db.query_row("SELECT EXISTS(SELECT 1 FROM recovery_identity WHERE account_id=?1 AND verified_email IS NOT NULL AND compromised=0)",[account],|r|r.get(0))?;
    if !verified {
        return Err(Error::Unverified);
    }
    let on: bool = db.query_row(
        "SELECT value FROM admin_settings WHERE key='emoji_uploads_enabled'",
        [],
        |r| r.get(0),
    )?;
    if !on {
        return Err(Error::Disabled);
    }
    Ok(())
}
pub fn upload(
    db: &mut Connection,
    account: i64,
    label: &str,
    sheet: Sheet,
    now: i64,
) -> Result<i64, Error> {
    if label.is_empty()
        || label.chars().count() > 24
        || !label
            .chars()
            .all(|c| (' '..='~').contains(&c) || ('\u{a0}'..='\u{ff}').contains(&c))
    {
        return Err(Error::Invalid);
    }
    let tx = db.transaction_with_behavior(TransactionBehavior::Immediate)?;
    allowed(&tx, account, now)?;
    let (today,total,pending):(i64,i64,i64)=tx.query_row("SELECT (SELECT COUNT(*) FROM emoji_assets WHERE owner_account_id=?1 AND created_unix>?2),(SELECT COUNT(*) FROM emoji_assets WHERE owner_account_id=?1 AND status NOT IN ('rejected','withdrawn')),(SELECT COUNT(*) FROM emoji_assets WHERE status='pending')",params![account,now-86400],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?)))?;
    if today >= 3 || total >= 10 {
        return Err(Error::Limited);
    }
    if pending >= 500 {
        return Err(Error::QueueFull);
    }
    tx.execute("INSERT INTO emoji_assets(owner_account_id,content_hash,status,label,sheet,frame_px,frames,columns,delays_json,source_format,source_bytes,created_unix) VALUES(?1,?2,'pending',?3,?4,128,?5,?6,?7,?8,?9,?10)",params![account,sheet.hash.as_slice(),label,sheet.bytes,sheet.frames,sheet.columns,serde_json::to_string(&sheet.delays).map_err(|_|Error::Unavailable)?,sheet.source,sheet.source_bytes as i64,now])?;
    let id = tx.last_insert_rowid();
    tx.commit()?;
    Ok(id)
}
pub fn library(db: &Connection, account: i64) -> Result<Value, Error> {
    let mut s=db.prepare("SELECT id,label,status,decision_reason,created_unix,frames,columns,delays_json,lower(hex(public_id)) FROM emoji_assets WHERE owner_account_id=?1 ORDER BY id DESC LIMIT 100")?;
    let uploads=s.query_map([account],|r|Ok(json!({"id":r.get::<_,i64>(0)?,"label":r.get::<_,String>(1)?,"status":r.get::<_,String>(2)?,"reason":r.get::<_,Option<String>>(3)?,"created":r.get::<_,i64>(4)?,"frames":r.get::<_,i64>(5)?,"columns":r.get::<_,i64>(6)?,"delays_ms":serde_json::from_str::<Value>(&r.get::<_,String>(7)?).unwrap_or(Value::Null),"public_id":r.get::<_,Option<String>>(8)?})))?.collect::<Result<Vec<_>,_>>()?;
    let mut slots = std::array::from_fn::<_, 3, _>(|slot| json!({"standard":slot}));
    let mut s=db.prepare("SELECT s.slot,s.standard_id,hex(e.public_id) FROM account_emoji_slots s LEFT JOIN emoji_assets e ON e.id=s.emoji_id WHERE s.account_id=?1")?;
    for row in s.query_map([account], |r| {
        Ok((
            r.get::<_, usize>(0)?,
            r.get::<_, Option<u8>>(1)?,
            r.get::<_, Option<String>>(2)?,
        ))
    })? {
        let (slot, standard, custom) = row?;
        if slot < 3 {
            slots[slot] = if let Some(standard) = standard {
                json!({"standard":standard})
            } else {
                json!({"custom":custom.unwrap_or_default().to_ascii_lowercase()})
            };
        }
    }
    Ok(json!({"slots":slots,"uploads":uploads,"limits":{"per_day":3,"total":10}}))
}
pub fn equip(db: &mut Connection, account: i64, input: Slots, now: i64) -> Result<Value, Error> {
    if input.slots.len() != 3 {
        return Err(Error::Invalid);
    }
    let tx = db.transaction_with_behavior(TransactionBehavior::Immediate)?;
    for (slot, reference) in input.slots.iter().enumerate() {
        let (standard, emoji) = match reference {
            Slot::Standard { standard } if *standard < 16 => (Some(*standard), None),
            Slot::Custom { custom } => {
                let public = super::reports::hex16(custom).ok_or(Error::Invalid)?;
                let id:Option<i64>=tx.query_row("SELECT id FROM emoji_assets WHERE public_id=?1 AND owner_account_id=?2 AND status='approved'",params![public.as_slice(),account],|r|r.get(0)).optional()?;
                if id.is_none() {
                    return Err(Error::Invalid);
                }
                (None, id)
            }
            _ => return Err(Error::Invalid),
        };
        tx.execute("INSERT INTO account_emoji_slots(account_id,slot,standard_id,emoji_id,updated_unix) VALUES(?1,?2,?3,?4,?5) ON CONFLICT(account_id,slot) DO UPDATE SET standard_id=?3,emoji_id=?4,updated_unix=?5",params![account,slot as i64,standard,emoji,now])?;
    }
    tx.commit()?;
    Ok(json!({"slots":input.slots}))
}
pub fn private(db: &Connection, account: i64, id: i64) -> Result<Vec<u8>, Error> {
    db.query_row("SELECT sheet FROM emoji_assets WHERE id=?1 AND owner_account_id=?2 AND status IN ('pending','approved')",params![id,account],|r|r.get(0)).optional()?.ok_or(Error::NotFound)
}
pub fn approved(db: &Connection, id: &[u8; 16]) -> Result<bool, Error> {
    Ok(db.query_row("SELECT EXISTS(SELECT 1 FROM emoji_assets WHERE public_id=?1 AND status='approved' AND sheet IS NOT NULL)",[id.as_slice()],|r|r.get(0))?)
}
pub fn public(db: &Connection, id: &[u8; 16]) -> Result<Vec<u8>, Error> {
    db.query_row(
        "SELECT sheet FROM emoji_assets WHERE public_id=?1 AND status='approved'",
        [id.as_slice()],
        |r| r.get(0),
    )
    .optional()?
    .ok_or(Error::NotFound)
}
pub fn withdraw(db: &mut Connection, account: i64, id: i64, now: i64) -> Result<(), Error> {
    let tx = db.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let changed=tx.execute("UPDATE emoji_assets SET status='withdrawn',reviewed_unix=?3 WHERE id=?1 AND owner_account_id=?2 AND status IN ('pending','approved')",params![id,account,now])?;
    if changed != 1 {
        return Err(Error::NotFound);
    }
    tx.execute("UPDATE account_emoji_slots SET standard_id=slot,emoji_id=NULL,updated_unix=?2 WHERE emoji_id=?1",params![id,now])?;
    super::moderation::event(&tx, "emoji_revoked", Some(account), &json!({"id":id}), now)?;
    tx.commit()?;
    Ok(())
}
pub fn prune(db: &Connection, now: i64) -> rusqlite::Result<()> {
    db.execute("UPDATE emoji_assets SET sheet=NULL WHERE id IN (SELECT id FROM emoji_assets WHERE status IN ('rejected','withdrawn') AND reviewed_unix<?1 AND sheet IS NOT NULL LIMIT 256)",[now-30*86400])?;
    Ok(())
}
