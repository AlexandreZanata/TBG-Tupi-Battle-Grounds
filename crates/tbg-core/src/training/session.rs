//! One local trial (P62 §3.3–3.4, R08): builds exact instances on a private
//! canonical `MatchState`, steps the shared combat order without economy,
//! Augments, tower fire or Stronghold outcomes, and owns the lifecycle.
//! The caller gets read-only access; commands carry the session id so a
//! command from an earlier trial is rejected.

use crate::content::RUNNING_START;
use crate::state::{Event, MatchState, Phase};
use crate::types::{Seat, UnitId};

use super::dummy::{self, DummyMode};
use super::metrics::{TrainingMetrics, MAX_ROWS};
use super::scenario::{
    default_x, Opponent, ScenarioError, Side, TrainingScenario, TrainingUnitSpec, MAX_LIVING,
    RUN_LIMIT_TICKS,
};
use super::Runtime;

/// Identity of one trial; Reset and ReplaceDraft issue a new one.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SessionId(pub u32);

impl SessionId {
    fn next(self) -> SessionId {
        SessionId(self.0.wrapping_add(1))
    }
}

/// Why a trial stopped advancing. Only `Player` can be resumed; the caps
/// keep the results and wait for Reset.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pause {
    Player,
    RunLimit,
    EntityCap,
    RowCap,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Finish {
    AlliesWon,
    OpponentsWon,
    /// Both sides eliminated on the same tick.
    Draw,
    /// Finite dummies: every target destroyed.
    TargetsDestroyed,
    /// Dummy mode: the attacking team itself died (self-burn, blasts).
    AlliesEliminated,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lifecycle {
    Ready,
    Running,
    Paused(Pause),
    Finished(Finish),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TrainingCommand {
    Start,
    Pause,
    Resume,
    /// Advance exactly one simulation tick from Ready or a player pause.
    Step,
    /// Rebuild the same scenario and seed.
    Reset,
    /// Validate a new draft atomically, then start a fresh trial from it.
    ReplaceDraft(Box<TrainingScenario>),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CommandError {
    StaleSession,
    InvalidState,
    /// A safety cap or the run limit stopped this trial; Reset it.
    LimitReached,
    Scenario(ScenarioError),
}

/// A dummy's place; `current` is `None` between a death and its respawn.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct DummySlot {
    x: i32,
    current: Option<UnitId>,
}

#[derive(Clone, Debug)]
pub struct TrainingSession {
    id: SessionId,
    scenario: TrainingScenario,
    state: MatchState,
    lifecycle: Lifecycle,
    active_ticks: u32,
    slots: Vec<DummySlot>,
    /// Events since the last `take_events`.
    events: Vec<Event>,
}

impl TrainingSession {
    pub fn new(
        scenario: TrainingScenario,
        id: SessionId,
    ) -> Result<TrainingSession, ScenarioError> {
        scenario.validate()?;
        Ok(build(scenario, id))
    }

    pub fn id(&self) -> SessionId {
        self.id
    }

    pub fn scenario(&self) -> &TrainingScenario {
        &self.scenario
    }

    /// Read-only canonical state, for views and inspection.
    pub fn state(&self) -> &MatchState {
        &self.state
    }

    pub fn metrics(&self) -> &TrainingMetrics {
        &self.runtime().meter
    }

    pub fn lifecycle(&self) -> Lifecycle {
        self.lifecycle
    }

    /// Simulation ticks advanced since Start/Reset: the denominator of
    /// every rate (`TICK_HZ` per second). Pauses never add to it.
    pub fn active_ticks(&self) -> u32 {
        self.active_ticks
    }

    pub fn is_dummy(&self, id: UnitId) -> bool {
        self.runtime().dummies.contains(&id)
    }

    pub fn take_events(&mut self) -> Vec<Event> {
        std::mem::take(&mut self.events)
    }

    /// Apply a command addressed to `session`; returns the id now current.
    pub fn command(
        &mut self,
        session: SessionId,
        cmd: TrainingCommand,
    ) -> Result<SessionId, CommandError> {
        if session != self.id {
            return Err(CommandError::StaleSession);
        }
        match (cmd, self.lifecycle) {
            (TrainingCommand::Reset, _) => {
                *self = build(self.scenario.clone(), self.id.next());
            }
            (TrainingCommand::ReplaceDraft(draft), _) => {
                draft.validate().map_err(CommandError::Scenario)?;
                *self = build(*draft, self.id.next());
            }
            (TrainingCommand::Start, Lifecycle::Ready)
            | (TrainingCommand::Resume, Lifecycle::Paused(Pause::Player)) => {
                self.lifecycle = Lifecycle::Running;
            }
            (TrainingCommand::Pause, Lifecycle::Running) => {
                self.lifecycle = Lifecycle::Paused(Pause::Player);
            }
            (TrainingCommand::Step, Lifecycle::Ready | Lifecycle::Paused(Pause::Player)) => {
                self.lifecycle = Lifecycle::Paused(Pause::Player);
                self.advance();
            }
            (TrainingCommand::Resume | TrainingCommand::Step, Lifecycle::Paused(_)) => {
                return Err(CommandError::LimitReached);
            }
            _ => return Err(CommandError::InvalidState),
        }
        Ok(self.id)
    }

    /// One fixed step of the caller's clock: advances only while Running.
    pub fn tick(&mut self) {
        if self.lifecycle == Lifecycle::Running {
            self.advance();
        }
    }

    fn runtime(&self) -> &Runtime {
        // Always installed by `build`; never removed.
        self.state
            .training
            .as_deref()
            .expect("training runtime installed by build")
    }

    fn runtime_mut(&mut self) -> &mut Runtime {
        self.state
            .training
            .as_deref_mut()
            .expect("training runtime installed by build")
    }

    /// One simulation tick: scheduled respawns, the shared unit pass and
    /// deaths. No Energy, Interest, Augment, tower fire or Stronghold end.
    fn advance(&mut self) {
        self.respawn();
        self.runtime_mut().meter.now = self.active_ticks;
        let mut ev = std::mem::take(&mut self.state.pending);
        self.state.clock += 1;
        self.state.unit_pass(&mut ev);
        crate::combat::remove_dead(&mut self.state, &mut ev);
        self.state.tick += 1;
        self.active_ticks += 1;
        self.runtime_mut().meter.now = self.active_ticks;
        self.record_deaths(&ev);
        self.register_rows();
        self.events.extend(ev);
        self.evaluate();
    }

    fn respawn(&mut self) {
        if self.dummy_mode() != Some(DummyMode::Renewable) {
            return;
        }
        let hp = match &self.scenario.opponent {
            Opponent::Dummies { hp, .. } => hp.hp(),
            Opponent::Champions(_) => return,
        };
        for i in 0..self.slots.len() {
            if self.slots[i].current.is_none() {
                let id = dummy::spawn(&mut self.state, self.slots[i].x, hp);
                self.slots[i].current = Some(id);
                self.register(id, true);
            }
        }
    }

    fn record_deaths(&mut self, ev: &[Event]) {
        for e in ev {
            let Event::Death { unit } = *e else { continue };
            let rt = self.runtime_mut();
            rt.meter.died(unit);
            rt.dummies.retain(|&d| d != unit);
            for slot in &mut self.slots {
                if slot.current == Some(unit) {
                    slot.current = None;
                }
            }
        }
    }

    /// Every unit gets a row, even one no hook has touched yet.
    fn register_rows(&mut self) {
        for i in 0..self.state.units.len() {
            let u = self.state.units[i];
            let dummy = self.is_dummy(u.id);
            self.runtime_mut().meter.register(&u, dummy);
        }
    }

    fn register(&mut self, id: UnitId, dummy: bool) {
        if let Some(i) = self.state.unit_index(id) {
            let u = self.state.units[i];
            self.runtime_mut().meter.register(&u, dummy);
        }
    }

    fn evaluate(&mut self) {
        if let Some(finish) = self.finish() {
            self.lifecycle = Lifecycle::Finished(finish);
            return;
        }
        let pause = if self.state.units.len() > MAX_LIVING {
            Some(Pause::EntityCap)
        } else if self.metrics().rows().len() >= MAX_ROWS {
            Some(Pause::RowCap)
        } else if self.active_ticks >= RUN_LIMIT_TICKS {
            Some(Pause::RunLimit)
        } else {
            None
        };
        if let Some(p) = pause {
            self.lifecycle = Lifecycle::Paused(p);
        }
    }

    fn finish(&self) -> Option<Finish> {
        let alive = |seat: Seat| {
            self.state
                .units
                .iter()
                .any(|u| u.seat == seat && u.hp > 0 && !self.is_dummy(u.id))
        };
        let allies = alive(Seat::A);
        match self.dummy_mode() {
            None => match (allies, alive(Seat::B)) {
                (true, true) => None,
                (true, false) => Some(Finish::AlliesWon),
                (false, true) => Some(Finish::OpponentsWon),
                (false, false) => Some(Finish::Draw),
            },
            Some(_) if !allies => Some(Finish::AlliesEliminated),
            Some(DummyMode::Finite) if self.slots.iter().all(|s| s.current.is_none()) => {
                Some(Finish::TargetsDestroyed)
            }
            Some(_) => None,
        }
    }

    fn dummy_mode(&self) -> Option<DummyMode> {
        match &self.scenario.opponent {
            Opponent::Dummies { mode, .. } => Some(*mode),
            Opponent::Champions(_) => None,
        }
    }
}

/// Construct a validated scenario: canonical rules, neutral Arcanas and
/// Augments, Classic Strongholds (no auras), the chosen arena, combat
/// running from the first tick.
fn build(scenario: TrainingScenario, id: SessionId) -> TrainingSession {
    let mut state = MatchState::new(scenario.seed).with_arena(scenario.arena);
    state.phase = Phase::Running;
    state.tick = RUNNING_START;
    state.training = Some(Box::default());
    place(&mut state, Seat::A, Side::Allies, &scenario.allies);
    let mut slots = Vec::new();
    match &scenario.opponent {
        Opponent::Champions(units) => place(&mut state, Seat::B, Side::Opponents, units),
        Opponent::Dummies { hp, positions, .. } => {
            for (i, x) in positions.iter().enumerate() {
                let x = x.unwrap_or_else(|| default_x(Side::Opponents, true, i));
                let current = Some(dummy::spawn(&mut state, x, hp.hp()));
                slots.push(DummySlot { x, current });
            }
        }
    }
    let mut session = TrainingSession {
        id,
        scenario,
        state,
        lifecycle: Lifecycle::Ready,
        active_ticks: 0,
        slots,
        events: Vec::new(),
    };
    session.register_rows();
    session
}

fn place(state: &mut MatchState, seat: Seat, side: Side, units: &[TrainingUnitSpec]) {
    for (i, spec) in units.iter().enumerate() {
        let id = state.spawn(seat, spec.kind, spec.tier, spec.branch);
        if let Some(idx) = state.unit_index(id) {
            state.units[idx].x = spec.x.unwrap_or_else(|| default_x(side, false, i));
        }
    }
}
