//! `tbg-ranked-admin`: offline ranked review and correction (P60 §7).
//!
//!   tbg-ranked-admin list-review --db COPY
//!   tbg-ranked-admin inspect --match HEX --db COPY
//!   tbg-ranked-admin plan --action invalidate|clear|review|restore --match HEX \
//!       --reason TEXT --actor NAME --db COPY --out PLAN
//!   tbg-ranked-admin apply --plan PLAN --db DB
//!
//! Read-only commands open the file read-only (use a backup). `apply` is for
//! the live file of a **stopped** server: it writes a fresh backup next to
//! the database first, then checks the plan's base revision and records in
//! one transaction. `--actor` names an operator; it is not authentication.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use rusqlite::{Connection, OpenFlags};

use tbg_server::ranked::{hex, parse_key};
use tbg_server::store::integrity_admin::{
    apply_plan, inspect, list_review, plan, Action, AdminError, Applied, Plan,
};

const USAGE: &str = "usage: tbg-ranked-admin list-review|inspect|plan|apply [--flag value]...";

fn flags(mut it: impl Iterator<Item = String>) -> Result<HashMap<String, String>, String> {
    let mut out = HashMap::new();
    while let Some(flag) = it.next() {
        let name = flag
            .strip_prefix("--")
            .ok_or(format!("unexpected argument {flag}"))?;
        let value = it.next().ok_or(format!("{flag} needs a value"))?;
        if out.insert(name.to_string(), value).is_some() {
            return Err(format!("{flag} given twice"));
        }
    }
    Ok(out)
}

fn need<'a>(f: &'a HashMap<String, String>, name: &str) -> Result<&'a str, String> {
    f.get(name)
        .map(String::as_str)
        .ok_or(format!("--{name} is required"))
}

fn read_only(path: &str) -> Result<Connection, String> {
    Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .map_err(|e| format!("{path}: {e}"))
}

fn key_of(f: &HashMap<String, String>) -> Result<[u8; 16], String> {
    parse_key(need(f, "match")?).ok_or("--match must be 32 hex characters".into())
}

fn admin(e: AdminError) -> String {
    match e {
        AdminError::StalePlan => {
            "the database changed since this plan (revision or records); plan again".into()
        }
        other => format!("{other:?}"),
    }
}

fn run(command: &str, f: HashMap<String, String>) -> Result<(), String> {
    match command {
        "list-review" => {
            let conn = read_only(need(&f, "db")?)?;
            for r in list_review(&conn).map_err(|e| e.to_string())? {
                let [a, b] = &r.names;
                println!("{}\t{}\t{}\t{a} vs {b}", r.match_id, r.key, r.ended_unix);
            }
            Ok(())
        }
        "inspect" => {
            let conn = read_only(need(&f, "db")?)?;
            let v = inspect(&conn, &key_of(&f)?).map_err(admin)?;
            println!("{}", serde_json::to_string_pretty(&v).unwrap_or_default());
            Ok(())
        }
        "plan" => {
            let conn = read_only(need(&f, "db")?)?;
            let action = Action::parse(need(&f, "action")?).ok_or("unknown --action")?;
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_or(0, |d| d.as_secs() as i64);
            let op = hex(&random16()?);
            let p = plan(
                &conn,
                &key_of(&f)?,
                action,
                need(&f, "reason")?,
                need(&f, "actor")?,
                op,
                now,
            )
            .map_err(admin)?;
            for (b, a) in p.before.iter().zip(&p.after) {
                println!(
                    "account {}: rating {} -> {}, peak {} -> {}, W/L/D {}/{}/{} -> {}/{}/{}",
                    b.id,
                    b.rating,
                    a.rating,
                    b.peak,
                    a.peak,
                    b.wins,
                    b.losses,
                    b.draws,
                    a.wins,
                    a.losses,
                    a.draws
                );
            }
            let out = need(&f, "out")?;
            std::fs::write(out, p.to_json().to_string()).map_err(|e| format!("{out}: {e}"))
        }
        "apply" => {
            let path = need(&f, "db")?;
            let text = std::fs::read_to_string(need(&f, "plan")?).map_err(|e| e.to_string())?;
            let p = serde_json::from_str(&text)
                .ok()
                .and_then(|v| Plan::from_json(&v))
                .ok_or("unreadable plan")?;
            let mut conn = Connection::open(path).map_err(|e| format!("{path}: {e}"))?;
            backup(&conn, Path::new(path), &p)?;
            match apply_plan(&mut conn, &p).map_err(admin)? {
                Applied::Committed => println!("applied {}", p.operation_key),
                Applied::AlreadyApplied => {
                    println!("already applied {}: no change", p.operation_key)
                }
            }
            Ok(())
        }
        _ => Err(USAGE.into()),
    }
}

/// A fresh, never-overwritten copy before any write (P60 §7).
fn backup(conn: &Connection, db: &Path, p: &Plan) -> Result<(), String> {
    let target = PathBuf::from(format!("{}.pre-{}.sqlite3", db.display(), p.operation_key));
    if target.exists() {
        return Ok(()); // A previous attempt of this operation already copied.
    }
    conn.execute("VACUUM INTO ?1", [target.to_string_lossy().as_ref()])
        .map(|_| ())
        .map_err(|e| format!("backup {} failed: {e}", target.display()))
}

fn random16() -> Result<[u8; 16], String> {
    let mut b = [0u8; 16];
    getrandom::fill(&mut b).map_err(|e| e.to_string())?;
    Ok(b)
}

fn main() {
    let mut args = std::env::args().skip(1);
    let command = args.next().unwrap_or_default();
    if let Err(e) = flags(args).and_then(|f| run(&command, f)) {
        eprintln!("tbg-ranked-admin: {e}");
        std::process::exit(2);
    }
}
