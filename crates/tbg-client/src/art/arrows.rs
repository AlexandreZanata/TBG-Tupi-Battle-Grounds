//! Release-driven cosmetic arrows; combat outcomes remain entirely in the simulation.

use super::{
    anim::{Animator, ClipEvent},
    bow::BowRig,
    FigureLibrary,
};
use bevy::prelude::*;

#[derive(Component)]
pub struct FlyingArrow {
    velocity: Vec3,
    age: f32,
    rate: f32,
}
pub fn release(
    mut commands: Commands,
    lib: Res<FigureLibrary>,
    figures: Query<(&Animator, &BowRig)>,
    sockets: Query<&GlobalTransform>,
    mut logged: Local<bool>,
) {
    for (a, bow) in &figures {
        if !a.archer || !a.events.contains(&ClipEvent::Release) {
            continue;
        }
        let Ok(socket) = sockets.get(bow.socket) else {
            continue;
        };
        let (_, rotation, _) = socket.to_scale_rotation_translation();
        let draw = super::archer_aim::draw(a.clip_time().1);
        let position = socket.transform_point(Vec3::new(0.0, 0.08, -0.23 - draw * 0.16));
        let range = tbg_core::content::effective_stats(tbg_core::UnitKind::Archer, 0, 0, 0).range
            as f32
            / 1000.0;
        // ponytail: facing-directed cosmetic flight until P21 supplies target-aware VFX.
        commands.spawn((
            Mesh3d(lib.bow.arrow[usize::from(!bow.own)].clone()),
            MeshMaterial3d(lib.material.clone()),
            Transform::from_translation(position)
                .with_rotation(rotation)
                .with_scale(socket.scale()),
            FlyingArrow {
                velocity: rotation * Vec3::Z * range / 0.12,
                age: 0.0,
                rate: a.rate,
            },
        ));
        if !*logged {
            info!("Archer Release: 0.18 s, arrow leaves bowstring at {position:?}");
            *logged = true;
        }
    }
}
pub fn fly(
    mut commands: Commands,
    time: Res<Time>,
    mut arrows: Query<(Entity, &mut Transform, &mut FlyingArrow)>,
) {
    for (e, mut t, mut arrow) in &mut arrows {
        let dt = time.delta_secs() * arrow.rate;
        arrow.age += dt;
        t.translation += arrow.velocity * dt;
        if arrow.age >= 0.12 {
            commands.entity(e).despawn();
        }
    }
}
