//! Frozen online match evidence independent from optional personal history and replay.
use rusqlite::{params, Connection};
use serde_json::Value;
pub struct Record {
    pub result: [u8; 16],
    pub ended: i64,
    pub mode: String,
    pub accounts: [Option<i64>; 2],
    pub names: [String; 2],
    pub outcome: Value,
    pub loadouts: Value,
    pub transcript: Value,
}
pub fn insert(db: &Connection, r: Record) -> rusqlite::Result<()> {
    db.execute("INSERT OR IGNORE INTO match_social_records(result_id,ended_unix,expires_unix,mode,a_account,b_account,a_name,b_name,outcome_json,loadouts_json,transcript_json) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11)",params![r.result.as_slice(),r.ended,r.ended+7*86400,r.mode,r.accounts[0],r.accounts[1],r.names[0],r.names[1],r.outcome.to_string(),r.loadouts.to_string(),r.transcript.to_string()])?;
    Ok(())
}
pub fn prune(db: &Connection, now: i64) -> rusqlite::Result<()> {
    db.execute("DELETE FROM match_social_records WHERE rowid IN (SELECT rowid FROM match_social_records WHERE expires_unix<=?1 LIMIT 256)",[now])?;
    db.execute("DELETE FROM reports WHERE id IN (SELECT id FROM reports WHERE status<>'open' AND resolved_unix<?1 LIMIT 256)",[now-180*86400])?;
    Ok(())
}
impl super::Store {
    pub(crate) fn social_record(&self, record: Record) {
        // The same bounded writer/fire-and-forget policy as insert_match.
        let run = Box::new(move |db: &mut Connection| {
            if insert(db, record).is_err() {
                tracing::error!("social record unavailable");
            }
        });
        let _ = self.tx.try_send(super::worker::StoreCmd::Release(run));
    }
}

/// Pairing setup only: no database access on a simulation tick or message relay.
pub fn profiles(
    db: &Connection,
    accounts: [Option<i64>; 2],
) -> rusqlite::Result<[(bool, [tbg_core::protocol::EmojiRef; 3]); 2]> {
    let mut profiles = std::array::from_fn(|_| (false, crate::social::emoji::defaults()));
    for (seat, account) in accounts.into_iter().enumerate() {
        let Some(account) = account else {
            continue;
        };
        profiles[seat].0=db.query_row("SELECT EXISTS(SELECT 1 FROM recovery_identity WHERE account_id=?1 AND verified_email IS NOT NULL AND compromised=0)",[account],|r|r.get(0))?;
        let mut s=db.prepare("SELECT s.slot,s.standard_id,e.public_id,e.frame_px,e.frames,e.columns,e.delays_json FROM account_emoji_slots s LEFT JOIN emoji_assets e ON e.id=s.emoji_id AND e.status='approved' AND e.owner_account_id=s.account_id WHERE s.account_id=?1 ORDER BY s.slot")?;
        for row in s.query_map([account], |r| {
            Ok((
                usize::from(r.get::<_, u8>(0)?),
                r.get::<_, Option<u8>>(1)?,
                r.get::<_, Option<Vec<u8>>>(2)?,
                r.get::<_, Option<u8>>(3)?,
                r.get::<_, Option<u8>>(4)?,
                r.get::<_, Option<u8>>(5)?,
                r.get::<_, Option<String>>(6)?,
            ))
        })? {
            let (slot, standard, id, px, frames, columns, delays) = row?;
            if slot >= 3 {
                continue;
            }
            if let Some(id) = standard.filter(|id| *id < 16) {
                profiles[seat].1[slot] = tbg_core::protocol::EmojiRef::Standard { id };
            } else if let (Some(id), Some(frame_px), Some(frames), Some(columns), Some(delays)) =
                (id, px, frames, columns, delays)
            {
                if let Ok(id) = id.try_into() {
                    if let Ok(delays_ms) = serde_json::from_str(&delays) {
                        profiles[seat].1[slot] = tbg_core::protocol::EmojiRef::Custom {
                            id,
                            frame_px,
                            frames,
                            columns,
                            delays_ms,
                        };
                    }
                }
            }
        }
    }
    Ok(profiles)
}
