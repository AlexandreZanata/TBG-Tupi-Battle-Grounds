//! The update transaction journal (P66 §5.1, U05): one durable document
//! saying how far the current activation got. Every launcher start
//! reconciles it before running anything, so an interruption at any point
//! resolves to a complete known-good release.

use serde::{Deserialize, Serialize};

use crate::fsx;
use crate::layout::Layout;

/// Where an update transaction stands.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum Txn {
    /// A verified, complete release waits in `releases/<to>`.
    Staged { id: String, to: String },
    /// `current` is being switched from `from` to `to`.
    Activating {
        id: String,
        from: String,
        to: String,
    },
    /// `to` was launched and must report healthy.
    Launching {
        id: String,
        from: String,
        to: String,
    },
    /// Boot health is durable; finish the notes receipt before clearing the journal.
    Healthy {
        id: String,
        from: String,
        to: String,
    },
}

impl Txn {
    pub fn id(&self) -> &str {
        match self {
            Txn::Staged { id, .. }
            | Txn::Activating { id, .. }
            | Txn::Launching { id, .. }
            | Txn::Healthy { id, .. } => id,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Doc {
    schema: u32,
    txn: Option<Txn>,
}

pub fn load(layout: &Layout) -> Option<Txn> {
    load_checked(layout).ok().flatten()
}

pub fn load_checked(layout: &Layout) -> std::io::Result<Option<Txn>> {
    if !layout.journal().try_exists()? {
        return Ok(None);
    }
    let doc: Doc = fsx::read_json(&layout.journal(), 64 * 1024)?;
    if doc.schema != crate::descriptor::LOCAL_STATE {
        return Err(std::io::Error::other("unsupported update journal schema"));
    }
    Ok(doc.txn)
}

pub fn store(layout: &Layout, txn: Option<&Txn>) -> std::io::Result<()> {
    fsx::private_dir(&layout.state())?;
    fsx::write_json(
        &layout.journal(),
        &Doc {
            schema: crate::descriptor::LOCAL_STATE,
            txn: txn.cloned(),
        },
    )
}

/// A new transaction id: time plus process id, unique per machine.
pub fn new_id() -> String {
    let t = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_nanos());
    format!("{t:x}-{:x}", std::process::id())
}
