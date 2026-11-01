//! The stable launcher (P66 §5.1, D-91): one instance per user, journal
//! reconciliation before anything runs, the selected release started as a
//! child with its health file, health-gated commit or rollback of a new
//! candidate, and an idle "apply" request (exit code 75) that activates a
//! staged release and relaunches. The game process is never killed: a slow
//! candidate is waited for, only an exit before its healthy report rolls
//! back.

use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::{Child, Command};
use std::time::{Duration, Instant};

use crate::activate::{self, Recovery};
use crate::journal::{self, Txn};
use crate::layout::Layout;
use crate::limits::READINESS_SECS;
use crate::{fsx, receipt};

/// The game asks the launcher to apply a staged update.
pub const EXIT_APPLY: i32 = 75;

/// How the last update attempt ended, for the game's admission report.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    None,
    Updated,
    Recovered,
}

impl Outcome {
    pub fn code(self) -> &'static str {
        match self {
            Outcome::None => "none",
            Outcome::Updated => "updated",
            Outcome::Recovered => "recovered",
        }
    }
}

/// A random-enough nonce for one launch's health report.
fn nonce() -> String {
    let t = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_nanos());
    format!("{:x}{:x}", t, std::process::id())
}

fn health_file(layout: &Layout, nonce: &str) -> PathBuf {
    layout.run().join(format!("health-{nonce}.json"))
}

/// `{"nonce": .., "build": ..}` written by the game once its menu is ready.
fn healthy(path: &Path, nonce: &str, build: &str) -> bool {
    let Ok(v) = fsx::read_json::<serde_json::Value>(path, 4096) else {
        return false;
    };
    v.get("nonce").and_then(|n| n.as_str()) == Some(nonce)
        && v.get("build").and_then(|b| b.as_str()) == Some(build)
}

fn spawn(
    layout: &Layout,
    build: &str,
    args: &[OsString],
    outcome: Outcome,
    nonce: &str,
) -> std::io::Result<Child> {
    let helper = std::env::current_exe()?;
    Command::new(layout.game_binary(build))
        .args(args)
        .current_dir(layout.release(build))
        .env("TBG_UPDATE_ROOT", &layout.root)
        .env("TBG_UPDATE_HELPER", helper)
        .env("TBG_INSTALLED_BUILD", build)
        .env("TBG_UPDATE_OUTCOME", outcome.code())
        .env("TBG_HEALTH_FILE", health_file(layout, nonce))
        .env("TBG_HEALTH_NONCE", nonce)
        .spawn()
}

/// How a candidate's launch went.
enum Watch {
    /// It reported healthy and is still running.
    Proven,
    /// It exited before reporting healthy.
    Exited,
}

/// Wait for a candidate's healthy report or its exit, whichever is first.
fn watch_candidate(
    child: &mut Child,
    path: &Path,
    nonce: &str,
    build: &str,
) -> std::io::Result<Watch> {
    let started = Instant::now();
    let mut warned = false;
    loop {
        if healthy(path, nonce, build) {
            return Ok(Watch::Proven);
        }
        if child.try_wait()?.is_some() {
            // A report written just before a clean exit still counts.
            return Ok(if healthy(path, nonce, build) {
                Watch::Proven
            } else {
                Watch::Exited
            });
        }
        if !warned && started.elapsed() > Duration::from_secs(READINESS_SECS) {
            // Diagnostic only: a slow boot is not a reason to kill.
            eprintln!("tbg-launcher: candidate {build} has not reported healthy after {READINESS_SECS} s; still waiting");
            warned = true;
        }
        std::thread::sleep(Duration::from_millis(200));
    }
}

/// Run the managed game until it exits for good.
pub fn run(layout: &Layout, args: &[OsString]) -> i32 {
    let _ = fsx::private_dir(&layout.run());
    let Ok(lock) = std::fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .write(true)
        .open(layout.lock())
    else {
        eprintln!("tbg-launcher: cannot open the install lock");
        return 1;
    };
    if lock.try_lock().is_err() {
        eprintln!("tbg-launcher: the game is already running");
        return 0;
    }
    let mut outcome = Outcome::None;
    match activate::reconcile(layout) {
        Ok(Recovery::RolledBack { from, to }) => {
            let _ = receipt::record(layout, receipt::Kind::Recovered, &to, &from, Vec::new());
            outcome = Outcome::Recovered;
        }
        Ok(Recovery::Committed { .. }) => outcome = Outcome::Updated,
        Ok(_) => {}
        Err(e) => {
            eprintln!("tbg-launcher: journal recovery failed: {e}");
            return 1;
        }
    }
    loop {
        let Some(build) = layout
            .current_build()
            .filter(|b| activate::complete(layout, b))
        else {
            eprintln!(
                "tbg-launcher: no complete release is installed; reinstall with the setup package"
            );
            return 1;
        };
        match run_once(layout, &build, args, outcome) {
            Step::Exit(code) => return code,
            Step::Relaunch(next) => outcome = next,
        }
    }
}

