//! `tbg-core`: deterministic game rules shared by client and server.
//!
//! Domain vocabulary, content tables, the fixed-tick simulation, per-seat
//! views, the bot, and the wire protocol. Free of graphics, I/O, clocks,
//! floats in simulation state, and unseeded randomness (docs/ARCHITECTURE.md §4).

pub mod account;
pub mod arcana;
pub mod augment;
pub mod bot;
mod combat;
pub mod content;
pub mod headless;
mod hits;
mod intent;
#[cfg(feature = "lab")]
mod lab;
pub mod protocol;
pub mod rating;
pub mod replay;
pub mod rng;
pub mod ruleset;
pub mod shop;
pub mod sim;
pub mod state;
mod summon;
mod synergy;
mod targeting;
pub mod types;
pub mod view;

pub use arcana::ArcanaPreset;
pub use intent::{PlayerIntention, Rejection};
pub use rng::Pcg32;
pub use ruleset::Ruleset;
pub use state::{EndReason, Event, MatchState, Outcome, Phase, PlayerState, Target, Unit};
pub use types::*;
pub use view::{MatchView, UnitView};
