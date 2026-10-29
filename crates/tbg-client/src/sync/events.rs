//! Simulation events → animation cues, effects, sounds, and the stats the
//! result screen shows.

use bevy::prelude::*;
use tbg_core::{Archetype, Event, Seat, Target, UnitId};

use super::{world_x, UnitEntities};
use crate::art::anim::Animator;
use crate::audio::{Sound, Sounds};
use crate::vfx::{self, VfxAssets};
use crate::visuals::{self, UnitVisual, VisualAssets, VisualCue};

/// What the player did this match (result screen).
#[derive(Resource, Default)]
pub struct MatchStats {
    pub bought: u32,
    pub evolutions: u32,
    pub damage_dealt: i32,
}

pub(super) type Visuals<'w, 's> = Query<
    'w,
    's,
    (
        &'static mut UnitVisual,
        &'static mut Transform,
        &'static mut Animator,
    ),
>;

/// Everything an event may touch besides the unit visuals.
pub(super) struct Fx<'a> {
    pub assets: &'a VisualAssets,
    pub vfx: &'a VfxAssets,
    pub sounds: &'a mut Sounds,
    pub stats: &'a mut MatchStats,
    pub you: Seat,
    pub time: f64,
    pub ring: &'a mut crate::art::aim::ImpactRing,
}

fn pos_of(map: &UnitEntities, visuals: &Visuals, you: Seat, t: Target) -> Option<Vec3> {
    match t {
        Target::Unit(id) => map
            .0
            .get(&id)
            .and_then(|e| visuals.get(*e).ok())
            .map(|(_, t, _)| t.translation + Vec3::Y * 0.6),
        Target::Stronghold(seat) => Some(Vec3::new(
            world_x(tbg_core::state::stronghold_x(seat), you),
            1.2,
            0.0,
        )),
    }
}

fn cue(map: &UnitEntities, visuals: &mut Visuals, a: &VisualAssets, id: UnitId, c: VisualCue) {
    if let Some((mut v, _, mut anim)) = map.0.get(&id).and_then(|e| visuals.get_mut(*e).ok()) {
        visuals::play_cue(&mut v, &mut anim, a, c);
    }
}

pub(super) fn apply_event(
    commands: &mut Commands,
    fx: &mut Fx,
    map: &UnitEntities,
    visuals: &mut Visuals,
    ev: Event,
) {
    let you = fx.you;
    let at = |visuals: &Visuals, t| pos_of(map, visuals, you, t);
    match ev {
        Event::Spawn { seat, .. } if seat == you => {
            fx.stats.bought += 1;
            fx.sounds.play(Sound::Deploy);
        }
        Event::AttackStart { from, to } => attack(commands, fx, map, visuals, from, to),
        Event::Hit {
            to: Target::Unit(id),
            ..
        } => {
            let _attacker = fx.ring.pop_matching(Target::Unit(id), fx.time);
            cue(map, visuals, fx.assets, id, VisualCue::Hit);
        }
        Event::Hit {
            to: Target::Stronghold(seat),
            amount,
        } => {
            if seat != you {
                fx.stats.damage_dealt += amount;
            }
            if let Some(p) = at(visuals, Target::Stronghold(seat)) {
                vfx::sparks(commands, fx.vfx, p);
            }
            fx.sounds.play(Sound::StrongholdHit);
        }
        Event::Heal { to, .. } => {
            cue(map, visuals, fx.assets, to, VisualCue::Heal);
            if let Some(p) = at(visuals, Target::Unit(to)) {
                vfx::heal(commands, fx.vfx, p);
            }
            fx.sounds.play(Sound::Heal);
        }
        Event::Evolved { unit, .. } => {
            cue(map, visuals, fx.assets, unit, VisualCue::Evolved);
            let own = map
                .0
                .get(&unit)
                .and_then(|e| visuals.get(*e).ok())
                .is_some_and(|(v, _, _)| v.own);
            fx.stats.evolutions += u32::from(own);
            if let Some(p) = at(visuals, Target::Unit(unit)) {
                vfx::evolve(commands, fx.vfx, p);
            }
            fx.sounds.play(Sound::Evolve);
        }
        Event::Death { unit } => {
            if let Some(p) = at(visuals, Target::Unit(unit)) {
                vfx::puff(commands, fx.vfx, p);
            }
            cue(map, visuals, fx.assets, unit, VisualCue::Death);
        }
        Event::SynergyChanged {
            seat,
            faction,
            level,
        } if level > 0 => {
            let own = seat == you;
            for e in map.0.values() {
                if let Ok((v, t, _)) = visuals.get(*e) {
                    if v.own == own && v.faction == faction {
                        vfx::synergy(commands, fx.vfx, t.translation, own);
                    }
                }
            }
            if own {
                fx.sounds.play(Sound::Synergy);
            }
        }
        _ => {}
    }
}

fn attack(
    commands: &mut Commands,
    fx: &mut Fx,
    map: &UnitEntities,
    visuals: &mut Visuals,
    from: Target,
    to: Target,
) {
    let you = fx.you;
    let (shoots, splash) = match from {
        Target::Unit(id) => {
            cue(map, visuals, fx.assets, id, VisualCue::AttackStart);
            let info = map
                .0
                .get(&id)
                .and_then(|e| visuals.get(*e).ok())
                .map(|(v, _, anim)| (v.archetype, v.splash, anim.archer, v.yaw));
            let (arch, splash, is_archer, base_yaw) =
                info.unwrap_or((Archetype::Tank, 0.0, false, 0.0));

            if let Some(&e) = map.0.get(&id) {
                let hold = match arch {
                    Archetype::Ranged => 0.18,
                    Archetype::Support | Archetype::Control => 0.22,
                    Archetype::Tank | Archetype::Dps => 0.30,
                };
                commands.entity(e).insert(crate::art::aim::Aim::new(
                    to, fx.time, hold, base_yaw, base_yaw,
                ));
            }
            fx.ring.push(from, to, fx.time + 0.30);

            fx.sounds.play(match arch {
                Archetype::Ranged if splash > 0.0 => Sound::Cannon,
                Archetype::Ranged | Archetype::Control | Archetype::Support => Sound::Shot,
                Archetype::Tank | Archetype::Dps => Sound::Melee,
            });
            // Native archer fires its own targeted projectile at Release (D2).
            // Eliminate duplicate generic projectile from AttackStart (F5).
            (
                !matches!(arch, Archetype::Tank | Archetype::Dps) && !is_archer,
                splash,
            )
        }
        Target::Stronghold(_) => (true, 0.0),
    };
    let (a, b) = (
        pos_of(map, visuals, you, from),
        pos_of(map, visuals, you, to),
    );
    if let (true, Some(a), Some(b)) = (shoots, a, b) {
        visuals::spawn_projectile(commands, fx.assets, a, b);
    }
    if let (true, Some(b)) = (splash > 0.0, b) {
        vfx::splash(commands, fx.vfx, b, splash);
    }
}
