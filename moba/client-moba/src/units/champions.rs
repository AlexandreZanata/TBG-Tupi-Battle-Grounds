//! 1v1 champion figures on the MOBA map: the same rig, kit, animation clips,
//! shots and impacts as in the 1v1 game, driven by MOBA snapshots.

use bevy::prelude::*;
use tbg_core::types::Archetype;
use tbg_moba_core::{Champion, UnitKind};

use super::UnitVisual;
use crate::art::anim::{Animator, Clip};
use crate::art::melee::{lunge_distance, strike_profile_for_unit};
use crate::art::{figure, presentation, FigureLibrary};
use crate::visuals::CORPSE_SECS;

/// Figure scale on the MOBA map (its lanes are far longer than the BattleLine).
const TROOP_SCALE: f32 = tbg_moba_core::content::CHAMPION_FIGURE_SCALE as f32;

/// A figure finishing its Death clip after the unit left the snapshot.
#[derive(Component)]
pub struct Dying {
    elapsed: f32,
    duration: f32,
}

/// The 1v1 champion a MOBA unit is drawn as, if any.
pub fn champion_of(kind: UnitKind) -> Option<Champion> {
    match kind {
        UnitKind::Troop(champion) => Some(champion),
        _ => None,
    }
}

pub fn scale(_kind: UnitKind) -> f32 {
    TROOP_SCALE
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
    // The shared gait measures travelled world distance. Account for the
    // scale on the MOBA parent when converting that distance into a stride.
    let mut plan = model.plan;
    plan.scale *= TROOP_SCALE;
    let mut animator = Animator::new(joints, plan, body, kind, id);
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
        (super::attacks::release, super::attacks::stomp)
            .after(bevy::transform::TransformSystems::Propagate)
            .run_if(in_state(crate::app::Screen::Match)),
    )
    .add_systems(Update, corpses);
}

/// The body faces before rig evaluation and weapon-joint propagation.
pub(super) fn face(
    time: Res<Time>,
    mut units: Query<(&mut UnitVisual, &Animator, &Transform)>,
    mut bodies: Query<&mut Transform, Without<UnitVisual>>,
) {
    for (mut v, a, root) in &mut units {
        let Ok(mut body) = bodies.get_mut(a.figure) else {
            continue;
        };
        let (clip, at) = a.clip_time();
        let profile = strike_profile_for_unit(a.kind(), a.form.0, a.form.1);
        let lunge = if clip == Clip::AttackMelee
            && a.aim_at.is_some_and(|p| {
                p.xz().distance(root.translation.xz()) < super::attacks::REACH * TROOP_SCALE
            }) {
            lunge_distance(at, 0.30, 0.50, profile.lunge)
        } else {
            0.0
        };
        body.rotation = Quat::from_rotation_y(v.yaw);
        v.recoil.update(time.delta_secs());
        body.translation = body.rotation * Vec3::Z * lunge + v.recoil.offset_vec3();
    }
}

/// Only the server's death event starts a corpse; leaving sight removes the mirror immediately.
pub fn die(commands: &mut Commands, entity: Entity, animator: &mut Animator) {
    animator.play(Clip::Death);
    commands.entity(entity).insert(Dying {
        elapsed: 0.0,
        duration: animator.duration(Clip::Death) + CORPSE_SECS,
    });
}

fn corpses(
    mut commands: Commands,
    time: Res<Time>,
    mut dying: Query<(Entity, &mut Dying, &Animator, &Transform)>,
    mut body: Query<&mut Transform, Without<Dying>>,
) {
    for (entity, mut death, a, root) in &mut dying {
        death.elapsed += time.delta_secs();
        if death.elapsed >= death.duration {
            commands.entity(entity).despawn();
            continue;
        }
        if a.kit.is_none() {
            if let Ok(mut t) = body.get_mut(a.figure) {
                let (_, yaw, _) = t.rotation.to_euler(EulerRot::XYZ);
                *t = crate::art::death::evaluate(
                    tbg_core::content::def(a.kind()).faction,
                    death.elapsed,
                    a.plan.scale / root.scale.x,
                    yaw,
                );
            }
        }
    }
}
