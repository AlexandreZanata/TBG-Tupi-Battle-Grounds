//! Selection switches and recovery (P66 §5.1): `current` and `previous`
//! are tiny atomic files naming release directories, written in a fixed
//! order under the journal so any crash point is recoverable. A release
//! directory is complete before it is ever named; partial data is never
//! executable. Cleanup only removes known release directories that are
//! neither selected, staged nor pinned.

use std::io;

use crate::fsx;
use crate::journal::{self, Txn};
use crate::layout::{valid_build_id, Layout};

/// Make `to` current, remembering the old current as `previous`.
pub fn switch(layout: &Layout, from: &str, to: &str) -> io::Result<()> {
    fsx::private_dir(&layout.state())?;
    if !from.is_empty() && from != to {
        fsx::write_atomic(&layout.previous(), from.as_bytes())?;
    }
    fsx::write_atomic(&layout.current(), to.as_bytes())
}

/// Put `from` back as current (the candidate failed before healthy).
pub fn restore(layout: &Layout, from: &str) -> io::Result<()> {
    fsx::write_atomic(&layout.current(), from.as_bytes())
}

/// A release directory holds a runnable game.
pub fn complete(layout: &Layout, build: &str) -> bool {
    valid_build_id(build)
        && layout.game_binary(build).is_file()
        && layout.release(build).join("build-info.json").is_file()
}

/// What reconciliation decided at launcher start.
#[derive(Debug, PartialEq, Eq)]
pub enum Recovery {
    Nothing,
    /// A staged release still waits for an idle apply.
    StillStaged(String),
    /// The interrupted candidate never proved healthy: back to `from`.
    RolledBack {
        from: String,
        to: String,
    },
    /// The healthy candidate's receipt is now durable, including after a crash.
    Committed {
        from: String,
        to: String,
    },
    /// The switch had not started: the old selection stands.
    Unchanged,
}

/// Reconcile the journal with the selection files (U05).
pub fn reconcile(layout: &Layout) -> io::Result<Recovery> {
    let Some(txn) = journal::load_checked(layout)? else {
        return Ok(Recovery::Nothing);
    };
    match txn {
        Txn::Healthy { from, to, .. } => {
            if !complete(layout, &to) {
                return Err(io::Error::other("healthy release is incomplete"));
            }
            restore(layout, &to)?;
            finish_healthy(layout, &from, &to)?;
            Ok(Recovery::Committed { from, to })
        }
        Txn::Staged { to, .. } => {
            if complete(layout, &to) {
                Ok(Recovery::StillStaged(to))
            } else {
                journal::store(layout, None)?;
                Ok(Recovery::Unchanged)
            }
        }
        Txn::Activating { from, to, .. } | Txn::Launching { from, to, .. } => {
            // No healthy acknowledgement was committed for `to` (that
            // clears the journal), so it is not proven: use `from`.
            if complete(layout, &from) {
                restore(layout, &from)?;
                journal::store(layout, None)?;
                Ok(Recovery::RolledBack { from, to })
            } else {
                // Nothing older to return to: keep the candidate.
                journal::store(layout, None)?;
                Ok(Recovery::Unchanged)
            }
        }
    }
}

/// The receipt must survive before the last recovery marker disappears (U10).
pub fn finish_healthy(layout: &Layout, from: &str, to: &str) -> io::Result<()> {
    crate::receipt::record(layout, crate::receipt::Kind::Updated, to, from, Vec::new())?;
    journal::store(layout, None)
}

/// Remove release directories that are not current, previous, staged or
/// in `pinned`. Unknown names and files are left alone.
pub fn cleanup(layout: &Layout, pinned: &[String]) -> io::Result<Vec<String>> {
    let mut keep: Vec<String> = pinned.to_vec();
    keep.extend(layout.current_build());
    keep.extend(layout.previous_build());
    if let Some(Txn::Staged { to, .. }) = journal::load(layout) {
        keep.push(to);
    }
    let mut removed = Vec::new();
    let Ok(entries) = std::fs::read_dir(layout.releases()) else {
        return Ok(removed);
    };
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        let is_dir = entry.file_type().map(|t| t.is_dir()).unwrap_or(false);
        if !is_dir || !valid_build_id(&name) || keep.contains(&name) {
            continue;
        }
        std::fs::remove_dir_all(entry.path())?;
        removed.push(name);
    }
    Ok(removed)
}
