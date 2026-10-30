//! `ServerMsg` → world: match info, the render timeline (P30-C), unit
//! entities, event cues. The only place that mirrors coordinates: the
//! player's own Stronghold is always at world x = 0.

use std::collections::HashMap;

use bevy::prelude::*;
use tbg_core::content::LANE_LEN;
use tbg_core::{MatchView, Outcome, Seat, UnitId};

use crate::art::FigureLibrary;
use crate::audio::Sounds;
use crate::vfx::VfxAssets;
use crate::visuals::layout::{self, Body, Layout};
use crate::visuals::{self, UnitLook, VisualAssets};

mod actions;
mod events;
mod messages;
mod timeline;
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
    /// D-40: both frozen Arcana presets, indexed by `Seat::index()`.
    pub arcanas: [tbg_core::ArcanaPreset; 2],
    /// ARENAS-G: the match's arena, from `MatchStart` (never from the URL).
    pub arena: tbg_core::Arena,
    /// P39: the result screen offers a rematch only for a Challenge match.
    pub challenge: bool,
}

/// P39: the result-screen rematch offer and its request state.
#[derive(Resource, Default)]
pub struct RematchInfo {
    /// Ignore offers already in flight when the player explicitly leaves.
    pub dismissed: bool,
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
    pub timeline: timeline::Timeline,
    /// Local arrival time of the newest view (view-gap diagnostics).
    pub last_arrival: Option<f64>,
}

#[derive(Resource, Default)]
pub struct UnitEntities(pub HashMap<UnitId, Entity>);

/// Everything the event pass writes besides the unit visuals.
type FxResources<'w> = (
    Res<'w, VfxAssets>,
    ResMut<'w, Sounds>,
    ResMut<'w, MatchStats>,
    ResMut<'w, visuals::TowerOrders>,
    ResMut<'w, crate::art::aim::ImpactRing>,
);

pub fn plugin(app: &mut App) {
    app.init_resource::<MatchInfo>()
        .init_resource::<LatestView>()
        .init_resource::<ViewBuffer>()
        .init_resource::<UnitEntities>()
        .init_resource::<Layout>()
        .init_resource::<QueueInfo>()
        .init_resource::<RematchInfo>()
        .init_resource::<MatchStats>()
        .init_resource::<ViewGaps>()
        .init_resource::<crate::art::aim::ImpactRing>()
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
    world_tile(x as f32 / 1000.0, you)
}

/// Lane x in tiles → world x, mirrored for seat B.
pub fn world_tile(t: f32, you: Seat) -> f32 {
    match you {
        Seat::A => t,
        Seat::B => LANE_LEN as f32 / 1000.0 - t,
    }
}

#[allow(clippy::too_many_arguments)] // the one system that turns views into the scene
fn sync_units(
    mut commands: Commands,
    time: Res<Time>,
    assets: Res<VisualAssets>,
    lib: Res<FigureLibrary>,
    mut buffer: ResMut<ViewBuffer>,
    mut map: ResMut<UnitEntities>,
    mut layout: ResMut<Layout>,
    mut visuals: events::Visuals,
    mut fx_res: FxResources,
) {
    let now = time.elapsed_secs_f64();
    let mut scene = Scene {
        commands: &mut commands,
        assets: &assets,
        lib: &lib,
        map: &mut map,
        layout: &mut layout,
        dt: time.delta_secs(),
        visuals: &mut visuals,
    };
    // Borrowed, not cloned: the frame lives only while units are placed.
    let placed = buffer
        .timeline
        .frame(now, time.delta_secs_f64())
        .map(|f| scene.place(f.from, f.to, f.alpha));
    let Some((you, alive)) = placed else {
        return;
    };
    let (vfx, sounds, stats, orders, ring) = (
        &fx_res.0,
        &mut *fx_res.1,
        &mut *fx_res.2,
        &mut *fx_res.3,
        &mut *fx_res.4,
    );
    let mut fx = events::Fx {
        assets: &assets,
        vfx,
        sounds,
        stats,
        orders,
        you,
        time: time.elapsed_secs_f64(),
        ring,
    };
    for ev in buffer.timeline.take_events() {
        apply_event(&mut commands, &mut fx, &map, &mut visuals, ev);
    }
    map.0.retain(|id, e| {
        let alive = alive.contains(id);
        if !alive && !visuals.get(*e).is_ok_and(|(v, _, _)| v.dying) {
            commands.entity(*e).try_despawn();
        }
        alive
    });
}

/// What placing units needs from the ECS, borrowed for one frame.
struct Scene<'a, 'w, 's, 'qw, 'qs> {
    commands: &'a mut Commands<'w, 's>,
    assets: &'a VisualAssets,
    lib: &'a FigureLibrary,
    map: &'a mut UnitEntities,
    layout: &'a mut Layout,
    /// Seconds since the last frame, for the layout's easing.
    dt: f32,
    visuals: &'a mut events::Visuals<'qw, 'qs>,
}

impl Scene<'_, '_, '_, '_, '_> {
    /// Move, update or spawn the visual of every unit of `from`, `alpha`
    /// of the way to its position in `to`; return the seat and unit ids.
    fn place(&mut self, from: &MatchView, to: &MatchView, alpha: f32) -> (Seat, Vec<UnitId>) {
        let you = from.you;
        self.layout.begin();
        // Both views list units by id, so one cursor walks `to` with `from`.
        let mut next = 0;
        for u in &from.units {
            while to.units.get(next).is_some_and(|n| n.id < u.id) {
                next += 1;
            }
            // A unit absent from the next view (it died) holds its position.
            let to_x = match to.units.get(next) {
                Some(n) if n.id == u.id => n.x,
                _ => u.x,
            };
            let x = u.x as f32 + (to_x - u.x) as f32 * alpha;
            self.layout.push(Body {
                id: u.id,
                seat: u.seat,
                x: x / 1000.0,
                radius: layout::BASE_RADIUS * self.lib.get(u.kind, u.tier, u.branch).plan.scale,
            });
        }
        self.layout.solve(self.dt);
        for (i, u) in from.units.iter().enumerate() {
            let slot = self.layout.slot(i);
            let pos = Vec3::new(world_tile(slot.x, you), 0.0, slot.z);
            match self
                .map
                .0
                .get(&u.id)
                .and_then(|e| self.visuals.get_mut(*e).ok())
            {
                Some((mut v, mut t, _)) => {
                    t.translation = pos;
                    visuals::update_unit_visual(self.commands, self.assets, self.lib, &mut v, u);
                }
                None => {
                    let look = UnitLook {
                        id: u.id,
                        kind: u.kind,
                        tier: u.tier,
                        branch: u.branch,
                        own: u.seat == you,
                    };
                    let e =
                        visuals::spawn_unit_visual(self.commands, self.assets, self.lib, look, pos);
                    self.map.0.insert(u.id, e);
                }
            }
        }
        (you, from.units.iter().map(|u| u.id).collect())
    }
}

fn clear_units(mut commands: Commands, mut map: ResMut<UnitEntities>, mut layout: ResMut<Layout>) {
    for (_, e) in map.0.drain() {
        commands.entity(e).try_despawn();
    }
    layout.clear();
}
