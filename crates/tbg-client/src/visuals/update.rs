//! Per-view refresh of a unit visual: evolution (mesh swap and scale), HP
//! bar, status ring, rank pips.

use bevy::prelude::*;
use tbg_core::UnitView;

use super::{UnitVisual, VisualAssets};
use crate::art::FigureLibrary;

/// Apply evolution, HP, and status from the latest view.
pub fn update_unit_visual(
    commands: &mut Commands,
    a: &VisualAssets,
    lib: &FigureLibrary,
    v: &mut UnitVisual,
    u: &UnitView,
) {
    v.fx = u.fx;
    if v.form != (u.kind, u.tier, u.branch) {
        v.form = (u.kind, u.tier, u.branch);
        v.refresh_stats();
        let form = lib.get(u.kind, u.tier, u.branch);
        for e in v.glow.drain(..) {
            commands.entity(e).try_despawn();
        }
        v.glow =
            crate::art::figure::spawn_glow(commands, &v.joints, &form.glow, &lib.glow_material);
        for (joint, m) in v.joints.iter().zip(form.meshes(v.own)) {
            match m {
                Some(m) => commands
                    .entity(*joint)
                    .try_insert((Mesh3d(m.clone()), MeshMaterial3d(lib.material.clone()))),
                None => commands.entity(*joint).try_remove::<Mesh3d>(),
            };
        }
        let t = Transform::from_rotation(Quat::from_rotation_y(v.yaw))
            .with_scale(Vec3::splat(form.plan.scale));
        commands.entity(v.figure).try_insert(t);
        commands
            .entity(v.bar)
            .try_insert(Transform::from_xyz(0.0, form.overhead, 0.0));
        let mat = &a.star_fill;
        for (i, cube) in v.tiers.iter().enumerate() {
            let vis = if usize::from(u.tier) > i {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            };
            commands.entity(*cube).try_insert((
                vis,
                MeshMaterial3d(mat.clone()),
                Transform::from_xyz(super::overhead::marker_x(u.tier, i), 0.19, 0.025),
            ));
        }
    }
    let ratio = (u.hp as f32 / u.max_hp.max(1) as f32).clamp(0.0, 1.0);
    let fill = &a.bar_fill[usize::from(ratio < 0.6) + usize::from(ratio < 0.3)];
    let t =
        Transform::from_xyz(-(1.0 - ratio) * 0.4, 0.0, 0.01).with_scale(Vec3::new(ratio, 1.0, 1.0));
    commands
        .entity(v.bar_fill)
        .try_insert((t, MeshMaterial3d(fill.clone())));
    super::status_badges::bleed(commands, v.bleed_badge, u);
    super::status_badges::shield(commands, a, v, u);
    super::status_badges::hostile(commands, a, v, u);
    super::status_badges::control(commands, a, v, u);
    let status = match u.status {
        1 => Some(&a.slowed),
        2 => Some(&a.rooted),
        _ if v.marked => Some(&a.marked),
        _ => None,
    };
    commands
        .entity(v.status_ring)
        .try_insert(if status.is_some() {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        });
    if let Some(mat) = status {
        commands
            .entity(v.status_ring)
            .try_insert(MeshMaterial3d(mat.clone()));
    }
    let ring = &a.team[usize::from(!v.own)];
    commands
        .entity(v.ring)
        .try_insert(MeshMaterial3d(ring.clone()));
}
