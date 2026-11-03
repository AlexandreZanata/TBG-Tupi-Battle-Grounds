//! Participant-derived report targets, immutable server evidence and shared filing limits.
use rusqlite::{params, Connection, OptionalExtension, TransactionBehavior};
use serde::Deserialize;
use serde_json::{json, Value};
#[derive(Deserialize)]
#[serde(tag = "context", rename_all = "snake_case", deny_unknown_fields)]
pub enum Input {
    Match {
        result_id: String,
        reason: String,
        comment: Option<String>,
    },
    Profile {
        target: String,
        reason: String,
        comment: Option<String>,
    },
    Emoji {
        result_id: String,
        emoji: String,
        reason: String,
        comment: Option<String>,
    },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    Invalid,
    NotEligible,
    Duplicate,
    Limited,
    Disabled,
    Forbidden,
    Unavailable,
}
impl From<rusqlite::Error> for Error {
    fn from(_: rusqlite::Error) -> Self {
        Self::Unavailable
    }
}
pub fn hex16(value: &str) -> Option<[u8; 16]> {
    if value.len() != 32 || !value.is_ascii() {
        return None;
    }
    let mut bytes = [0; 16];
    for (i, part) in value.as_bytes().chunks(2).enumerate() {
        bytes[i] = u8::from_str_radix(std::str::from_utf8(part).ok()?, 16).ok()?;
    }
    Some(bytes)
}
type EvidenceRow = (
    Option<i64>,
    Option<i64>,
    String,
    String,
    String,
    String,
    String,
    String,
);

pub fn file(db: &mut Connection, reporter: i64, input: Input, now: i64) -> Result<i64, Error> {
    let tx = db.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let enabled: bool = tx.query_row(
        "SELECT value FROM admin_settings WHERE key='reports_enabled'",
        [],
        |r| r.get(0),
    )?;
    if !enabled {
        return Err(Error::Disabled);
    }
    if super::moderation::state(&tx, reporter)
        .map_err(|_| Error::Unavailable)?
        .banned(now)
    {
        return Err(Error::Forbidden);
    }
    let (context, result, emoji, reason, comment, target) = match input {
        Input::Match {
            result_id,
            reason,
            comment,
        } => ("match", Some(result_id), None, reason, comment, None),
        Input::Emoji {
            result_id,
            emoji,
            reason,
            comment,
        } => ("emoji", Some(result_id), Some(emoji), reason, comment, None),
        Input::Profile {
            target,
            reason,
            comment,
        } => ("profile", None, None, reason, comment, Some(target)),
    };
    if ![
        "cheating",
        "abusive_chat",
        "offensive_name",
        "offensive_emoji",
        "harassment",
        "griefing",
        "impersonation",
        "other",
    ]
    .contains(&reason.as_str())
        || comment.as_deref().is_some_and(|v| {
            v.chars().count() > 500
                || !v
                    .chars()
                    .all(|c| (' '..='~').contains(&c) || ('\u{a0}'..='\u{ff}').contains(&c))
        })
    {
        return Err(Error::Invalid);
    }
    let result = result
        .map(|s| hex16(&s).ok_or(Error::Invalid))
        .transpose()?;
    let (target, evidence, emoji_id) = if let Some(result) = result {
        let row:Option<EvidenceRow>=tx.query_row("SELECT a_account,b_account,a_name,b_name,mode,outcome_json,loadouts_json,transcript_json FROM match_social_records WHERE result_id=?1 AND expires_unix>?2",params![result.as_slice(),now],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?,r.get(5)?,r.get(6)?,r.get(7)?))).optional()?;
        let (a, b, an, bn, mode, outcome, loadouts, transcript) = row.ok_or(Error::NotEligible)?;
        let (target, other) = if a == Some(reporter) {
            (b, 1)
        } else if b == Some(reporter) {
            (a, 0)
        } else {
            return Err(Error::NotEligible);
        };
        let target = target.ok_or(Error::NotEligible)?;
        let mut emoji_id: Option<i64> = None;
        if let Some(public) = emoji {
            let public = hex16(&public).ok_or(Error::Invalid)?;
            let equipped: [[tbg_core::protocol::EmojiRef; 3]; 2] =
                serde_json::from_str(&loadouts).map_err(|_| Error::Unavailable)?;
            if !equipped[other]
                .iter()
                .any(|r| matches!(r,tbg_core::protocol::EmojiRef::Custom{id,..}if *id==public))
            {
                return Err(Error::NotEligible);
            }
            emoji_id = tx
                .query_row(
                    "SELECT id FROM emoji_assets WHERE public_id=?1 AND owner_account_id=?2",
                    params![public.as_slice(), target],
                    |r| r.get(0),
                )
                .optional()?;
            if emoji_id.is_none() {
                return Err(Error::NotEligible);
            }
        }
        (
            target,
            json!({"names":[an,bn],"mode":mode,"outcome":serde_json::from_str::<Value>(&outcome).map_err(|_|Error::Unavailable)?,"loadouts":serde_json::from_str::<Value>(&loadouts).map_err(|_|Error::Unavailable)?,"transcript":serde_json::from_str::<Value>(&transcript).map_err(|_|Error::Unavailable)?}),
            emoji_id,
        )
    } else {
        let name = target.ok_or(Error::Invalid)?;
        let id: Option<i64> = tx
            .query_row(
                "SELECT id FROM accounts WHERE name_lower=?1",
                [name.to_ascii_lowercase()],
                |r| r.get(0),
            )
            .optional()?;
        let id = id.ok_or(Error::NotEligible)?;
        if super::moderation::state(&tx, id)
            .map_err(|_| Error::Unavailable)?
            .banned(now)
        {
            return Err(Error::NotEligible);
        }
        (id, json!({"name":name}), None)
    };
    if target == reporter {
        return Err(Error::NotEligible);
    }
    let duplicate:bool=tx.query_row("SELECT EXISTS(SELECT 1 FROM reports WHERE reporter_id=?1 AND target_id=?2 AND result_id IS ?3 AND status='open')",params![reporter,target,result.as_ref().map(|v|v.as_slice())],|r|r.get(0))?;
    if duplicate {
        return Err(Error::Duplicate);
    }
    let count: i64 = tx.query_row(
        "SELECT COUNT(*) FROM reports WHERE reporter_id=?1 AND created_unix>?2",
        params![reporter, now - 86400],
        |r| r.get(0),
    )?;
    if count >= 10 {
        return Err(Error::Limited);
    }
    tx.execute("INSERT INTO reports(created_unix,reporter_id,target_id,reason,context,result_id,emoji_id,comment,evidence_json) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9)",params![now,reporter,target,reason,context,result.as_ref().map(|v|v.as_slice()),emoji_id,comment,evidence.to_string()])?;
    let id = tx.last_insert_rowid();
    tx.commit()?;
    crate::metrics::p75::add(&crate::metrics::p75::REPORTS);
    Ok(id)
}
