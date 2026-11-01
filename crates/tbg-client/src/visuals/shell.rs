//! Shared detachable turtle plates; lost plates fall independently without altering combat.
use super::UnitVisual;
use crate::art::{anim::Animator, rig::Joint, FigureLibrary};
use bevy::prelude::*;
#[derive(Component)]
struct Shell {
    pieces: [Entity; 4],
    form: (u8, u8),
    count: usize,
}
#[derive(Component)]
struct Falling {
    age: f32,
    velocity: Vec3,
}
pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        PostUpdate,
        update.after(bevy::transform::TransformSystems::Propagate),
    )
    .add_systems(Update, fall);
}
fn update(
    mut commands: Commands,
    lib: Res<FigureLibrary>,
    mut units: Query<(Entity, &UnitVisual, &Animator, Option<&mut Shell>)>,
    globals: Query<&GlobalTransform>,
) {
    for (entity, v, a, shell) in &mut units {
        if v.kind() != tbg_core::UnitKind::Shatterback {
            continue;
        }
        let key = a.form;
        let Some(meshes) = lib.shells.get(&key) else {
            continue;
        };
        let count = usize::from((v.fx >> 8) & 7).min(4);
        let Some(mut shell) = shell else {
            let pieces = std::array::from_fn(|i| {
                let id = commands
                    .spawn((
                        Mesh3d(meshes[i].clone()),
                        MeshMaterial3d(lib.material.clone()),
                        Transform::IDENTITY,
                        if i < count {
                            Visibility::Inherited
                        } else {
                            Visibility::Hidden
                        },
                    ))
                    .id();
                commands
                    .entity(a.joints[Joint::Chest.index()])
                    .add_child(id);
                id
            });
            commands.entity(entity).insert(Shell {
                pieces,
                form: key,
                count,
            });
            continue;
        };
        if shell.form != key {
            for (e, m) in shell.pieces.iter().zip(meshes) {
                commands.entity(*e).try_insert(Mesh3d(m.clone()));
            }
            shell.form = key;
        }
        for (i, mesh) in meshes.iter().enumerate().take(shell.count).skip(count) {
            let e = shell.pieces[i];
            commands.entity(e).try_insert(Visibility::Hidden);
            if let Ok(g) = globals.get(e) {
                commands.spawn((
                    Mesh3d(mesh.clone()),
                    MeshMaterial3d(lib.material.clone()),
                    g.compute_transform(),
                    Falling {
                        age: 0.0,
                        velocity: Vec3::new(if i.is_multiple_of(2) { -0.7 } else { 0.7 }, 1.1, 0.4),
                    },
                ));
            }
        }
        shell.count = count;
    }
}
fn fall(
    mut commands: Commands,
    time: Res<Time>,
    mut pieces: Query<(Entity, &mut Transform, &mut Falling)>,
) {
    for (e, mut t, mut f) in &mut pieces {
        let dt = time.delta_secs().min(0.05);
        f.age += dt;
        f.velocity.y -= 5.0 * dt;
        t.translation += f.velocity * dt;
        t.rotate_local_z(dt * 4.0);
        if f.age > 0.7 {
            commands.entity(e).despawn();
        }
    }
}
