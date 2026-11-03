//! Where snapshots come from: an in-process host (offline practice) or a
//! room server. Both produce the same `WorldView`/`PrivateView` stream and
//! consume the same intention outbox, so the rest of the client is mode-blind.

use bevy::prelude::*;
use tbg_moba_core::protocol::{PrivateView, WorldView};
use tbg_moba_core::state::Event;
use tbg_moba_core::{MobaIntention, Reject, Seat};

use crate::app::Screen;

mod local;
mod online;

pub use local::restart as restart_local;

/// Intentions the player issued this frame (HUD, hotkeys, right clicks).
#[derive(Resource, Default)]
pub struct Outbox(pub Vec<MobaIntention>);

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum Status {
    #[default]
    Idle,
    Connecting,
    Live,
    Failed(FailReason),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FailReason {
    Lost,
    Full,
}

/// The two latest worlds for interpolation, plus everything HUDs read.
#[derive(Resource)]
pub struct Feed {
    pub prev: Option<WorldView>,
    pub curr: Option<WorldView>,
    /// Fraction of the authoritative interval drawn this frame.
    pub alpha: f32,
    /// Bumped on every new world so systems react once.
    pub generation: u64,
    pub me: Option<PrivateView>,
    pub seat: Option<Seat>,
    pub status: Status,
    /// Events not yet shown by the HUD.
    pub events: Vec<Event>,
    pub motion_events: Vec<Event>,
    pub rejects: Vec<Reject>,
}

impl Default for Feed {
    fn default() -> Self {
        Self {
            prev: None,
            curr: None,
            alpha: 1.0,
            generation: 0,
            me: None,
            seat: None,
            status: Status::Idle,
            events: Vec::new(),
            motion_events: Vec::new(),
            rejects: Vec::new(),
        }
    }
}

impl Feed {
    pub fn push(&mut self, world: WorldView) {
        if self.curr.as_ref().is_some_and(|old| old.tick == world.tick) {
            return;
        }
        self.events.extend(world.events.iter().copied());
        self.motion_events.extend(
            world
                .events
                .iter()
                .filter(|event| {
                    matches!(
                        event,
                        Event::CombatTrace { .. }
                            | Event::ChampionAbility { .. }
                            | Event::UnitDied { .. }
                            | Event::UnitHit { .. }
                    )
                })
                .copied(),
        );
        self.prev = self.curr.replace(world);
        self.generation += 1;
    }
}

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Receive;

pub fn plugin(app: &mut App) {
    app.init_resource::<Outbox>()
        .init_resource::<Feed>()
        .add_systems(OnEnter(Screen::Match), start)
        .add_systems(OnExit(Screen::Match), stop)
        .add_systems(
            Update,
            (local::run, online::run)
                .in_set(Receive)
                .run_if(in_state(Screen::Match)),
        );
}

fn start(world: &mut World) {
    *world.resource_mut::<Feed>() = Feed::default();
    world.resource_mut::<Outbox>().0.clear();
    let launch = world.resource::<crate::config::Launch>().clone();
    if launch.online {
        online::connect(world, &launch.server);
    } else {
        local::start(world, &launch);
    }
}

fn stop(world: &mut World) {
    world.remove_resource::<local::LocalHost>();
    world.remove_non_send::<online::Socket>();
}
