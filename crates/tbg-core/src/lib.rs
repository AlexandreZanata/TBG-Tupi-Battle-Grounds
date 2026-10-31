//! `tbg-core`: deterministic game rules shared by client and server.
//!
//! Domain vocabulary, content tables, the fixed-tick simulation, per-seat
//! views, the bot, and the wire protocol. Free of graphics, I/O, clocks,
//! floats in simulation state, and unseeded randomness (docs/ARCHITECTURE.md §4).

pub mod account;
mod acidmaw;
mod afflictions;
pub mod arcana;
pub mod augment;
mod bonehorn;
pub mod bot;
mod bulwark;
mod capybara;
mod celestial_archer;
mod combat;
pub mod content;
mod crowcaller;
mod demolisher;
pub mod headless;
mod hits;
mod intent;
mod jawguard;
mod justiciar;
#[cfg(feature = "lab")]
pub mod lab;
mod luminous_oracle;
mod mantis;
#[cfg(feature = "bench-profile")]
pub mod profiling;
pub mod protocol;
pub mod rating;
mod rattle_tick;
mod ravager;
pub mod replay;
pub mod rng;
mod roster_slots;
pub mod ruleset;
mod secondary_targets;
pub mod shop;
pub mod sim;
mod solar_sentinel;
mod specials;
pub mod state;
pub mod stronghold;
mod summon;
mod support;
mod synergy;
mod targeting;
mod trident;
mod trigger;
pub mod types;
pub mod view;
mod whisperer;

pub use arcana::ArcanaPreset;
pub use intent::{PlayerIntention, Rejection};
pub use rng::Pcg32;
pub use ruleset::Ruleset;
pub use state::{EndReason, Event, MatchState, Outcome, Phase, PlayerState, Target, Unit};
pub use types::*;
pub use view::{MatchView, PublicView, StrongholdView, UnitView};
