//! Isolated Linux menu diorama: two real core bots, no network or account outcomes.

use super::{timeline::Timeline, MatchStats, UnitEntities};
use crate::{audio::Sounds, GameState};
use bevy::prelude::*;
use tbg_core::{bot::Bot, ArcanaPreset, MatchState, Pcg32, Phase, PlayerIntention, Seat, UnitKind};

#[derive(Component)]
pub(super) struct DemoUnit;

#[derive(Resource)]
pub(super) struct Demo {
    pub timeline: Timeline,
    pub stats: MatchStats,
    pub sounds: Sounds,
    battle: Option<Battle>,
    accumulator: f32,
    age: f32,
    seed: u64,
}

impl Default for Demo {
    fn default() -> Self {
        let seed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(1, |d| d.as_nanos() as u64);
        let mut sounds = Sounds::default();
        sounds.muted = true;
        Self {
            timeline: Timeline::default(),
            stats: MatchStats::default(),
            sounds,
            battle: None,
            accumulator: 0.,
            age: 0.,
            seed,
        }
    }
}

struct Battle {
    state: MatchState,
    bots: [Bot; 2],
    events: Vec<tbg_core::Event>,
    opening: [[UnitKind; 3]; 2],
    recruited: [usize; 2],
}

impl Battle {
    fn new(seed: u64) -> Self {
        let mut rng = Pcg32::new(seed, 31);
        let state = MatchState::with_preview(seed, [ArcanaPreset::DEFAULT; 2])
            .with_arena(crate::config::load_arena());
        // Curate only the opening Shop; purchases, Energy and combat stay in the core.
        let opening = std::array::from_fn(|_| {
            [
                [
                    UnitKind::BoneBrute,
                    UnitKind::Shieldbearer,
                    UnitKind::Rampart,
                ][rng.below(3) as usize],
                [
                    UnitKind::Bloomkeeper,
                    UnitKind::Drumguard,
                    UnitKind::AntidoteBrewer,
                ][rng.below(3) as usize],
                [UnitKind::Farwatch, UnitKind::Rotcap, UnitKind::SiegeWalker]
                    [rng.below(3) as usize],
            ]
        });
        let mut battle = Self {
            state,
            bots: [Bot::new(seed ^ 0xa11), Bot::new(seed ^ 0xb22)],
            events: Vec::new(),
            opening,
            recruited: [0; 2],
        };
        // Start with an already moving skirmish instead of an empty countdown.
        for _ in 0..20 * tbg_core::content::TICK_HZ {
            battle.step();
        }
        battle.events.clear();
        battle
    }

    fn step(&mut self) {
        if self.state.clock.is_multiple_of(2) {
            for seat in Seat::BOTH {
                let i = seat.index();
                if self.state.phase == Phase::Running && self.recruited[i] < 3 {
                    self.state.players[i].offers[0] = self.opening[i][self.recruited[i]];
                    if self
                        .state
                        .apply_intention(
                            seat,
                            self.state.clock,
                            PlayerIntention::Purchase { offer: 0 },
                        )
                        .is_ok()
                    {
                        self.recruited[i] += 1;
                    }
                    continue;
                }
                let view = self.state.view_for(seat, &[]);
                if let Some(intent) = self.bots[seat.index()].decide(&view) {
                    // Presentation budget only; never change competitive army limits.
                    if matches!(intent, PlayerIntention::Purchase { .. })
                        && self.state.army_size(seat) >= 5
                    {
                        continue;
                    }
                    let _ = self.state.apply_intention(seat, self.state.clock, intent);
                }
            }
        }
        self.events.extend(self.state.step());
    }
}

pub(super) fn enabled() -> bool {
    crate::config::bevy_ui() && !crate::config::arena_preview()
}

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<Demo>()
        .add_systems(OnEnter(GameState::Menu), enter)
        .add_systems(OnExit(GameState::Menu), super::clear_units)
        .add_systems(
            Update,
            tick.after(super::poll_link)
                .before(super::sync_units)
                .run_if(in_state(GameState::Menu)),
        );
}

fn enter(mut demo: ResMut<Demo>) {
    demo.battle = None;
}

fn tick(
    time: Res<Time>,
    mut demo: ResMut<Demo>,
    mut commands: Commands,
    units: Query<Entity, With<DemoUnit>>,
    mut map: ResMut<UnitEntities>,
) {
    if !enabled() {
        return;
    }
    let restart = demo
        .battle
        .as_ref()
        .is_none_or(|b| b.state.phase == Phase::Ended)
        || demo.age > 75.;
    if restart {
        for e in &units {
            commands.entity(e).try_despawn();
        }
        map.0.clear();
        demo.seed = demo.seed.wrapping_add(0x9e37_79b9_7f4a_7c15);
        demo.battle = Some(Battle::new(demo.seed));
        demo.timeline = Timeline::default();
        demo.stats = MatchStats::default();
        demo.age = 0.;
        demo.accumulator = 0.;
    }
    let dt = time.delta_secs().min(0.25);
    demo.age += dt;
    demo.accumulator += dt;
    let step = 1. / tbg_core::content::TICK_HZ as f32;
    while demo.accumulator >= step {
        demo.accumulator -= step;
        let Some(battle) = &mut demo.battle else {
            break;
        };
        battle.step();
        if battle.state.clock.is_multiple_of(2) {
            let view = battle.state.view_for(Seat::A, &battle.events);
            battle.events.clear();
            demo.timeline.push(time.elapsed_secs_f64(), view);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opening_recruits_all_three_roles_through_core_purchases() {
        let battle = Battle::new(42);
        assert_eq!(battle.recruited, [3, 3]);
        for lineup in battle.opening {
            let roles = lineup.map(|kind| tbg_core::content::def(kind).archetype);
            assert_eq!(
                roles,
                [
                    tbg_core::Archetype::Tank,
                    tbg_core::Archetype::Support,
                    tbg_core::Archetype::Ranged
                ]
            );
        }
        assert_eq!(battle.state.phase, Phase::Running);
    }
}
