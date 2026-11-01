//! `tbg-release-admin`: operator control of the P66 release directory on
//! the host (the database file is the authority; run it as the server's
//! user). Every change is one audited IMMEDIATE transaction; none stops a
//! process or touches a running match.
//!
//! ```text
//! tbg-release-admin --db PATH status
//! tbg-release-admin --db PATH canary   SLOT PCT  [--expect V] [--actor NAME]
//! tbg-release-admin --db PATH promote  SLOT      [--expect V] [--actor NAME]
//! tbg-release-admin --db PATH hold     SLOT      [--expect V]   (stop a canary)
//! tbg-release-admin --db PATH drain    SLOT      [--expect V]
//! tbg-release-admin --db PATH rollback SLOT      [--expect V]   (new admissions back to SLOT)
//! tbg-release-admin --db PATH ranked   SLOT      [--force]
//! tbg-release-admin --db PATH retire-check SLOT  (exit 0 only when safe to stop)
//! tbg-release-admin --db PATH retire   SLOT      (after the process stopped)
//! tbg-release-admin --db PATH audit    [N]
//! ```

use std::process::ExitCode;

use rusqlite::Connection;
use tbg_server::store::release::{now_unix, read_directory};
use tbg_server::store::release_admin::{self as admin, retire_blockers, AdminError};

fn value(args: &[String], name: &str) -> Option<String> {
    args.iter()
        .position(|a| a == name)
        .and_then(|i| args.get(i + 1))
        .cloned()
}

fn status(conn: &Connection) -> Result<(), String> {
    let now = now_unix();
    let rows = read_directory(conn).map_err(|e| e.to_string())?;
    println!("slot state     fresh auth canary running pending generation / contract");
    for r in rows {
        println!(
            "{:<4} {:<9} {:<5} {:<4} {:>5}% {:>7} {:>7} {} / {} (v{})",
            r.slot,
            r.state,
            r.fresh(now),
            r.ranked_authority,
            r.canary_pct,
            r.matches_running,
            r.results_pending,
            r.generation,
            r.contract,
            r.version
        );
    }
    Ok(())
}

fn audit(conn: &Connection, n: i64) -> Result<(), String> {
    let mut stmt = conn
        .prepare("SELECT at_unix, actor, action, slot, detail FROM release_audit ORDER BY id DESC LIMIT ?1")
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([n], |r| {
            Ok(format!(
                "{} {} {} {} {}",
                r.get::<_, i64>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, String>(3)?,
                r.get::<_, String>(4)?
            ))
        })
        .map_err(|e| e.to_string())?;
    for line in rows {
        println!("{}", line.map_err(|e| e.to_string())?);
    }
    Ok(())
}

/// Exit 0 only when the slot may be stopped (P66 §6.2 step 5).
fn retire_check(conn: &Connection, slot: &str) -> Result<ExitCode, String> {
    let rows = read_directory(conn).map_err(|e| e.to_string())?;
    let row = rows.iter().find(|r| r.slot == slot).ok_or("unknown slot")?;
    let why = retire_blockers(row);
    if why.is_empty() {
        println!("{slot}: safe to stop (drained, nothing running or pending)");
        Ok(ExitCode::SUCCESS)
    } else {
        println!("{slot}: not yet — {}", why.join("; "));
        Ok(ExitCode::from(3))
    }
}

fn change(conn: &mut Connection, cmd: &str, args: &[String]) -> Result<(), AdminError> {
    let slot = args.get(1).cloned().unwrap_or_default();
    let actor = value(args, "--actor").unwrap_or_else(|| "operator".into());
    let expect = value(args, "--expect").and_then(|v| v.parse().ok());
    match cmd {
        "canary" => {
            let pct = args
                .get(2)
                .and_then(|p| p.parse().ok())
                .ok_or(AdminError::BadPercent)?;
            admin::canary(conn, &slot, pct, &actor, expect)
        }
        "promote" => admin::promote(conn, &slot, &actor, expect),
        "hold" => admin::hold(conn, &slot, &actor, expect),
        "drain" => admin::drain(conn, &slot, &actor, expect),
        "rollback" => admin::rollback(conn, &slot, &actor, expect),
        "ranked" => {
            admin::ranked_authority(conn, &slot, args.iter().any(|a| a == "--force"), &actor)
        }
        "retire" => admin::mark_retired(conn, &slot, &actor, expect),
        _ => Err(AdminError::WrongState(format!("unknown command {cmd}"))),
    }
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let Some(db) = value(&args, "--db") else {
        eprintln!("usage: tbg-release-admin --db PATH <status|canary|promote|hold|drain|rollback|ranked|retire-check|retire|audit> …");
        return ExitCode::from(2);
    };
    let rest: Vec<String> = args
        .iter()
        .enumerate()
        .filter(|(i, a)| *a != "--db" && (*i == 0 || args[*i - 1] != "--db"))
        .map(|(_, a)| a.clone())
        .collect();
    let mut conn = match Connection::open(&db) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("tbg-release-admin: {e}");
            return ExitCode::from(1);
        }
    };
    let _ = conn.busy_timeout(std::time::Duration::from_secs(5));
    let cmd = rest.first().cloned().unwrap_or_default();
    let result = match cmd.as_str() {
        "status" => status(&conn).map(|_| ExitCode::SUCCESS),
        "audit" => audit(
            &conn,
            rest.get(1).and_then(|n| n.parse().ok()).unwrap_or(20),
        )
        .map(|_| ExitCode::SUCCESS),
        "retire-check" => retire_check(&conn, rest.get(1).map(String::as_str).unwrap_or("")),
        _ => change(&mut conn, &cmd, &rest)
            .map(|_| ExitCode::SUCCESS)
            .map_err(|e| format!("{e:?}")),
    };
    match result {
        Ok(code) => {
            if !matches!(cmd.as_str(), "status" | "audit" | "retire-check") {
                let _ = status(&conn);
            }
            code
        }
        Err(e) => {
            eprintln!("tbg-release-admin: refused: {e}");
            ExitCode::from(1)
        }
    }
}