enum Step {
    Exit(i32),
    Relaunch(Outcome),
}

fn run_once(layout: &Layout, build: &str, args: &[OsString], outcome: Outcome) -> Step {
    let candidate = match journal::load(layout) {
        Some(Txn::Launching { from, to, .. }) if to == build => Some(from),
        _ => None,
    };
    let n = nonce();
    let health = health_file(layout, &n);
    let mut child = match spawn(layout, build, args, outcome, &n) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("tbg-launcher: cannot start {build}: {e}");
            return match candidate {
                Some(from) => roll_back(layout, &from, build),
                None => Step::Exit(1),
            };
        }
    };
    if let Some(from) = &candidate {
        match watch_candidate(&mut child, &health, &n, build) {
            // Commit as soon as it is healthy: a later crash during play is
            // an ordinary crash, not a reason to undo a proven update.
            Ok(Watch::Proven) => commit(layout, from, build),
            Ok(Watch::Exited) => {
                let _ = std::fs::remove_file(&health);
                return roll_back(layout, from, build);
            }
            Err(e) => {
                eprintln!("tbg-launcher: lost the candidate process: {e}");
                return Step::Exit(1);
            }
        }
    }
    let status = child.wait();
    let _ = std::fs::remove_file(&health);
    let Ok(status) = status else {
        return Step::Exit(1);
    };
    if status.code() == Some(EXIT_APPLY) {
        return apply(layout, build);
    }
    Step::Exit(status.code().unwrap_or(1))
}

/// The candidate exited before reporting healthy: back to `from`.
fn roll_back(layout: &Layout, from: &str, to: &str) -> Step {
    if !crate::policy::permits_fallback(layout, from) {
        eprintln!("tbg-launcher: fallback is not permitted; use a trusted repair bootstrap");
        return Step::Exit(1);
    }
    if activate::restore(layout, from).is_err() || journal::store(layout, None).is_err() {
        return Step::Exit(1);
    }
    let _ = receipt::record(layout, receipt::Kind::Recovered, to, from, Vec::new());
    Step::Relaunch(Outcome::Recovered)
}

/// Commit health before notes, then clear the journal. Retention needs owner policy.
fn commit(layout: &Layout, from: &str, to: &str) {
    let Some(Txn::Launching { id, .. }) = journal::load(layout) else {
        return;
    };
    let healthy = Txn::Healthy {
        id,
        from: from.into(),
        to: to.into(),
    };
    if journal::store(layout, Some(&healthy))
        .and_then(|_| activate::finish_healthy(layout, from, to))
        .is_err()
    {
        eprintln!("tbg-launcher: healthy receipt commit pending; retained recovery journal");
    }
}

/// Exit 75: activate the staged release (if any) and relaunch it.
fn apply(layout: &Layout, running: &str) -> Step {
    let Some(Txn::Staged { id, to }) = journal::load(layout) else {
        return Step::Relaunch(Outcome::None);
    };
    if !activate::complete(layout, &to) || to == running {
        let _ = journal::store(layout, None);
        return Step::Relaunch(Outcome::None);
    }
    let from = running.to_string();
    let activating = Txn::Activating {
        id: id.clone(),
        from: from.clone(),
        to: to.clone(),
    };
    if journal::store(layout, Some(&activating)).is_err()
        || activate::switch(layout, &from, &to).is_err()
    {
        let _ = activate::restore(layout, &from);
        let _ = journal::store(layout, None);
        return Step::Relaunch(Outcome::Recovered);
    }
    let launching = Txn::Launching { id, from, to };
    if journal::store(layout, Some(&launching)).is_err() {
        return Step::Exit(1);
    }
    Step::Relaunch(Outcome::Updated)
}
