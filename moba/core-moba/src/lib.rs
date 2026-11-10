//! 5v5 MOBA deterministic core: map, rules, bots, snapshot view and wire protocol.
//!
//! Integer-only simulation at [`content::TICK_HZ`]; the server runs it
//! authoritatively and the client runs the same code for offline practice.
//! Canon: `docs/moba/GAME-DESIGN-MOBA.md`. Contracts: `docs/moba/CONTRACTS.md`.

pub mod bot;
pub mod champions;
pub mod content;
pub mod geom;
pub mod grid;
pub mod host;
pub mod intent;
pub mod map;
pub mod protocol;
pub mod rng;
pub mod shop;
pub mod sim;
pub mod state;
pub mod types;
pub mod view;
pub mod vision;

pub use intent::{MobaIntention, Reject};
pub use state::MobaMatch;
pub use types::{Champion, Destination, Lane, Role, Seat, Team, UnitKind};
