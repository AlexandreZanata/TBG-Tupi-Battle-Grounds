//! P66 native updates (D-91): install layout, durable journal, release
//! contracts, TUF-verified downloads, safe extraction, activation with
//! health-gated rollback, post-update notes receipts and the release tool.
//! Lives outside the game binary: the Bevy client only shows state and
//! asks the launcher's helper to act.

pub mod activate;
pub mod descriptor;
pub mod extract;
pub mod fsx;
pub mod helper;
pub mod install;
pub mod journal;
pub mod launcher;
pub mod layout;
pub mod limits;
pub mod notes;
pub mod publish;
pub mod receipt;
pub mod transport;
pub mod trust;
