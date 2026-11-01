//! Post-update notes receipts (P66 §3.2, BR-UP04, U10): after a healthy
//! activation the launcher records which build to present; the game shows
//! that build's bundled notes until the player acknowledges, even across a
//! crash or close (at-least-once). A rollback records `recovered`, never an
//! update success.

use serde::{Deserialize, Serialize};

use crate::fsx;
use crate::layout::{valid_build_id, Layout};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    /// `build` was installed and proved healthy.
    Updated,
    /// The update to `build` failed; `from` is running again.
    Recovered,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Receipt {
    pub schema: u32,
    pub kind: Kind,
    /// The build whose notes (updated) or failure (recovered) to show.
    pub build: String,
    pub from: String,
    /// Skipped builds between `from` and `build` (highlights only).
    pub history: Vec<String>,
    pub acknowledged: bool,
}

fn path(layout: &Layout) -> std::path::PathBuf {
    layout.receipts().join("pending.json")
}

pub fn record(
    layout: &Layout,
    kind: Kind,
    build: &str,
    from: &str,
    history: Vec<String>,
) -> std::io::Result<()> {
    fsx::private_dir(&layout.receipts())?;
    fsx::write_json(
        &path(layout),
        &Receipt {
            schema: crate::descriptor::LOCAL_STATE,
            kind,
            build: build.to_string(),
            from: from.to_string(),
            history,
            acknowledged: false,
        },
    )
}

/// The receipt still waiting for the player, if any.
pub fn pending(layout: &Layout) -> Option<Receipt> {
    let r: Receipt = fsx::read_json(&path(layout), 64 * 1024).ok()?;
    let valid = r.schema == crate::descriptor::LOCAL_STATE && valid_build_id(&r.build);
    (valid && !r.acknowledged).then_some(r)
}

/// The player closed the notes for `build`; older receipts are ignored.
pub fn acknowledge(layout: &Layout, build: &str) -> std::io::Result<bool> {
    let Some(mut r) = pending(layout) else {
        return Ok(false);
    };
    if r.build != build {
        return Ok(false);
    }
    r.acknowledged = true;
    fsx::write_json(&path(layout), &r)?;
    Ok(true)
}
