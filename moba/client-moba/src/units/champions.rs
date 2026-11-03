//! 1v1 champion figures on the MOBA map: the same rig, kit, animation clips,
//! shots and impacts as in the 1v1 game, driven by MOBA snapshots.

use bevy::prelude::*;
use tbg_core::types::Archetype;
use tbg_moba_core::content::commander_champion;
use tbg_moba_core::{Champion, UnitKind};

use super::UnitVisual;
use crate::art::anim::{Animator, Clip, ClipEvent};
use crate::art::kits::Attack;
use crate::art::{figure, presentation, FigureLibrary};
use crate::vfx::impacts::{self, Area};
use crate::vfx::{shots, VfxAssets};
use crate::visuals::CORPSE_SECS;

/// Figure scale on the MOBA map (its lanes are far longer than the BattleLine).
const TROOP_SCALE: f32 = 3.0;
const COMMANDER_SCALE: f32 = 3.8;

/// A figure finishing its Death clip after the unit left the snapshot.
#[derive(Component)]
pub struct Dying(f32);

/// The 1v1 champion a MOBA unit is drawn as, if any.
pub fn champion_of(kind: UnitKind) -> Option<Champion> {
    match kind {
        UnitKind::Troop(champion) => Some(champion),
        UnitKind::Commander(role) => Some(commander_champion(role)),
        _ => None,
    }
}

pub fn scale(kind: UnitKind) -> f32 {
    if matches!(kind, UnitKind::Commander(_)) {
        COMMANDER_SCALE
    } else {
        TROOP_SCALE
    }
}

/// Builds the base-form figure under `entity` and gives it an Animator,
/// exactly as the 1v1 battle does (Spawn clip first).
pub fn spawn_figure(
    commands: &mut Commands,
    lib: &FigureLibrary,
    entity: Entity,
    kind: Champion,
    id: u32,
) {
    let model = lib.get(kind, 0, 0);
    let body = commands
        .spawn((
            Transform::from_scale(Vec3::splat(model.plan.scale)),
            Visibility::Inherited,
        ))
        .id();
    commands.entity(entity).add_child(body);
    let joints = figure::spawn_joints(
        commands,
        body,
        kind,
        &model.plan,
        model.meshes(true),
        &lib.material,
    );
    figure::spawn_glow(commands, &joints, &model.glow, &lib.glow_material);
    presentation::attachments(commands, lib, kind, (0, 0), &joints, &lib.material);
    let mut animator = Animator::new(joints, model.plan, body, kind, id);
    animator.native = model.native;
    animator.play(Clip::Spawn);
    commands.entity(entity).insert(animator);
}

/// The attack clip of a champion's archetype, as in the 1v1 battle cues.
pub fn attack_clip(kind: Champion) -> Clip {
    match tbg_core::content::def(kind).archetype {
        Archetype::Ranged => Clip::AttackRanged,
        Archetype::Support | Archetype::Control => Clip::Cast,
        Archetype::Tank | Archetype::Dps => Clip::AttackMelee,
    }
}

pub fn plugin(app: &mut App) {
    app.add_systems(
        PostUpdate,
        (face, strikes)
            .chain()
            .after(bevy::transform::TransformSystems::Propagate),
    )
    .add_systems(Update, corpses);
}

/// Figures face along the unit's heading (+Z forward in the 1v1 rig).
fn face(
    units: Query<(&UnitVisual, &Animator)>,
    mut bodies: Query<&mut Transform, Without<UnitVisual>>,
) {
    for (v, a) in &units {
        if let Ok(mut body) = bodies.get_mut(a.figure) {
            body.rotation = Quat::from_rotation_y(v.yaw + std::f32::consts::PI);
        }
    }
}

/// Fires each kit's shot on Release and lands its melee impact on Strike,
/// from the weapon joint, like the 1v1 battle presentation.
fn strikes(
    mut commands: Commands,
    vfx: Res<VfxAssets>,
    animators: Query<&Animator>,
    globals: Query<&GlobalTransform>,
) {
    for a in &animators {
        let Some(kit) = a.kit else { continue };
        let (t, b) = a.form;
        let joint = |node: crate::art::rig::Node| globals.get(a.joints[node.index()]).ok();
        let release = a.events.contains(&ClipEvent::Release);
        let strike = a.events.contains(&ClipEvent::Strike);
        let to = a.aim_at;
        match kit.attack {
            Attack::Shot {
                from, muzzle, shot, ..
            } if release => {
                if let (Some(g), Some(to)) = (joint(from), to) {
                    shots::fire(
                        &mut commands,
                        &vfx,
                        shot(t, b),
                        g.transform_point(muzzle(t, b)),
                        to,
                        Area::default(),
                    );
                }
            }
            Attack::CountedShot {
                from,
                muzzle,
                normal,
                ..
            } if release => {
                if let (Some(g), Some(to)) = (joint(from), to) {
                    shots::fire(
                        &mut commands,
                        &vfx,
                        normal,
                        g.transform_point(muzzle(t, b)),
                        to,
                        Area::default(),
                    );
                }
            }
            Attack::TwinShot {
                hands,
                muzzle,
                shot,
                ..
            } if release => {
                if let (Some(g), Some(to)) = (joint(hands[a.attack_variant()]), to) {
                    shots::fire(
                        &mut commands,
                        &vfx,
                        shot,
                        g.transform_point(muzzle),
                        to,
                        Area::default(),
                    );
                }
            }
            Attack::Melee { from, tip, impact } if strike => {
                if let Some(g) = joint(from) {
                    let at = to.unwrap_or_else(|| g.transform_point(tip(t, b)));
                    impacts::land(&mut commands, &vfx, impact(t, b), at, Area::default());
                }
            }
            _ => {}
        }
    }
}

/// Starts the Death clip of a figure whose unit just died.
pub fn die(commands: &mut Commands, entity: Entity, animator: &mut Animator) {
    animator.play(Clip::Death);
    commands
        .entity(entity)
        .insert(Dying(Clip::Death.duration() + CORPSE_SECS));
}

fn corpses(mut commands: Commands, time: Res<Time>, mut dying: Query<(Entity, &mut Dying)>) {
    for (entity, mut left) in &mut dying {
        left.0 -= time.delta_secs();
        if left.0 <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}
