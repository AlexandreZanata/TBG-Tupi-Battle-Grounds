//! The read-only live snapshot (P47, BR-LV06/BR-LV07): what `/api/live` and
//! `/api/online` answer. The lobby builds it at most once a second and
//! publishes it on a `watch` channel, so no HTTP request touches lobby state
//! or SQL (ARCHITECTURE §11), the same pattern as the leaderboard.

use tbg_core::protocol::{LiveId, MatchMode};
use tbg_core::{Arena, StrongholdKind};

/// What the match task last reported (every 20 ticks, `try_send`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LiveProgress {
    /// Stronghold HP per seat, in basis points of its maximum.
    pub hp_bp: [u16; 2],
    pub tiers: [u8; 2],
    pub elapsed_s: u32,
    pub spectators: u16,
}

impl Default for LiveProgress {
    fn default() -> Self {
        LiveProgress {
            hp_bp: [10_000; 2],
            tiers: [0; 2],
            elapsed_s: 0,
            spectators: 0,
        }
    }
}

/// Rows in `/api/live` (featured first). The panel shows far fewer.
pub const LIST_ROWS: usize = 50;

/// Seconds a featured battle keeps the spot unless it ends (BR-LV07).
pub const FEATURED_HOLD_S: u64 = 30;

/// Rating used for a guest in the featured sort (BR-LV07).
pub const GUEST_RATING: u16 = 1000;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LiveSnapshot {
    pub counts: Counts,
    pub featured: Option<LiveId>,
    /// Listed matches: the featured one first, then highest mean rating.
    pub matches: Vec<LiveRow>,
    /// Every online account, rating descending then name (BR-LV06).
    pub accounts: Vec<OnlineRow>,
}

/// BR-LV06: `players` and `accounts` count distinct sessions and accounts
/// with a connection; `in_battle` counts seats of running matches.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Counts {
    pub players: u32,
    pub accounts: u32,
    pub in_battle: u32,
    pub watching: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LiveRow {
    pub id: LiveId,
    pub mode: MatchMode,
    pub arena: Arena,
    pub players: [LiveSeat; 2],
    pub elapsed_s: u32,
    /// Stronghold HP per seat, in basis points of its maximum.
    pub hp_bp: [u16; 2],
    pub spectators: u16,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LiveSeat {
    pub name: String,
    /// `None` for guests; never listed as accounts (BR-LV06).
    pub rating: Option<u16>,
    pub kind: StrongholdKind,
    pub tier: u8,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OnlineRow {
    pub name: String,
    pub rating: u16,
    pub status: PresenceStatus,
}

/// BR-LV06: what an online account is doing right now.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PresenceStatus {
    Idle,
    Queued,
    Playing,
    Watching,
}

impl PresenceStatus {
    pub fn slug(self) -> &'static str {
        match self {
            PresenceStatus::Idle => "idle",
            PresenceStatus::Queued => "queued",
            PresenceStatus::Playing => "playing",
            PresenceStatus::Watching => "watching",
        }
    }
}

/// Stable lowercase name of a Stronghold kind for JSON (not a wire type).
pub fn kind_slug(kind: StrongholdKind) -> &'static str {
    match kind {
        StrongholdKind::Classic => "classic",
        StrongholdKind::Treasury => "treasury",
        StrongholdKind::Bastion => "bastion",
        StrongholdKind::Artillery => "artillery",
        StrongholdKind::Command => "command",
    }
}

/// Mean of two seats' ratings, guests at `GUEST_RATING` (BR-LV07).
pub fn mean_rating(ratings: [Option<u16>; 2]) -> u32 {
    let sum: u32 = ratings
        .into_iter()
        .map(|r| u32::from(r.unwrap_or(GUEST_RATING)))
        .sum();
    sum / 2
}
