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

/// Harmful effects have HUD badges separate from the ownership ring.
pub(super) fn hostile(
    commands: &mut Commands,
    a: &VisualAssets,
    v: &mut super::UnitVisual,
    u: &tbg_core::UnitView,
) {
    let alive = u.hp > 0;
    badge(
        commands,
        &mut v.venom_badge,
        (v.bar, &a.tier_cube, &a.marked, 0.81),
        alive && u.effects.and_then(|e| e.venom).is_some(),
    );
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
    commands
        .entity(e)
        .try_insert((Visibility::Inherited, MeshMaterial3d(material.clone())));
}

/// Control is a HUD fact; it must never replace the ownership ring color.
pub(super) fn control(
    commands: &mut Commands,
    a: &VisualAssets,
    v: &mut super::UnitVisual,
    u: &tbg_core::UnitView,
) {
    let material = if u.status == 2 { &a.rooted } else { &a.slowed };
    badge(
        commands,
        &mut v.control_badge,
        (v.bar, &a.tier_cube, material, 0.94),
        u.hp > 0 && u.status != 0,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::RunSystemOnce;

    #[derive(Resource, Default)]
    struct Badge(Option<Entity>);

    #[test]
    fn ring_only_control_badge_changes_material_and_hides_without_reallocation() {
        let mut world = World::new();
        world.init_resource::<Assets<Mesh>>();
        world.init_resource::<Assets<StandardMaterial>>();
        world.init_resource::<Badge>();
        world.run_system_once(super::super::assets::init).unwrap();
        // Exercise the actual retained HUD entity through Root -> Slow -> cleared.
        world
            .run_system_once(
                |mut commands: Commands, a: Res<VisualAssets>, mut slot: ResMut<Badge>| {
                    let bar = commands.spawn_empty().id();
                    badge(
                        &mut commands,
                        &mut slot.0,
                        (bar, &a.tier_cube, &a.rooted, 0.94),
                        true,
                    );
                },
            )
            .unwrap();
        let entity = world.resource::<Badge>().0.unwrap();
        let root = world.resource::<VisualAssets>().rooted.clone();
        assert_eq!(
            world
                .get::<MeshMaterial3d<StandardMaterial>>(entity)
                .unwrap()
                .0,
            root
        );
        world
            .run_system_once(
                |mut commands: Commands, a: Res<VisualAssets>, mut slot: ResMut<Badge>| {
                    let parent = commands.spawn_empty().id();
                    badge(
                        &mut commands,
                        &mut slot.0,
                        (parent, &a.tier_cube, &a.slowed, 0.94),
                        true,
                    );
                },
            )
            .unwrap();
        assert_eq!(world.resource::<Badge>().0, Some(entity));
        assert_eq!(
            world
                .get::<MeshMaterial3d<StandardMaterial>>(entity)
                .unwrap()
                .0,
            world.resource::<VisualAssets>().slowed
        );
        world
            .run_system_once(
                |mut commands: Commands, a: Res<VisualAssets>, mut slot: ResMut<Badge>| {
                    badge(
                        &mut commands,
                        &mut slot.0,
                        (Entity::PLACEHOLDER, &a.tier_cube, &a.slowed, 0.94),
                        false,
                    );
                },
            )
            .unwrap();
        assert_eq!(world.resource::<Badge>().0, Some(entity));
        assert_eq!(
            *world.get::<Visibility>(entity).unwrap(),
            Visibility::Hidden
        );
    }
}
