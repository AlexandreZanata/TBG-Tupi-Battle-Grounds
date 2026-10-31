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
    let radius = entries
        .iter()
        .map(|(_, e)| layout::radius(lib.get(e.kind, e.tier, e.branch), &meshes))
        .fold(0.8f32, f32::max);
    let formation = layout::formation(entries.len(), radius, size.x / size.y.max(1.), cast.winner);
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
            (time.elapsed_secs() + phase * duration).rem_euclid(duration)
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
}

fn aim(scene: Res<Scene>, mut camera: Query<&mut Transform, With<ArenaCamera>>) {
    if let Some(view) = scene.view {
        if let Ok(mut transform) = camera.single_mut() {
            *transform = view;
        }
    }
}
