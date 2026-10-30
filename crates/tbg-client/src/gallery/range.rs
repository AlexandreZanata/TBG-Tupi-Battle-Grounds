//! `?gallery=<unit>&range`: the unit's five forms on the BattleLine, each
//! attacking an enemy dummy at its own range on the real interval, through
//! the full visual path (aim, kit shot or strike, impact, hit reaction).

use bevy::prelude::*;
use tbg_core::{content::effective_stats, UnitKind};

use super::settings::Gallery;
use crate::art::{anim::Animator, FigureLibrary, FORMS};
use crate::vfx::{impacts, VfxAssets};
use crate::visuals::{self, AimTarget, UnitLook, UnitVisual, VisualAssets, VisualCue};

/// A shooter, its dummy, and its clock.
#[derive(Component)]
pub(super) struct Range {
    target: Entity,
    interval: f32,
    next: f32,
    hit: Option<f32>,
    shots: u32,
}

/// Across-lane spacing between the five firing lines.
const LANE: f32 = 1.6;
pub(super) const FIRST_X: f32 = 2.0;

/// Distance from shooter to dummy: inside the form's range, never closer
/// than a melee swing.
pub(super) fn reach(unit: UnitKind) -> f32 {
    (effective_stats(unit, 0, 0, 0).range as f32 / 1000.0 * 0.85).clamp(1.3, 5.2)
}

pub(super) fn spawn(
    commands: &mut Commands,
    lib: &FigureLibrary,
    assets: &VisualAssets,
    unit: UnitKind,
) {
    let gap = reach(unit);
    for (i, (tier, branch)) in FORMS.into_iter().enumerate() {
        let z = (i as f32 - 2.0) * LANE;
        let id = 100 + i as u32;
        let dummy = visuals::spawn_unit_visual(
            commands,
            assets,
            lib,
            UnitLook {
                id: id + 10,
                kind: UnitKind::Shieldbearer,
                tier: 0,
                branch: 0,
                own: false,
            },
            Vec3::new(FIRST_X + gap, 0.0, z),
        );
        let shooter = visuals::spawn_unit_visual(
            commands,
            assets,
            lib,
            UnitLook {
                id,
                kind: unit,
                tier,
                branch,
                own: true,
            },
            Vec3::new(FIRST_X, 0.0, z),
        );
        let s = effective_stats(unit, tier, branch, 0);
        commands.entity(shooter).insert(Range {
            target: dummy,
            interval: f32::from(s.interval) / tbg_core::content::TICK_HZ as f32,
            next: 0.8 + 0.25 * i as f32,
            hit: None,
            shots: 0,
        });
    }
}

/// Start each attack on its interval and land the dummy's hit reaction at
/// the simulation's 0.30 s.
pub(super) fn fire(
    mut commands: Commands,
    time: Res<Time>,
    options: Res<Gallery>,
    (assets, vfx): (Res<VisualAssets>, Res<VfxAssets>),
    mut units: Query<(
        &mut UnitVisual,
        &mut Animator,
        &Transform,
        Option<&mut Range>,
    )>,
) {
    let now = time.elapsed_secs();
    let mut hits = Vec::new();
    for (mut v, mut anim, at, range) in &mut units {
        let Some(mut r) = range else {
            continue;
        };
        if now >= r.next {
            r.next = now + r.interval.max(1.0);
            r.shots += 1;
            if options.evolve && r.shots % 3 == 0 {
                visuals::play_cue(&mut v, &mut anim, &assets, VisualCue::Evolved);
                if let Some(kit) = v.kit() {
                    let p = at.translation + Vec3::Y * 0.6;
                    impacts::evolution(&mut commands, &vfx, kit.evolve, p);
                }
                continue;
            }
            r.hit = Some(now + 0.3);
            v.engage(AimTarget::Unit(r.target));
            visuals::play_cue(&mut v, &mut anim, &assets, VisualCue::AttackStart);
        }
        if r.hit.is_some_and(|at| now >= at) {
            r.hit = None;
            hits.push(r.target);
        }
    }
    for target in hits {
        if let Ok((mut v, mut anim, _, _)) = units.get_mut(target) {
            visuals::play_cue(&mut v, &mut anim, &assets, VisualCue::Hit);
        }
    }
}
