//! `ServerMsg` → world: match info, the render timeline (P30-C), unit
//! entities and event cues in a common blue-left/red-right world.

use std::collections::HashMap;

use bevy::prelude::*;
use tbg_core::{MatchView, Outcome, Seat, UnitId};

use crate::art::FigureLibrary;
use crate::audio::Sounds;
use crate::vfx::VfxAssets;
use crate::visuals::{self, UnitLook, VisualAssets};

mod actions;
pub(crate) mod crowd_ecs;
mod events;
#[cfg(tbg_native_ui)]
pub(crate) mod lobby_battle;
mod messages;
mod result;
mod special_cues;
mod spectate;
mod timeline;
use crate::GameState;
use events::apply_event;
pub use events::MatchStats;
use messages::poll_link;
pub use messages::{QueueInfo, ViewGaps};

#[derive(Resource, Default)]
pub struct MatchInfo {
    pub personal: Option<Box<tbg_core::protocol::PersonalMatchSummary>>,
    pub history_save: Option<tbg_core::protocol::HistorySaveState>,
    pub result_id: Option<tbg_core::protocol::ResultId>,
    pub celebration: Option<tbg_core::celebration::CelebrationCast>,
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
    /// P47: a watcher, not a seat: no seat controls, a Leave button instead.
    pub spectating: bool,
}

/// P39: the result-screen rematch offer and its request state.
#[derive(Resource, Default)]
pub struct RematchInfo {
    pub opponent: String,
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
    #[cfg(tbg_native_ui)]
    lobby_battle::plugin(app);
    app.init_resource::<MatchInfo>()
        .init_resource::<LatestView>()
        .init_resource::<crate::vfx::barrels::Entities>()
        .init_resource::<ViewBuffer>()
        .init_resource::<UnitEntities>()
        .init_resource::<QueueInfo>()
        .init_resource::<RematchInfo>()
        .init_resource::<MatchStats>()
        .init_resource::<ViewGaps>()
        .init_resource::<crate::art::aim::ImpactRing>()
        .init_resource::<crate::visuals::mount::Mounts>()
        .add_systems(Update, poll_link)
        .add_systems(
            Update,
            sync_units
                .after(poll_link)
                .after(crate::training::runtime::feed)
                .run_if(render_units),
        )
        .add_systems(
            Update,
            crowd_ecs::update
                .after(sync_units)
                .after(crate::stress::walk)
                .before(visuals::aim::update_aim),
        )
        .add_systems(
            Update,
            crate::visuals::mount::passengers
                .after(crowd_ecs::update)
                .before(visuals::aim::update_aim)
                .run_if(|s: Res<State<GameState>>| {
                    matches!(*s.get(), GameState::InMatch | GameState::Training)
                }),
        )
        .add_systems(OnExit(GameState::InMatch), clear_units);
}

fn render_units(state: Res<State<GameState>>, shell: Res<crate::ui::shell::Shell>) -> bool {
    if shell.panel == crate::ui::shell::Panel::Troops && *state.get() == GameState::Menu {
        return false;
    }
    #[cfg(tbg_native_ui)]
    if *state.get() == GameState::Menu {
        return lobby_battle::enabled();
    }
    matches!(*state.get(), GameState::InMatch | GameState::Training)
}

/// Authoritative lane x in milli-tiles, identical for every viewer.
pub fn world_x(x: i32, you: Seat) -> f32 {
    world_tile(x as f32 / 1000.0, you)
}

/// Common world orientation; ownership never changes presentation coordinates.
pub fn world_tile(t: f32, _you: Seat) -> f32 {
    t
}

