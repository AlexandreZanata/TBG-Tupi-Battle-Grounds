//! Durable pending ranked results (P60 §7): one JSON file per completed
//! match in a private `ranked-pending/` directory beside the database,
//! written through a temp file, `sync_all`, atomic rename and a directory
//! sync. A file is deleted only after its settlement committed. Blocking
//! I/O: callers run these through `spawn_blocking`.

use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use serde_json::{json, Value};

use tbg_core::{ArcanaPreset, EndReason, Outcome, Seat};

use super::{hex, parse_key, EndDetail, IntegritySeat, JournalEntry, RankedFinal};

/// Per-file and whole-directory caps (P60 §7).
pub const FILE_CAP: usize = 32 * 1024;
pub const DIR_CAP: u64 = 32 * 1024 * 1024;
const FORMAT: u64 = 1;

pub fn dir_for(db: &Path) -> PathBuf {
    db.parent()
        .unwrap_or_else(|| Path::new("."))
        .join("ranked-pending")
}

fn file_name(entry: &JournalEntry) -> String {
    format!(
        "{:020}-{}.json",
        entry.completion_seq,
        hex(&entry.result.key)
    )
}

/// Persist `entry`; an error means it exists only in memory.
pub fn write(dir: &Path, entry: &JournalEntry) -> io::Result<()> {
    fs::create_dir_all(dir)?;
    let bytes = encode(entry).to_string().into_bytes();
    if bytes.len() > FILE_CAP {
        return Err(io::Error::other("ranked journal entry over 32 KiB"));
    }
    if used(dir)? + bytes.len() as u64 > DIR_CAP {
        return Err(io::Error::other("ranked journal directory over 32 MiB"));
    }
    let name = file_name(entry);
    let tmp = dir.join(format!(".tmp-{name}"));
    {
        let mut f = fs::File::create(&tmp)?;
        f.write_all(&bytes)?;
        f.sync_all()?;
    }
    fs::rename(&tmp, dir.join(&name))?;
    sync_dir(dir)
}

/// Forget a committed entry. A crash before this is safe: replay returns
/// the committed receipt for the same key.
pub fn remove(dir: &Path, entry: &JournalEntry) -> io::Result<()> {
    match fs::remove_file(dir.join(file_name(entry))) {
        Err(e) if e.kind() != io::ErrorKind::NotFound => Err(e),
        _ => sync_dir(dir),
    }
}

/// Keep a conflicting entry for the operator; it never settles.
pub fn quarantine(dir: &Path, entry: &JournalEntry) -> io::Result<()> {
    let name = file_name(entry);
    match fs::rename(dir.join(&name), dir.join(format!("{name}.quarantine"))) {
        Err(e) if e.kind() != io::ErrorKind::NotFound => Err(e),
        _ => sync_dir(dir),
    }
}

/// Every pending entry in completion order. Unreadable files are left in
/// place and reported, never deleted or guessed.
pub fn load_all(dir: &Path) -> io::Result<Vec<JournalEntry>> {
    let mut out = Vec::new();
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(out),
        Err(e) => return Err(e),
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().is_none_or(|e| e != "json") {
            continue;
        }
        let decoded = fs::read(&path)
            .ok()
            .and_then(|b| serde_json::from_slice::<Value>(&b).ok())
            .and_then(|v| decode(&v));
        match decoded {
            Some(e) => out.push(e),
            None => tracing::error!("unreadable ranked journal file {}", path.display()),
        }
    }
    out.sort_by_key(|e| e.completion_seq);
    Ok(out)
}

fn used(dir: &Path) -> io::Result<u64> {
    let mut total = 0;
    for entry in fs::read_dir(dir)?.flatten() {
        total += entry.metadata().map(|m| m.len()).unwrap_or(0);
    }
    Ok(total)
}

fn sync_dir(dir: &Path) -> io::Result<()> {
    // Directory fsync makes the rename durable on POSIX file systems.
    #[cfg(unix)]
    fs::File::open(dir)?.sync_all()?;
    #[cfg(not(unix))]
    let _ = dir;
    Ok(())
}

