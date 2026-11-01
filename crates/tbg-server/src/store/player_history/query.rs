//! Indexed keyset reads with subject-bound cursors and a strict single-participant projection.

use super::{Envelope, Error};
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Clone)]
pub enum Subject {
    Owner { account: i64, name: String },
    Public(String),
}
#[derive(Clone, Default)]
pub struct Query {
    pub limit: usize,
    pub cursor: Option<String>,
    pub mode: Option<String>,
    pub outcome: Option<String>,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Cursor {
    version: u8,
    binding: String,
    ended: i64,
    entry: String,
}

impl Query {
    pub fn parse(raw: Option<&str>) -> Result<Self, Error> {
        let mut q = Self {
            limit: 20,
            ..Self::default()
        };
        let raw = raw.unwrap_or("");
        if raw.len() > 1024 {
            return Err(Error::Invalid);
        }
        let mut seen = std::collections::BTreeSet::new();
        for (key, value) in url::form_urlencoded::parse(raw.as_bytes()) {
            if !seen.insert(key.to_string()) {
                return Err(Error::Invalid);
            }
            match key.as_ref() {
                "limit" => {
                    q.limit = value
                        .parse::<usize>()
                        .map_err(|_| Error::Invalid)?
                        .clamp(1, 50)
                }
                "cursor" if value.len() <= 512 => q.cursor = Some(value.into_owned()),
                "mode" if matches!(value.as_ref(), "casual" | "ranked" | "challenge") => {
                    q.mode = Some(value.into_owned())
                }
                "outcome" if matches!(value.as_ref(), "win" | "loss" | "draw") => {
                    q.outcome = Some(value.into_owned())
                }
                _ => return Err(Error::Invalid),
            }
        }
        Ok(q)
    }
    fn binding(&self, name: &str, revision: i64, public: bool) -> String {
        use sha2::{Digest, Sha256};
        crate::ranked::hex(
            &Sha256::digest(
                format!(
                    "{}:{revision}:{public}:{:?}:{:?}",
                    name.to_ascii_lowercase(),
                    self.mode,
                    self.outcome
                )
                .as_bytes(),
            )[..16],
        )
    }
}

fn resolve(conn: &Connection, subject: &Subject) -> Result<(i64, String, i64, bool), Error> {
    let (owner, name) = match subject {
        Subject::Owner { account, name } => (Some(*account), name),
        Subject::Public(name) => (None, name),
    };
    let found: Option<(i64,String,String,i64)> = conn.query_row("SELECT id,name,history_visibility,history_privacy_revision FROM accounts WHERE name_lower=?1",[name.to_ascii_lowercase()], |r| Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?))).optional()?;
    let (id, name, visibility, revision) = found.ok_or(Error::NotFound)?;
    if owner.is_some_and(|owner| owner != id) {
        return Err(Error::NotFound);
    }
    if owner.is_none() && visibility != "public" {
        return Err(Error::Private);
    }
    Ok((id, name, revision, owner.is_none()))
}

const SELECT: &str = "SELECT s.entry_id,s.seat,h.payload_json,m.id,
  CASE WHEN r.account_id IS NOT NULL THEN r.effective_delta WHEN s.seat=0 THEN m.a_after-m.a_before ELSE m.b_after-m.b_before END
  FROM player_match_history_sides s JOIN player_match_history h ON h.history_key=s.history_key
  LEFT JOIN ranked_matches m ON m.match_key=h.ranked_match_key OR m.id=h.ranked_row_id
  LEFT JOIN ranked_sides r ON r.match_id=m.id AND r.account_id=s.account_id";

