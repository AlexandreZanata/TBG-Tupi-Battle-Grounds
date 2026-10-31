//! Build only the visible cosmetic page and restore the one arena camera on hide/exit.

use super::{layout, range, total, Presentation};
use crate::{
    art::{anim::Animator, figure, FigureLibrary},
    camera::ArenaCamera,
    sync::MatchInfo,
    GameState,
};
use bevy::prelude::*;

#[derive(Component)]
pub(crate) struct Owned;
#[derive(Component)]
struct Dancer(u64);
#[derive(Resource, Default)]
struct Scene {
    built: Option<([u8; 16], u64, bool, bool, Vec2)>,
    camera: Option<(Entity, Transform, Projection, Vec2)>,
    view: Option<Transform>,
}

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<Scene>()
        .add_systems(OnEnter(GameState::Result), enter)
        .add_systems(Update, build.run_if(in_state(GameState::Result)))
        .add_systems(
            PostUpdate,
            (animate, aim)
                .before(crate::art::anim::animate)
                .before(bevy::transform::TransformSystems::Propagate)
                .run_if(in_state(GameState::Result)),
        )
        .add_systems(OnExit(GameState::Result), exit);
}

fn enter(
    mut state: ResMut<Presentation>,
    info: Res<MatchInfo>,
    mut arena: ResMut<crate::arena::ActiveArena>,
) {
    state.page = 0;
    state.hidden = false;
    if info.outcome.is_some() {
        arena.set_if_neq(crate::arena::ActiveArena::of(info.arena));
    }
}

#[allow(clippy::too_many_arguments)]
fn build(
    mut commands: Commands,
    info: Res<MatchInfo>,
    mut state: ResMut<Presentation>,
    mut scene: ResMut<Scene>,
    lib: Res<FigureLibrary>,
    meshes: Res<Assets<Mesh>>,
    window: Single<&Window>,
    mut cameras: Query<(Entity, &mut Transform, &mut Projection), With<ArenaCamera>>,
    owned: Query<Entity, With<Owned>>,
    mut bounds: Local<std::collections::HashMap<(tbg_core::UnitKind, u8, u8), (f32, f32)>>,
) {
    let Some(total) = total(&info) else {
        return;
    };
    let Some(id) = info.result_id else {
        return;
    };
    let size = Vec2::new(window.width(), window.height());
    let page = state.page.min(total.saturating_sub(1) / super::PAGE_SIZE);
    if state.page != page {
        state.page = page;
    }
    let key = (id, state.page, state.hidden, state.reduced, size);
    if scene.built == Some(key) {
        return;
    }
    for root in &owned {
        commands.entity(root).despawn();
    }
    restore(&mut scene, &mut cameras, size);
    scene.built = Some(key);
    scene.view = None;
    if state.hidden || total == 0 {
        return;
    }
    let Some(cast) = &info.celebration else {
        return;
    };
    let (first, last) = range(total, state.page);
    let entries: Vec<_> = (first..last)
        .filter_map(|i| cast.at(i).map(|e| (i, e)))
        .collect();
    let (mut radius, mut height) = (0.2f32, 0.5f32);
    for (_, e) in &entries {
        let envelope = bounds.entry((e.kind, e.tier, e.branch)).or_insert_with(|| {
            crate::art::celebration::bounds::envelope(
                lib.get(e.kind, e.tier, e.branch),
                &meshes,
                e.kind,
                e.tier,
                e.branch,
            )
        });
        radius = radius.max(envelope.0);
        height = height.max(envelope.1);
    }
    let formation = layout::formation(
        entries.len(),
        radius,
        height,
        size.x / size.y.max(1.),
        cast.winner,
    );
    if let Ok((entity, mut transform, mut projection)) = cameras.single_mut() {
        scene.camera = Some((entity, *transform, projection.clone(), size));
        *transform = formation.camera;
        if let Projection::Perspective(p) = &mut *projection {
            p.fov = crate::camera::FOV_Y;
        }
    }
    scene.view = Some(formation.camera);
    for ((index, entry), position) in entries.into_iter().zip(formation.positions) {
        let form = lib.get(entry.kind, entry.tier, entry.branch);
        let root = commands
            .spawn((
                Transform::from_translation(position),
                Visibility::Inherited,
                Owned,
                Dancer(index),
            ))
            .id();
        let figure = commands
            .spawn((
                Transform::from_rotation(layout::facing(position, formation.camera.translation))
                    .with_scale(Vec3::splat(form.plan.scale * formation.scale)),
                Visibility::Inherited,
            ))
            .id();
        commands.entity(root).add_child(figure);
        let joints = figure::spawn_joints(
            &mut commands,
            figure,
            &form.plan,
            form.meshes(true),
            &lib.material,
        );
        figure::spawn_glow(&mut commands, &joints, &form.glow, &lib.glow_material);
        let mut animator = Animator::new(joints, form.plan, figure, entry.kind, index as u32);
        animator.native = form.native;
        animator.form = (entry.tier, entry.branch);
        animator.treadmill = true;
        animator.archer = form.native && entry.kind == tbg_core::UnitKind::Archer;
        if animator.archer {
            crate::art::bow::attach(
                &mut commands,
                root,
                &animator,
                &lib,
                &lib.material,
                true,
                form.bow_half,
            );
        }
        if state.reduced {
            for joint in joints {
                commands
                    .entity(joint)
                    .remove::<crate::art::springs::SpringJoint>();
            }
        }
        commands.entity(root).insert(animator);
    }
}

