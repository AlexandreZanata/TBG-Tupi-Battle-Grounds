//! `ServerMsg` → world: match info, view buffer with interpolation, unit
//! entities, event cues. The only place that mirrors coordinates: the
//! player's own Stronghold is always at world x = 0.

use std::collections::HashMap;

use bevy::prelude::*;
use tbg_core::content::LANE_LEN;
use tbg_core::{Event, MatchView, Outcome, Seat, UnitId};

use crate::art::FigureLibrary;
use crate::audio::Sounds;
use crate::vfx::VfxAssets;
use crate::visuals::{self, UnitLook, VisualAssets};

mod events;
mod messages;
use crate::GameState;
use events::apply_event;
pub use events::MatchStats;
use messages::poll_link;
pub use messages::{QueueInfo, ViewGaps};

#[derive(Resource, Default)]
pub struct MatchInfo {
    pub you: Option<Seat>,
    /// Post-P14: the HUD shows both names next to the HP bars.
    pub you_name: String,
    pub you_rating: Option<u16>,
    pub opponent: String,
    pub outcome: Option<Outcome>,
    pub last_tick: u32,
    /// P12: ranked matches show a badge and the opponent's rating.
    pub ranked: bool,
    pub opponent_rating: Option<u16>,
    /// P39: the result screen offers a rematch only for a Challenge match.
    pub challenge: bool,
}

/// P39: the result-screen rematch offer and its request state.
#[derive(Resource, Default)]
pub struct RematchInfo {
    pub open: bool,
    pub you_requested: bool,
    pub opponent_requested: bool,
    /// `Some(true)` for an explicit decline, `Some(false)` for expiry.
    pub closed: Option<bool>,
}

#[derive(Resource, Default)]
pub struct LatestView(pub Option<MatchView>);

#[derive(Resource, Default)]
pub(crate) struct ViewBuffer {
    pub prev: Option<(f64, MatchView)>,
    pub last: Option<(f64, MatchView)>,
    pub events: Vec<Event>,
}

#[derive(Resource, Default)]
struct UnitEntities(HashMap<UnitId, Entity>);

pub fn plugin(app: &mut App) {
    app.init_resource::<MatchInfo>()
        .init_resource::<LatestView>()
        .init_resource::<ViewBuffer>()
        .init_resource::<UnitEntities>()
        .init_resource::<QueueInfo>()
        .init_resource::<RematchInfo>()
        .init_resource::<MatchStats>()
        .init_resource::<ViewGaps>()
        .add_systems(Update, poll_link)
        .add_systems(
            Update,
            sync_units
                .after(poll_link)
                .run_if(in_state(GameState::InMatch)),
        )
        .add_systems(OnExit(GameState::InMatch), clear_units);
}

/// Lane x in milli-tiles → world x, mirrored for seat B.
pub fn world_x(x: i32, you: Seat) -> f32 {
    let t = x as f32 / 1000.0;
    match you {
        Seat::A => t,
        Seat::B => LANE_LEN as f32 / 1000.0 - t,
    }
}

fn lane_z(id: UnitId) -> f32 {
    [-0.8, 0.0, 0.8][(id % 3) as usize]
}

#[allow(clippy::too_many_arguments)] // the one system that turns views into the scene
fn sync_units(
    mut commands: Commands,
    time: Res<Time>,
    assets: Res<VisualAssets>,
    lib: Res<FigureLibrary>,
    mut buffer: ResMut<ViewBuffer>,
    mut map: ResMut<UnitEntities>,
    mut visuals: events::Visuals,
    mut fx_res: (Res<VfxAssets>, ResMut<Sounds>, ResMut<MatchStats>),
) {
    let Some((last_t, last)) = buffer.last.clone() else {
        return;
    };
    let you = last.you;
    let alpha = match &buffer.prev {
        Some((prev_t, _)) => {
            ((time.elapsed_secs_f64() - last_t) / (last_t - prev_t).max(0.05)).clamp(0.0, 1.0)
        }
        None => 1.0,
    };
    for u in &last.units {
        let prev_x = buffer
            .prev
            .as_ref()
            .and_then(|(_, p)| p.units.iter().find(|p| p.id == u.id))
            .map_or(u.x, |p| p.x);
        let x = prev_x as f64 + (u.x - prev_x) as f64 * alpha;
        let pos = Vec3::new(world_x(x as i32, you), 0.0, lane_z(u.id));
        match map.0.get(&u.id).and_then(|e| visuals.get_mut(*e).ok()) {
            Some((mut v, mut t, _)) => {
                t.translation = pos;
                visuals::update_unit_visual(&mut commands, &assets, &lib, &mut v, u);
            }
            None => {
                let look = UnitLook {
                    id: u.id,
                    kind: u.kind,
                    tier: u.tier,
                    branch: u.branch,
                    own: u.seat == you,
                };
                map.0.insert(
                    u.id,
                    visuals::spawn_unit_visual(&mut commands, &assets, &lib, look, pos),
                );
            }
        }
    }
    let (vfx, sounds, stats) = (&fx_res.0, &mut *fx_res.1, &mut *fx_res.2);
    let mut fx = events::Fx {
        assets: &assets,
        vfx,
        sounds,
        stats,
        you,
    };
    for ev in std::mem::take(&mut buffer.events) {
        apply_event(&mut commands, &mut fx, &map, &mut visuals, ev);
    }
    map.0.retain(|id, e| {
        let alive = last.units.iter().any(|u| u.id == *id);
        if !alive && !visuals.get(*e).is_ok_and(|(v, _, _)| v.dying) {
            commands.entity(*e).try_despawn();
        }
        alive
    });
}

fn clear_units(mut commands: Commands, mut map: ResMut<UnitEntities>) {
    for (_, e) in map.0.drain() {
        commands.entity(e).try_despawn();
    }
}
