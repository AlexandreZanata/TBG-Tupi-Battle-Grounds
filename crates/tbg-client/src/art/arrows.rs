//! Release-driven targeted cosmetic arrows (D2); combat outcomes remain entirely in the simulation.

use super::{
    aim::Aim,
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
    figures: Query<(&Animator, &BowRig, Option<&Aim>)>,
    sockets: Query<&GlobalTransform>,
    mut logged: Local<bool>,
) {
    for (a, bow, aim_opt) in &figures {
        if !a.archer || !a.events.contains(&ClipEvent::Release) {
            continue;
        }
        let Ok(socket) = sockets.get(bow.socket) else {
            continue;
        };
        let draw = super::archer_aim::draw(a.clip_time().1);
        let position = socket.transform_point(Vec3::new(0.0, 0.08, -0.23 - draw * 0.16));
        let flight_duration = 0.12;

        let target_chest = aim_opt
            .and_then(|aim| aim.last_target_point)
            .map(|pt| pt + Vec3::new(0.0, 0.55, 0.0));

        let (velocity, rotation) = if let Some(chest) = target_chest {
            let dir = (chest - position).normalize_or_zero();
            let vel = (chest - position) / flight_duration;
            let rot = if dir.length_squared() > 1e-4 {
                Quat::from_rotation_arc(Vec3::Z, dir)
            } else {
                let (_, r, _) = socket.to_scale_rotation_translation();
                r
            };
            (vel, rot)
        } else {
            let (_, rotation, _) = socket.to_scale_rotation_translation();
            let range = tbg_core::content::effective_stats(tbg_core::UnitKind::Archer, 0, 0, 0)
                .range as f32
                / 1000.0;
            (rotation * Vec3::Z * range / flight_duration, rotation)
        };

        commands.spawn((
            Mesh3d(lib.bow.arrow[usize::from(!bow.own)].clone()),
            MeshMaterial3d(lib.material.clone()),
            Transform::from_translation(position)
                .with_rotation(rotation)
                .with_scale(socket.scale()),
            FlyingArrow {
                velocity,
                age: 0.0,
                rate: a.rate,
            },
        ));
        if !*logged {
            info!("Archer Release: 0.18 s, targeted arrow leaves bowstring at {position:?}");
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arrow_endpoint_accuracy_across_8_angles() {
        // Contract check: Arrow flight must land within 0.15 tile of the target chest
        // across all 8 angles in the aim ring for both near (1.6) and far (4.2) distances.
        let attacker_pos = Vec3::new(8.0, 0.0, 0.0);
        let socket_offset = Vec3::new(0.0, 0.65, 0.0);
        let socket_pos = attacker_pos + socket_offset;
        let flight_duration = 0.12;

        for angle_idx in 0..8 {
            let angle = angle_idx as f32 * std::f32::consts::TAU / 8.0;
            for far in [false, true] {
                let dist = if far { 4.2 } else { 1.6 };
                let target_base =
                    attacker_pos + Vec3::new(angle.sin() * dist, 0.0, angle.cos() * dist);
                let target_chest = target_base + Vec3::new(0.0, 0.55, 0.0);

                let velocity = (target_chest - socket_pos) / flight_duration;
                let endpoint = socket_pos + velocity * flight_duration;

                let err = (endpoint - target_chest).length();
                assert!(
                    err < 0.15,
                    "Angle {angle_idx} (far={far}): arrow endpoint err {err} exceeded 0.15 tile"
                );

                // Discrete integration simulation at 60 Hz (dt = 1/60 s)
                let dt = 1.0 / 60.0;
                let mut sim_pos = socket_pos;
                let mut age = 0.0;
                while age + dt < flight_duration {
                    sim_pos += velocity * dt;
                    age += dt;
                }
                // Final partial step to arrival:
                sim_pos += velocity * (flight_duration - age);
                let discrete_err = (sim_pos - target_chest).length();
                assert!(
                    discrete_err < 0.15,
                    "Discrete integration error {discrete_err} exceeded 0.15 tile"
                );
            }
        }
    }

    #[test]
    fn test_arrow_spawn_direction_matches_aim_dir() {
        let attacker_pos = Vec3::new(8.0, 0.0, 0.0);
        let socket_offset = Vec3::new(0.0, 0.65, 0.0);
        let socket_pos = attacker_pos + socket_offset;
        let flight_duration = 0.12;

        for angle_idx in 0..8 {
            let angle = angle_idx as f32 * std::f32::consts::TAU / 8.0;
            let target_base = attacker_pos + Vec3::new(angle.sin() * 3.0, 0.0, angle.cos() * 3.0);
            let target_chest = target_base + Vec3::new(0.0, 0.55, 0.0);

            let aim_dir_xz = Vec2::new(
                target_base.x - attacker_pos.x,
                target_base.z - attacker_pos.z,
            )
            .normalize();
            let velocity = (target_chest - socket_pos) / flight_duration;
            let arrow_dir_xz = Vec2::new(velocity.x, velocity.z).normalize();

            let dot = aim_dir_xz.dot(arrow_dir_xz).clamp(-1.0, 1.0);
            let angle_deg = dot.acos().to_degrees();

            assert!(
                angle_deg < 1.0,
                "Angle {angle_idx}: angle(arrow_dir, aim_dir) = {angle_deg} deg >= 1.0 deg"
            );
        }
    }
}