fn animate(
    time: Res<Time>,
    state: Res<Presentation>,
    mut dancers: Query<(&Dancer, &mut Animator)>,
) {
    for (slot, mut animator) in &mut dancers {
        let phase = (slot.0.wrapping_mul(2654435761) % 997) as f32 / 997.;
        let motion = crate::art::celebration::celebration_for(
            animator.kind(),
            animator.form.0,
            animator.form.1,
        );
        let duration = motion.duration;
        let t = if state.reduced {
            duration * 0.4
        } else {
            if motion.looping {
                time.elapsed_secs() + phase * duration
            } else {
                time.elapsed_secs().min(duration)
            }
        };
        animator.celebrate(motion, t);
    }
}

fn restore(
    scene: &mut Scene,
    cameras: &mut Query<(Entity, &mut Transform, &mut Projection), With<ArenaCamera>>,
    size: Vec2,
) {
    if let Some((entity, transform, projection, old_size)) = scene.camera.take() {
        if let Ok((_, mut t, mut p)) = cameras.get_mut(entity) {
            *t = transform;
            *p = projection;
            if size != old_size {
                crate::camera::frame(&mut t, &mut p, size.x, size.y);
            }
        }
    }
}

fn exit(
    mut commands: Commands,
    mut scene: ResMut<Scene>,
    window: Single<&Window>,
    mut cameras: Query<(Entity, &mut Transform, &mut Projection), With<ArenaCamera>>,
    owned: Query<Entity, With<Owned>>,
) {
    for root in &owned {
        commands.entity(root).despawn();
    }
    restore(
        &mut scene,
        &mut cameras,
        Vec2::new(window.width(), window.height()),
    );
    scene.built = None;
    scene.view = None;
}

fn aim(scene: Res<Scene>, mut camera: Query<&mut Transform, With<ArenaCamera>>) {
    if let Some(view) = scene.view {
        if let Ok(mut transform) = camera.single_mut() {
            *transform = view;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::RunSystemOnce;

    #[test]
    fn celebration_exit_restores_camera_and_removes_the_whole_page() {
        let mut world = World::new();
        let window = Window::default();
        let size = Vec2::new(window.width(), window.height());
        world.spawn(window);
        let original = Transform::from_xyz(8., 20., 10.);
        let camera = world
            .spawn((ArenaCamera, Transform::default(), Projection::default()))
            .id();
        let child = world.spawn_empty().id();
        let root = world.spawn((Owned, Transform::default())).id();
        world.entity_mut(root).add_child(child);
        world.insert_resource(Scene {
            built: Some(([63; 16], 1, false, false, size)),
            camera: Some((camera, original, Projection::default(), size)),
            view: Some(Transform::from_xyz(8., 2., 5.)),
        });

        world.run_system_once(exit).unwrap();

        assert_eq!(*world.get::<Transform>(camera).unwrap(), original);
        assert!(world.get_entity(root).is_err() && world.get_entity(child).is_err());
        let scene = world.resource::<Scene>();
        assert!(scene.camera.is_none() && scene.built.is_none() && scene.view.is_none());
        // A later nonwinning Result must not inherit the winner's camera override.
        world.run_system_once(aim).unwrap();
        assert_eq!(*world.get::<Transform>(camera).unwrap(), original);
    }
}
