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
mod bot_shop;
mod bulwark;
mod capybara;
pub mod celebration;
mod celestial_archer;
mod combat;
pub mod content;
mod crowcaller;
mod demolisher;
mod drillnest;
pub mod headless;
mod hits;
mod hornet;
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
mod pyre_warden;
pub mod ranked_integrity;
pub mod rating;
mod rattle_tick;
mod ravager;
pub mod replay;
pub mod rng;
mod roster_slots;
pub mod ruleset;
pub mod ruleset_version;
mod secondary_targets;
pub mod shop;
pub mod sim;
mod siphon;
mod solar_sentinel;
mod specials;
pub mod state;
pub mod stronghold;
mod summon;
mod support;
mod synergy;
mod targeting;
#[cfg(feature = "training")]
pub mod training;
mod training_hooks;
mod trident;
mod trigger;
pub mod types;
pub mod view;
mod weaver;
mod whisperer;

pub use arcana::ArcanaPreset;
pub use intent::{PlayerIntention, Rejection};
pub use rng::Pcg32;
pub use ruleset::Ruleset;
pub use state::{EndReason, Event, MatchState, Outcome, Phase, PlayerState, Target, Unit};
pub use types::*;
pub use view::{MatchView, PublicView, StrongholdView, UnitView};

pub mod practice_bot;
