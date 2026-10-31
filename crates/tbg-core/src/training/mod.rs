//! Local Troops training laboratory (P62, D-80): exact compositions, inert
//! dummies and attributed meters over the canonical combat step.
//!
//! Only the client enables the `training` feature; the server never
//! constructs a session. Nothing here touches the Shop, Energy, rating or
//! the wire protocol, and a normal match never carries a `Runtime`.

mod dummy;
mod metrics;
mod scenario;
mod session;

pub use crate::training_hooks::{Effect, Source};
pub use dummy::{DummyHp, DummyMode};
pub use metrics::{InstanceRow, TeamTotals, TrainingMetrics, MAX_ROWS};
pub use scenario::{
    default_x, Opponent, ScenarioError, Side, TrainingScenario, TrainingUnitSpec, LANE_MAX,
    LANE_MIN, MAX_DUMMIES, MAX_LIVING, MAX_UNITS_PER_SIDE, RUN_LIMIT_TICKS, SCHEMA,
};
pub use session::{
    CommandError, Finish, Lifecycle, Pause, SessionId, TrainingCommand, TrainingSession,
};

use crate::state::Unit;
use crate::types::{Seat, UnitId};

/// What a `TrainingSession` installs on its private `MatchState`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct Runtime {
    /// Living dummy ids (at most `MAX_DUMMIES`).
    pub dummies: Vec<UnitId>,
    /// Attribution of the HP change being resolved (`training_hooks`).
    pub context: Option<(Source, Effect)>,
    pub meter: TrainingMetrics,
}

/// A hook's source, resolved against the live units.
#[derive(Clone, Copy, Debug)]
pub(crate) enum Attributed<'a> {
    Unit(&'a Unit),
    /// The source unit already left the lane; its row still exists.
    Gone(UnitId),
    Team(Seat),
    Unknown,
}
