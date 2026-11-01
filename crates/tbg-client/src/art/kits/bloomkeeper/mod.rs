//! Bloomkeeper kit: a dryad who lifts a blessing hand while idle, walks
//! lightly with her staff, and thrusts the flower at the ally she heals so
//! the petals leave it on the cast Release; she sinks to her knees on
//! death and grows up out of the soil on arrival.

mod model;

use bevy::prelude::*;

use super::common::{moves, moves::*};
use super::{tools, Attack, Gait, Kit, Moment};
use crate::art::{
    anim::{Clip, Pose},
    rig::{Joint, Node, Socket},
};
use crate::vfx::{impacts::Burst, shots::ShotKind};
use model::plan;

pub const KIT: Kit = Kit {
    anatomy: None,
    figure: model::spec,
    pose,
    gait: Gait::LIGHT,
    attack: Attack::Shot {
        loaded: None,
        from: Node::Prop(Socket::WeaponR),
        muzzle: |_, _| model::muzzle(),
        shot: |_, _| ShotKind::Petal,
        flash: |_, _| None,
    },
    evolve: Burst::Bloom,
    durations: moves::DURATIONS,
};

pub fn pose(m: &Moment, pose: &mut Pose) {
    let p = plan(m.tier);
    match m.clip {
        Clip::Idle => {
            moves::idle(m, pose, &p, 0.1);
            moves::staff(pose, &p, 0.0, 0.0, 0.0);
            let bless = (m.clock * 0.9).sin() * 0.5 + 0.5;
            let palm = Vec3::new(0.2, 0.2 + 0.05 * bless, 0.22);
            tools::reach(pose, true, palm, None, (&p, Vec3::new(1.0, -0.6, -0.3)));
            tools::set(pose, Joint::HandL, [-20.0, 0.0, 25.0]);
            tools::add(pose, Joint::Head, [-3.0 * bless, 5.0 * bless, 0.0]);
        }
        Clip::Walk => {
            moves::walk(m, pose, 2.0, 3.0);
            moves::staff(pose, &p, 8.0, 0.0, 0.04);
            let step = (m.walk.unwrap_or(0.0) * std::f32::consts::TAU).sin();
            tools::set(pose, Joint::UpperArmL, [-18.0 + 10.0 * step, 0.0, 18.0]);
            tools::set(pose, Joint::ForearmL, [-30.0, 0.0, 0.0]);
            tools::add(pose, Joint::Head, [0.0, 3.0 * step, -2.0 * step]);
        }
        Clip::Cast => {
            let gather = tools::hold(m.t, [0.0, 0.14, 0.18, 0.30]);
            let offer = tools::hold(m.t, [0.16, 0.22, 0.34, 0.62]);
            tools::set(
                pose,
                Joint::Chest,
                [-5.0 * gather + 7.0 * offer, 0.0, -3.0 * offer],
            );
            moves::staff(
                pose,
                &p,
                -8.0 * gather + 42.0 * offer,
                0.06 * gather + 0.03 * offer,
                0.06 * offer,
            );
            tools::reach(
                pose,
                true,
                Vec3::new(
                    0.18 - 0.05 * offer,
                    0.23 + 0.03 * gather,
                    0.20 + 0.07 * offer,
                ),
                Some(Quat::from_rotation_z(25f32.to_radians())),
                (&p, Vec3::new(1.0, -0.6, -0.3)),
            );
            tools::set(pose, Joint::Head, [-6.0 * gather, 0.0, 0.0]);
            // A healer may cast behind while its feet keep the lane heading.
            // Twisting the full Root toward each heal fought forward locomotion.
            if let Some(aim) = m.aim {
                let yaw = super::aim::yaw(Vec3::ZERO, aim.target)
                    .clamp(-65f32.to_radians(), 65f32.to_radians())
                    * aim.weight;
                tools::add(pose, Joint::Chest, [0.0, yaw.to_degrees(), 0.0]);
                tools::add(pose, Joint::Head, [0.0, yaw.to_degrees() * 0.25, 0.0]);
            }
            tools::plant(pose, &p, tools::stance(&p, 1.15, 0.06));
        }
        Clip::Hit => moves::hit(m, pose, 1.0),
        Clip::Death => moves::death(m, pose, &p, Fall::Kneel),
        Clip::Spawn => moves::spawn(m, pose, &p, Entry::Rise),
        Clip::Evolve => moves::evolve(m, pose, &p, Power::Spin),
        Clip::Victory => {
            let bloom = 0.5 - 0.5 * (m.t / 0.8 * std::f32::consts::TAU).cos();
            tools::set(
                pose,
                Joint::Chest,
                [-4.0 * bloom, 0.0, 3.0 * (m.t * 7.85).sin()],
            );
            tools::set(pose, Joint::Head, [-8.0 * bloom, 5.0, 0.0]);
            moves::staff(pose, &p, 4.0 + 7.0 * bloom, 0.025 * bloom, 0.0);
            tools::reach(
                pose,
                true,
                Vec3::new(0.24 + 0.05 * bloom, 0.25 + 0.08 * bloom, 0.18),
                Some(Quat::from_rotation_z(35f32.to_radians())),
                (&p, Vec3::new(1.0, -0.4, -0.3)),
            );
            tools::plant(pose, &p, tools::stance(&p, 1.1, 0.02));
        }
        _ => moves::staff(pose, &p, 0.0, 0.0, 0.0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn healing_behind_never_turns_the_locomotion_root() {
        for side in [-0.001, 0.001] {
            for t in [0.0, 0.15, 0.3, 0.6] {
                let mut m = Moment {
                    clip: Clip::Cast,
                    t,
                    tier: 0,
                    branch: 0,
                    aim: None,
                    since_attack: 0.0,
                    variant: 0,
                    clock: 0.0,
                    walk: None,
                    gesture: 0.0,
                    growth: 0.0,
                    reduction: 0,
                    counter: 0,
                    armor_flash: 0.0,
                    special: None,
                    special_ready: false,
                };
                m.aim = Some(super::super::Aim {
                    target: Vec3::new(side, 0.6, -2.0),
                    weight: 1.0,
                });
                let mut p = crate::art::curves::sample(Clip::Cast, t);
                pose(&m, &mut p);
                assert_eq!(p[Joint::Root.index()].0.y, 0.0);
            }
        }
    }
}
