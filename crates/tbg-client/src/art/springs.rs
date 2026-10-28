//! Damped secondary rotation driven by parent velocities, sub-stepped at 120 Hz.

use bevy::prelude::*;

#[derive(Component)]
pub struct SpringJoint {
    parent: Entity,
    pub stiffness: f32,
    pub damping: f32,
    angle: Vec3,
    velocity: Vec3,
    previous: Option<(Vec3, Quat)>,
    remainder: f32,
}
impl SpringJoint {
    pub fn new(parent: Entity) -> Self {
        Self {
            parent,
            stiffness: 60.0,
            damping: 8.0,
            angle: Vec3::ZERO,
            velocity: Vec3::ZERO,
            previous: None,
            remainder: 0.0,
        }
    }
}
pub fn update(
    time: Res<Time>,
    parents: Query<&GlobalTransform>,
    mut joints: Query<(&mut SpringJoint, &mut Transform)>,
) {
    let dt = time.delta_secs().min(0.1);
    if dt <= 0.0 {
        return;
    }
    for (mut spring, mut transform) in &mut joints {
        let Ok(parent) = parents.get(spring.parent) else {
            continue;
        };
        let (position, rotation) = (parent.translation(), parent.rotation());
        let (previous, previous_rotation) = spring
            .previous
            .replace((position, rotation))
            .unwrap_or((position, rotation));
        let linear = rotation.inverse() * (position - previous) / dt;
        let angular = (previous_rotation.inverse() * rotation).to_scaled_axis() / dt;
        let drive = -angular - linear.cross(Vec3::Y);
        spring.remainder += dt;
        while spring.remainder >= 1.0 / 120.0 {
            let acceleration = drive * spring.damping
                - spring.angle * spring.stiffness
                - spring.velocity * spring.damping;
            spring.velocity += acceleration / 120.0;
            let next = spring.angle + spring.velocity / 120.0;
            let limit = Vec3::splat(35f32.to_radians());
            spring.angle = next.clamp(-limit, limit);
            for i in 0..3 {
                if spring.angle[i] != next[i] {
                    spring.velocity[i] = 0.0;
                }
            }
            spring.remainder -= 1.0 / 120.0;
        }
        transform.rotation = Quat::from_scaled_axis(spring.angle);
    }
}
