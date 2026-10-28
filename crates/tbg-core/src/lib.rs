//! `tbg-core`: deterministic game rules shared by client and server.
//!
//! Domain vocabulary, content tables, the fixed-tick simulation, per-seat
//! views, the bot, and the wire protocol. Free of graphics, I/O, clocks,
//! floats in simulation state, and unseeded randomness (docs/ARCHITECTURE.md §4).

pub mod bot;
mod combat;
pub mod content;
mod hits;
mod intent;
pub mod protocol;
pub mod rng;
pub mod shop;
pub mod sim;
pub mod state;
mod synergy;
mod targeting;
pub mod types;
pub mod view;

pub use intent::{PlayerIntention, Rejection};
pub use rng::Pcg32;
pub use state::{EndReason, Event, MatchState, Outcome, Phase, Target, Unit};
pub use types::*;
pub use view::{MatchView, UnitView};