fn project(row: &rusqlite::Row<'_>, public: bool, details: bool) -> rusqlite::Result<Value> {
    let id: Vec<u8> = row.get(0)?;
    let seat = usize::from(row.get::<_, u8>(1)?);
    let payload: String = row.get(2)?;
    let e: Envelope = serde_json::from_str(&payload).map_err(|_| rusqlite::Error::InvalidQuery)?;
    let side = e.sides.get(seat).ok_or(rusqlite::Error::InvalidQuery)?;
    let other = &e.sides[1 - seat];
    let summary = &side.summary;
    let opponent = (!public
        || summary.origin == tbg_core::protocol::MatchOrigin::PublicQueue
            && other.account.is_some())
    .then(|| other.name.clone());
    let ranked = summary.mode == tbg_core::protocol::MatchMode::Ranked;
    let stored = row.get::<_, Option<i64>>(3)?.is_some();
    let mut value = json!({"entry_id":crate::ranked::hex(&id),"ended_unix":summary.ended_unix,
        "outcome":summary.outcome,"reason":summary.reason,"mode":summary.mode,"origin": if public && summary.origin != tbg_core::protocol::MatchOrigin::PublicQueue { "private" } else { match summary.origin { tbg_core::protocol::MatchOrigin::PublicQueue => "public_queue", tbg_core::protocol::MatchOrigin::PrivateRoom => "private_room", tbg_core::protocol::MatchOrigin::Invitation => "invitation",tbg_core::protocol::MatchOrigin::Rematch => "rematch",tbg_core::protocol::MatchOrigin::LegacyRanked => "legacy_ranked" } },
        "arena":summary.arena,"wall_ms":summary.wall_ms,"combat_ticks":summary.combat_ticks,"tick_hz":summary.tick_hz,
        "opponent":opponent,"quality":summary.quality,"summary_version":summary.version,
        "rating_status": if !ranked { "unranked" } else if stored { "committed" } else { "pending" },
        "rating_delta": if ranked && stored { row.get::<_,Option<i64>>(4)? } else { None }});
    if details {
        // The DTO contains only this side. Internal keys and the other side are never serialized.
        let mut personal =
            serde_json::to_value(summary).map_err(|_| rusqlite::Error::InvalidQuery)?;
        if public {
            personal["origin"] = value["origin"].clone();
        }
        value["summary"] = personal;
    }
    Ok(value)
}

pub fn list(conn: &mut Connection, subject: &Subject, query: &Query) -> Result<Value, Error> {
    let tx = conn.transaction()?;
    let (account, name, revision, public) = resolve(&tx, subject)?;
    let binding = query.binding(&name, revision, public);
    let (ended, entry) = match &query.cursor {
        None => (i64::MAX, vec![255u8; 16]),
        Some(raw) => {
            let bytes = decode_hex(raw).ok_or(Error::Invalid)?;
            let c: Cursor = serde_json::from_slice(&bytes).map_err(|_| Error::Invalid)?;
            if c.version != 1 || c.binding != binding || c.ended < 0 {
                return Err(Error::Invalid);
            }
            let entry = decode_hex(&c.entry)
                .filter(|b| b.len() == 16)
                .ok_or(Error::Invalid)?;
            (c.ended, entry)
        }
    };
    let sql = format!("{SELECT} WHERE s.account_id=?1 AND (s.ended_unix,s.entry_id)<(?2,?3)
        AND (?4 IS NULL OR h.mode=?4)
        AND (?5 IS NULL OR CASE WHEN h.winner_seat IS NULL THEN 'draw' WHEN h.winner_seat=s.seat THEN 'win' ELSE 'loss' END=?5)
        ORDER BY s.ended_unix DESC,s.entry_id DESC LIMIT ?6");
    let mut stmt = tx.prepare(&sql)?;
    let mut rows = stmt
        .query_map(
            params![
                account,
                ended,
                entry,
                query.mode,
                query.outcome,
                query.limit.clamp(1, 50) as i64 + 1
            ],
            |r| project(r, public, false),
        )?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    let more = rows.len() > query.limit.clamp(1, 50);
    rows.truncate(query.limit.clamp(1, 50));
    let cursor = if more {
        rows.last().and_then(|v| {
            let c = Cursor {
                version: 1,
                binding,
                ended: v["ended_unix"].as_i64()?,
                entry: v["entry_id"].as_str()?.into(),
            };
            serde_json::to_vec(&c).ok().map(|b| crate::ranked::hex(&b))
        })
    } else {
        None
    };
    Ok(
        json!({"version":1,"name":name,"privacy_revision":revision,"rows":rows,"has_more":more,"cursor":cursor}),
    )
}

pub fn detail(conn: &mut Connection, subject: &Subject, entry: &str) -> Result<Value, Error> {
    let entry = decode_hex(entry)
        .filter(|b| b.len() == 16)
        .ok_or(Error::NotFound)?;
    let tx = conn.transaction()?;
    let (account, _, revision, public) = resolve(&tx, subject)?;
    let sql = format!("{SELECT} WHERE s.account_id=?1 AND s.entry_id=?2");
    let mut value = tx
        .query_row(&sql, params![account, entry], |r| project(r, public, true))
        .optional()?
        .ok_or(Error::NotFound)?;
    value["privacy_revision"] = json!(revision);
    if value.to_string().len() > 32768 {
        return Err(Error::Invalid);
    }
    Ok(value)
}

fn decode_hex(s: &str) -> Option<Vec<u8>> {
    if s.len() > 512
        || s.len() % 2 != 0
        || !s
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        return None;
    }
    s.as_bytes()
        .chunks_exact(2)
        .map(|p| {
            std::str::from_utf8(p)
                .ok()
                .and_then(|s| u8::from_str_radix(s, 16).ok())
        })
        .collect()
}