#[allow(clippy::too_many_arguments)] // the one system that turns views into the scene
pub(crate) fn sync_units(
    mut commands: Commands,
    time: Res<Time>,
    assets: Res<VisualAssets>,
    lib: Res<FigureLibrary>,
    mut buffer: ResMut<ViewBuffer>,
    mut map: ResMut<UnitEntities>,
    mut visuals: events::Visuals,
    mut fx_res: FxResources,
    mut training: ResMut<crate::training::Trial>,
    mut barrels: ResMut<crate::vfx::barrels::Entities>,
    state: Res<State<GameState>>,
    #[cfg(tbg_native_ui)] mut demo: ResMut<lobby_battle::Demo>,
) {
    let now = time.elapsed_secs_f64();
    let dummies = training
        .session
        .as_ref()
        .map(|s| s.dummy_ids().to_vec())
        .unwrap_or_default();
    let (vfx, sounds, stats, orders, ring) = (
        &fx_res.0,
        &mut *fx_res.1,
        &mut *fx_res.2,
        &mut *fx_res.3,
        &mut *fx_res.4,
    );
    #[cfg(tbg_native_ui)]
    let menu = *state.get() == GameState::Menu;
    #[cfg(tbg_native_ui)]
    let (timeline, sounds, stats) = if menu {
        let lobby_battle::Demo {
            timeline,
            sounds,
            stats,
            ..
        } = &mut *demo;
        (timeline, sounds, stats)
    } else {
        (&mut buffer.timeline, sounds, stats)
    };
    #[cfg(not(tbg_native_ui))]
    let timeline = &mut buffer.timeline;
    let stats = if *state.get() == GameState::Training {
        &mut training.stats
    } else {
        stats
    };
    let mut scene = Scene {
        commands: &mut commands,
        assets: &assets,
        lib: &lib,
        map: &mut map,
        visuals: &mut visuals,
        #[cfg(tbg_native_ui)]
        menu,
        dummies: &dummies,
    };
    // Borrowed, not cloned: the frame lives only while units are placed.
    let placed = timeline.frame(now, time.delta_secs_f64()).map(|f| {
        crate::vfx::barrels::place(scene.commands, vfx, &mut barrels, f.from, f.to, f.alpha);
        scene.place(f.from, f.to, f.alpha)
    });
    let Some((you, alive)) = placed else {
        crate::vfx::barrels::clear(scene.commands, &mut barrels);
        return;
    };
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
    for ev in timeline.take_events() {
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
    visuals: &'a mut events::Visuals<'qw, 'qs>,
    dummies: &'a [UnitId],
    #[cfg(tbg_native_ui)]
    menu: bool,
}

impl Scene<'_, '_, '_, '_, '_> {
    /// Move, update or spawn the visual of every unit of `from`, `alpha`
    /// of the way to its position in `to`; return the seat and unit ids.
    fn place(&mut self, from: &MatchView, to: &MatchView, alpha: f32) -> (Seat, Vec<UnitId>) {
        let you = from.you;
        let marked: std::collections::BTreeSet<_> = from
            .units
            .iter()
            .filter(|u| u.kind == tbg_core::UnitKind::Crowcaller)
            .filter_map(|u| u.effects.and_then(|e| e.mark).map(|m| m.target))
            .collect();
        // Both views list units by id, so one cursor walks `to` with `from`.
        let mut next = 0;
        for u in &from.units {
            let anchor = crowd_ecs::LaneAnchor::from_view(u, you, from.physical_lanes);
            while to.units.get(next).is_some_and(|n| n.id < u.id) {
                next += 1;
            }
            // A unit absent from the next view (it died) holds its position.
            let to_x = match to.units.get(next) {
                Some(n) if n.id == u.id => n.x,
                _ => u.x,
            };
            let x = u.x as f32 + (to_x - u.x) as f32 * alpha;
            let to_z = to
                .units
                .get(next)
                .filter(|n| n.id == u.id)
                .map_or(u.lane_z, |n| n.lane_z);
            let z = f32::from(u.lane_z) + (f32::from(to_z) - f32::from(u.lane_z)) * alpha;
            let pos = Vec3::new(world_x(x.round() as i32, you), 0.0, z / 1000.0);
            match self
                .map
                .0
                .get(&u.id)
                .and_then(|e| self.visuals.get_mut(*e).ok())
            {
                Some((mut v, _, mut anim)) => {
                    v.marked = marked.contains(&u.id);
                    anim.carried = u.is_carried();
                    anim.set_reduction(u.reduction);
                    anim.observe_counter(u.ability_counter());
                    anim.completion = u.effects.and_then(|e| e.completion);
                    anim.observe_special(
                        u.special.is_some_and(|s| s.ready_in == 0),
                        u.special.and_then(|s| s.cast).map(|c| c.left),
                        u.special.map_or(tbg_core::content::WINDUP, |s| s.windup),
                        u.status == 2,
                    );
                    if anim.completion.is_some_and(|f| f.interrupt_ticks > 0)
                        && u.special.is_none_or(|s| s.cast.is_none())
                    {
                        anim.cancel_special();
                    }
                    visuals::update_unit_visual(self.commands, self.assets, self.lib, &mut v, u);
                }
                None => {
                    let look = UnitLook {
                        id: u.id,
                        kind: u.kind,
                        tier: u.tier,
                        branch: u.branch,
                        own: u.seat == you,
                        seat: u.seat,
                    };
                    let e = if self.dummies.contains(&u.id) {
                        visuals::spawn_training_dummy(
                            self.commands,
                            self.assets,
                            self.lib,
                            look,
                            pos,
                        )
                    } else {
                        visuals::spawn_unit_visual(self.commands, self.assets, self.lib, look, pos)
                    };
                    #[cfg(tbg_native_ui)]
                    if self.menu {
                        self.commands
                            .entity(e)
                            .remove::<crate::ui::UiAction>()
                            .insert((lobby_battle::DemoUnit, DespawnOnExit(GameState::Menu)));
                    }
                    self.commands
                        .entity(e)
                        .insert(crowd_ecs::CrowdState::default());
                    self.map.0.insert(u.id, e);
                }
            }
            if let Some(e) = self.map.0.get(&u.id) {
                self.commands
                    .entity(*e)
                    .insert(crowd_ecs::LaneAnchor { pos, ..anchor })
                    .insert(crate::vfx::completion::Facts {
                        view: u.effects.and_then(|f| f.completion),
                        you,
                        alive: u.hp > 0,
                        form: (u.kind, u.tier, u.branch),
                    });
            }
        }
        (you, from.units.iter().map(|u| u.id).collect())
    }
}