pub fn encode(e: &JournalEntry) -> Value {
    let r = &e.result;
    json!({
        "format": FORMAT,
        "seq": e.completion_seq,
        "ended_unix": e.ended_unix,
        "key": hex(&r.key),
        "accounts": r.accounts,
        "winner": r.outcome.winner.map(|s| s.index()),
        "reason": match r.outcome.reason {
            EndReason::StrongholdDestroyed => "stronghold",
            EndReason::Forfeit => "forfeit",
        },
        "end_detail": r.end_detail.as_str(),
        "combat_ticks": r.combat_ticks,
        "wall_ms": r.wall_ms,
        "reached_running": r.reached_running,
        "ruleset_version": r.ruleset_version,
        "ruleset_source_hash": format!("{:016x}", r.ruleset_source_hash),
        "seats": r.seats.each_ref().map(|s| json!([
            s.purchases, s.evolutions, s.fortifications, s.augments_applied,
            s.reconnects, s.surrender_tick, s.final_hp, s.final_max_hp,
        ])),
        "factions": r.factions,
        "arcanas": r.arcanas.each_ref().map(|a| a.ranks),
        "stats": r.stats,
    })
}

pub fn decode(v: &Value) -> Option<JournalEntry> {
    if v["format"].as_u64()? != FORMAT {
        return None;
    }
    let seat = |i: usize| -> Option<IntegritySeat> {
        let s = v["seats"].get(i)?;
        let n = |j: usize| -> Option<u32> { u32::try_from(s.get(j)?.as_u64()?).ok() };
        let hp = |j: usize| -> Option<i32> { i32::try_from(s.get(j)?.as_i64()?).ok() };
        Some(IntegritySeat {
            purchases: n(0)?,
            evolutions: n(1)?,
            fortifications: n(2)?,
            augments_applied: n(3)?,
            reconnects: n(4)?,
            surrender_tick: match s.get(5)? {
                Value::Null => None,
                t => Some(u32::try_from(t.as_u64()?).ok()?),
            },
            final_hp: hp(6)?,
            final_max_hp: hp(7)?,
        })
    };
    let quad = |x: &Value| -> Option<[u32; 4]> {
        let mut out = [0u32; 4];
        for (i, slot) in out.iter_mut().enumerate() {
            *slot = u32::try_from(x.get(i)?.as_u64()?).ok()?;
        }
        Some(out)
    };
    let preset = |x: &Value| -> Option<ArcanaPreset> {
        let mut ranks = [0u8; tbg_core::arcana::NODES];
        for (i, slot) in ranks.iter_mut().enumerate() {
            *slot = u8::try_from(x.get(i)?.as_u64()?).ok()?;
        }
        Some(ArcanaPreset { ranks })
    };
    let winner = match &v["winner"] {
        Value::Null => None,
        w => Some(
            Seat::BOTH
                .get(usize::try_from(w.as_u64()?).ok()?)
                .copied()?,
        ),
    };
    let reason = match v["reason"].as_str()? {
        "stronghold" => EndReason::StrongholdDestroyed,
        "forfeit" => EndReason::Forfeit,
        _ => return None,
    };
    let result = RankedFinal {
        key: parse_key(v["key"].as_str()?)?,
        accounts: [v["accounts"][0].as_i64()?, v["accounts"][1].as_i64()?],
        outcome: Outcome { winner, reason },
        end_detail: EndDetail::parse(v["end_detail"].as_str()?)?,
        combat_ticks: u32::try_from(v["combat_ticks"].as_u64()?).ok()?,
        wall_ms: v["wall_ms"].as_u64()?,
        reached_running: v["reached_running"].as_bool()?,
        ruleset_version: u32::try_from(v["ruleset_version"].as_u64()?).ok()?,
        ruleset_source_hash: u64::from_str_radix(v["ruleset_source_hash"].as_str()?, 16).ok()?,
        seats: [seat(0)?, seat(1)?],
        factions: [quad(&v["factions"][0])?, quad(&v["factions"][1])?],
        arcanas: [preset(&v["arcanas"][0])?, preset(&v["arcanas"][1])?],
        stats: match &v["stats"] {
            Value::Null => None,
            s => Some(s.clone()),
        },
    };
    Some(JournalEntry {
        completion_seq: v["seq"].as_i64()?,
        ended_unix: v["ended_unix"].as_i64()?,
        result,
    })
}
