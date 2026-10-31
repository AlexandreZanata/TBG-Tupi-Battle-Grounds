//! `tbg-stats`: internal ranked statistics report (P61 E1, §9) from a
//! SQLite **backup**. Opens the file read-only, writes one Markdown file,
//! never touches production data or balance values.
//!
//!   tbg-stats report --db BACKUP.sqlite3 --patch 1 [--compare 0] --out report.md

use std::path::PathBuf;

use rusqlite::{Connection, OpenFlags};

use tbg_server::stats::aggregate::{build, Report};
use tbg_server::stats::render::{markdown, Header};
use tbg_server::stats::{eligible, Catalog, Coverage};
use tbg_server::store::stats::read_cohort;

struct Args {
    db: PathBuf,
    patch: u32,
    compare: Option<u32>,
    out: PathBuf,
}

fn parse(mut it: impl Iterator<Item = String>) -> Result<Args, String> {
    if it.next().as_deref() != Some("report") {
        return Err("usage: tbg-stats report --db FILE --patch N [--compare N] --out FILE".into());
    }
    let (mut db, mut patch, mut compare, mut out) = (None, None, None, None);
    while let Some(flag) = it.next() {
        let value = it.next().ok_or(format!("{flag} needs a value"))?;
        let number = |v: &str| {
            v.parse::<u32>()
                .map_err(|_| format!("{flag}: not a revision"))
        };
        match flag.as_str() {
            "--db" => db = Some(PathBuf::from(value)),
            "--patch" => patch = Some(number(&value)?),
            "--compare" => compare = Some(number(&value)?),
            "--out" => out = Some(PathBuf::from(value)),
            other => return Err(format!("unknown flag {other}")),
        }
    }
    Ok(Args {
        db: db.ok_or("--db is required")?,
        patch: patch.ok_or("--patch is required")?,
        compare,
        out: out.ok_or("--out is required")?,
    })
}

/// `(report, coverage, source hash)` for one revision; `Ok(None)` when the
/// revision has no stored catalog.
fn revision(conn: &Connection, version: u32) -> Result<Option<(Report, Coverage, String)>, String> {
    let (catalog_json, matches) =
        read_cohort(conn, version).map_err(|e| format!("revision {version}: {e}"))?;
    let Some(catalog_json) = catalog_json else {
        return Ok(None);
    };
    let catalog =
        Catalog::parse(&catalog_json).ok_or(format!("revision {version}: unreadable catalog"))?;
    let hash: String = conn
        .query_row(
            "SELECT source_hash FROM ranked_rulesets WHERE version = ?1",
            [version],
            |r| r.get(0),
        )
        .map_err(|e| e.to_string())?;
    let (obs, cov) = eligible(&matches, &catalog);
    Ok(Some((build(&obs, &catalog), cov, hash)))
}

fn run(args: Args) -> Result<(), String> {
    let conn = Connection::open_with_flags(&args.db, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .map_err(|e| format!("{}: {e}", args.db.display()))?;
    let (report, cov, hash) =
        revision(&conn, args.patch)?.ok_or(format!("revision {} is not registered", args.patch))?;
    let previous = match args.compare {
        Some(v) => Some((v, revision(&conn, v)?.map(|(r, ..)| r).unwrap_or_default())),
        None => None,
    };
    let text = markdown(
        &Header {
            revision: args.patch,
            source_hash: &hash,
            compare: previous.as_ref().map(|(v, r)| (*v, r)),
        },
        &cov,
        &report,
    );
    std::fs::write(&args.out, text).map_err(|e| format!("{}: {e}", args.out.display()))
}

fn main() {
    if let Err(e) = parse(std::env::args().skip(1)).and_then(run) {
        eprintln!("tbg-stats: {e}");
        std::process::exit(2);
    }
}