fn clear_units(
    mut commands: Commands,
    mut map: ResMut<UnitEntities>,
    mut mounts: ResMut<crate::visuals::mount::Mounts>,
    roots: Query<Entity, cleanup::BattleVisuals>,
    mut impacts: ResMut<crate::art::aim::ImpactRing>,
) {
    cleanup::clear(&mut commands, &mut map, &mut mounts, &roots, &mut impacts);
}

pub(crate) mod cleanup;

#[cfg(test)]
mod side_tests {
    use super::*;

    #[test]
    fn side01_players_and_spectators_share_world_coordinates_and_facing() {
        for viewer in Seat::BOTH {
            assert_eq!(world_x(0, viewer), 0.);
            assert_eq!(world_x(16_000, viewer), 16.);
            assert_eq!(world_x(4_750, viewer), 4.75);
            for seat in Seat::BOTH {
                let mut populated = tbg_core::MatchState::new(41);
                while populated.phase != tbg_core::Phase::Running {
                    populated.step();
                }
                populated.players[seat.index()].energy = 20 * tbg_core::content::ENERGY_SCALE;
                populated
                    .apply_intention(seat, 1, tbg_core::PlayerIntention::Purchase { offer: 0 })
                    .unwrap();
                let unit = populated
                    .view_for(viewer, &[])
                    .units
                    .into_iter()
                    .find(|u| u.seat == seat)
                    .unwrap();
                let anchor = crowd_ecs::LaneAnchor::from_view(&unit, viewer, false);
                assert_eq!(anchor.body.facing, seat.dir() as f32);
            }
        }
    }
}
