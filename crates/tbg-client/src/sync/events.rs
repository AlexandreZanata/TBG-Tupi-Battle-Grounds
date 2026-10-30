//! Simulation events → animation cues, effects, sounds, and the stats the
//! result screen shows.

use bevy::prelude::*;
use tbg_core::{Archetype, Event, Seat, Target, UnitId};

use super::{world_x, UnitEntities};
use crate::art::anim::Animator;
use crate::audio::{Sound, Sounds};
use crate::vfx::{self, VfxAssets};
use crate::visuals::{self, AimTarget, TowerOrders, UnitVisual, VisualAssets, VisualCue};

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
    pub orders: &'a mut TowerOrders,
    pub you: Seat,
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
    super::actions::cue(commands, fx, map, visuals, ev);
    match ev {
        Event::Spawn { seat, .. } if seat == you => {
            fx.stats.bought += 1;
            fx.sounds.play(Sound::Deploy);
        }
        Event::AttackStart { from, to } => attack(commands, fx, map, visuals, from, to),
        Event::Hit {
            to: Target::Unit(id),
            ..
        } => cue(map, visuals, fx.assets, id, VisualCue::Hit),
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
                let kit = map
                    .0
                    .get(&unit)
                    .and_then(|e| visuals.get(*e).ok())
                    .and_then(|(v, _, _)| v.kit());
                if let Some(kit) = kit {
                    vfx::impacts::evolution(commands, fx.vfx, kit.evolve, p);
                }
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
    let target = match to {
        Target::Unit(id) => map.0.get(&id).map(|e| AimTarget::Unit(*e)),
        Target::Stronghold(_) => pos_of(map, visuals, you, to).map(AimTarget::Point),
    };
    let (shoots, splash) = match from {
        Target::Unit(id) => {
            cue(map, visuals, fx.assets, id, VisualCue::AttackStart);
            let info =
                map.0
                    .get(&id)
                    .and_then(|e| visuals.get_mut(*e).ok())
                    .map(|(mut v, _, _)| {
                        if let Some(t) = target {
                            v.engage(t);
                        }
                        (v.archetype, v.splash, v.has_kit())
                    });
            let (arch, splash, kit) = info.unwrap_or((Archetype::Tank, 0.0, false));
            fx.sounds.play(match arch {
                Archetype::Ranged if splash > 0.0 => Sound::Cannon,
                Archetype::Ranged | Archetype::Control | Archetype::Support => Sound::Shot,
                Archetype::Tank | Archetype::Dps => Sound::Melee,
            });
            // Kit units fire their own shots and impacts (visuals/attacks.rs).
            let ranged = !matches!(arch, Archetype::Tank | Archetype::Dps);
            (ranged && !kit, if kit { 0.0 } else { splash })
        }
        Target::Stronghold(seat) => {
            // Towers aim and fire their own projectiles (arena/strongholds).
            if let Some(t) = target {
                fx.orders.0.push((seat == you, t));
            }
            (false, 0.0)
        }
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
