//! Persistent harmful-effect badges use shared geometry and authoritative snapshot presence.
use super::VisualAssets;
use bevy::{light::NotShadowCaster, prelude::*};
pub(super) fn spawn_bleed(commands: &mut Commands, a: &VisualAssets, bar: Entity) -> Entity {
    commands
        .spawn((
            Mesh3d(a.tier_cube.clone()),
            MeshMaterial3d(a.bar_fill[2].clone()),
            Transform::from_xyz(0.55, 0.0, 0.02)
                .with_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_4))
                .with_scale(Vec3::splat(0.7)),
            Visibility::Hidden,
            NotShadowCaster,
            ChildOf(bar),
        ))
        .id()
}
pub(super) fn bleed(commands: &mut Commands, entity: Entity, u: &tbg_core::UnitView) {
    let visible = u.hp > 0 && u.effects.and_then(|f| f.bleed).is_some();
    commands.entity(entity).try_insert(if visible {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    });
}

/// Allocated once on the first real shield, reused/hidden from full view facts.
pub(super) fn shield(
    commands: &mut Commands,
    a: &VisualAssets,
    v: &mut super::UnitVisual,
    u: &tbg_core::UnitView,
) {
    let visible = u.hp > 0 && u.shield > 0;
    if !visible {
        if let Some(e) = v.shield_outline {
            commands.entity(e).try_insert(Visibility::Hidden);
        }
        return;
    }
    let e = *v.shield_outline.get_or_insert_with(|| {
        commands
            .spawn((
                Mesh3d(a.shield_outline.clone()),
                MeshMaterial3d(a.gold.clone()),
                Transform::from_xyz(0.0, 0.7, 0.0)
                    .with_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2))
                    .with_scale(Vec3::new(0.85, 1.0, 1.25)),
                Visibility::Inherited,
                NotShadowCaster,
                ChildOf(v.figure),
            ))
            .id()
    });
    commands.entity(e).try_insert(Visibility::Inherited);
}

/// Marks and weakening keep their own badges when a control ring has priority.
pub(super) fn hostile(
    commands: &mut Commands,
    a: &VisualAssets,
    v: &mut super::UnitVisual,
    u: &tbg_core::UnitView,
) {
    let alive = u.hp > 0;
    badge(
        commands,
        &mut v.mark_badge,
        (v.bar, &a.tier_cube, &a.marked, -0.55),
        alive && v.marked,
    );
    badge(
        commands,
        &mut v.weaken_badge,
        (v.bar, &a.tier_cube, &a.gold, 0.68),
        alive && u.effects.and_then(|e| e.weaken).is_some(),
    );
}
fn badge(
    commands: &mut Commands,
    slot: &mut Option<Entity>,
    look: (Entity, &Handle<Mesh>, &Handle<StandardMaterial>, f32),
    visible: bool,
) {
    if !visible {
        if let Some(e) = *slot {
            commands.entity(e).try_insert(Visibility::Hidden);
        }
        return;
    }
    let (parent, mesh, material, x) = look;
    let e = *slot.get_or_insert_with(|| {
        commands
            .spawn((
                Mesh3d(mesh.clone()),
                MeshMaterial3d(material.clone()),
                Transform::from_xyz(x, 0.0, 0.02)
                    .with_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_4))
                    .with_scale(Vec3::splat(0.7)),
                Visibility::Inherited,
                NotShadowCaster,
                ChildOf(parent),
            ))
            .id()
    });
    commands.entity(e).try_insert(Visibility::Inherited);
}
