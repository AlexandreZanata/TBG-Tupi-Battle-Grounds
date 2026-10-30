//! Archer kit: a skilled archer who steps side-on to loose arrows from an
//! upright longbow. Paws draw string to cheek, holds aim along the arrow's arc,
//! releases at 0.18 s with a crisp recoil, and reaches back to the quiver for
//! the next shot.

mod model;

use bevy::prelude::*;

use super::common::{moves, moves::*};
use super::{tools, Attack, Gait, Kit, Moment};
use crate::art::{
    anim::{Clip, Pose},
    rig::{Joint, Node, Proportions, Socket},
};
use crate::vfx::{
    impacts::{Burst, Muzzle},
    shots::{self, ShotKind},
};
use model::{plan, MUZZLE};

pub const KIT: Kit = Kit {
    figure: model::spec,
    pose,
    gait: Gait::LIGHT,
    attack: Attack::Shot {
        from: Node::Prop(Socket::OffhandL),
        muzzle: |_, _| MUZZLE,
        shot: |_, _| ShotKind::Arrow,
        flash: |tier, _| {
            if tier >= 1 {
                Some((Muzzle::Bolt, None))
            } else {
                None
            }
        },
    },
    evolve: Burst::Valor,
    durations: moves::DURATIONS,
};

pub fn pose(m: &Moment, pose: &mut Pose) {
    let p = plan(m.tier);
    match m.clip {
        Clip::Idle => {
            moves::idle(m, pose, &p, 0.2);
            let ready = m.aim.map_or(0.0, |a| a.weight);
            // Relaxed carry vs combat ready stance
            tools::set(
                pose,
                Joint::UpperArmL,
                [-25.0 - 20.0 * ready, 0.0, -8.0 + 12.0 * ready],
            );
            tools::set(pose, Joint::ForearmL, [-65.0 + 20.0 * ready, 0.0, 0.0]);
            tools::set(
                pose,
                Joint::UpperArmR,
                [-8.0 - 15.0 * ready, 0.0, -8.0 - 10.0 * ready],
            );
            tools::set(pose, Joint::ForearmR, [-15.0 - 25.0 * ready, 0.0, 0.0]);
        }
        Clip::Walk => {
            moves::walk(m, pose, 5.0, 2.0);
            // Low carry in left hand while walking
            tools::set(pose, Joint::UpperArmL, [-28.0, 0.0, -8.0]);
            tools::set(pose, Joint::ForearmL, [-60.0, 0.0, 0.0]);
        }
        Clip::AttackRanged => aim(m, pose, &p),
        Clip::Hit => {
            moves::hit(m, pose, 0.8);
            tools::set(pose, Joint::UpperArmL, [-30.0, 0.0, -5.0]);
            tools::set(pose, Joint::ForearmL, [-50.0, 0.0, 0.0]);
        }
        Clip::Death => {
            moves::death(m, pose, &p, Fall::Back);
        }
        Clip::Spawn => {
            moves::spawn(m, pose, &p, Entry::March);
            tools::set(pose, Joint::UpperArmL, [-30.0, 0.0, -5.0]);
            tools::set(pose, Joint::ForearmL, [-50.0, 0.0, 0.0]);
        }
        Clip::Evolve => {
            moves::evolve(m, pose, &p, Power::Raise);
            // Both arms raise bow overhead
            tools::set(pose, Joint::UpperArmL, [-165.0, 0.0, 20.0]);
            tools::set(pose, Joint::ForearmL, [-15.0, 0.0, 0.0]);
            tools::set(pose, Joint::UpperArmR, [-160.0, 0.0, -20.0]);
            tools::set(pose, Joint::ForearmR, [-15.0, 0.0, 0.0]);
        }
        Clip::Victory => {
            moves::victory(m, pose, &p, Cheer::Raise);
            // Victorious bow salute
            tools::set(pose, Joint::UpperArmL, [-170.0, 0.0, 15.0]);
            tools::set(pose, Joint::ForearmL, [-10.0, 0.0, 0.0]);
            tools::set(pose, Joint::UpperArmR, [-150.0, 0.0, -30.0]);
            tools::set(pose, Joint::ForearmR, [-20.0, 0.0, 0.0]);
        }
        _ => {
            moves::idle(m, pose, &p, 0.2);
        }
    }
}

/// Side-on archer draw (0–0.12 s), aim elevation along arc (to 0.18 s), Release
/// with recoil at 0.18 s, and reach back to the quiver (0.28–0.62 s).
fn aim(m: &Moment, pose: &mut Pose, p: &Proportions) {
    let draw_k = tools::window(m.t, 0.0, 0.12);
    let recover_k = tools::window(m.t, 0.45, 0.62);
    let kick = tools::snap(m.t, 0.18, 0.21) * (1.0 - tools::window(m.t, 0.22, 0.32));
    let reach = tools::window(m.t, 0.30, 0.44) * (1.0 - tools::window(m.t, 0.48, 0.60));

    // Turn body side-on to face target
    tools::face(pose, m.aim, Vec3::ZERO);
    tools::shift(pose, Joint::Hips, [0.0, -p.leg * 0.06 * draw_k, 0.0]);
    tools::add(
        pose,
        Joint::Hips,
        [0.0, -42.0 * draw_k * (1.0 - recover_k), 0.0],
    );
    tools::add(
        pose,
        Joint::Spine,
        [0.0, -15.0 * draw_k * (1.0 - recover_k), 0.0],
    );
    tools::add(
        pose,
        Joint::Chest,
        [0.0, -22.0 * draw_k * (1.0 - recover_k), 0.0],
    );
    tools::add(
        pose,
        Joint::Head,
        [0.0, 78.0 * draw_k * (1.0 - recover_k), 0.0],
    );

    // Elevation along arrow flight arc
    let up = moves::aim_level(m, pose, 0.85, shots::arc(ShotKind::Arrow));
    let up_deg = up.to_degrees() * draw_k * (1.0 - recover_k);
    tools::add(pose, Joint::Chest, [-up_deg * 0.4, 0.0, 0.0]);

    // Left arm: extends forward holding the bow toward target
    let bow_pitch = -88.0 * draw_k - up_deg * 0.6 - 12.0 * kick + 60.0 * recover_k;
    tools::set(
        pose,
        Joint::UpperArmL,
        [bow_pitch, 0.0, 42.0 * draw_k * (1.0 - recover_k)],
    );
    tools::set(
        pose,
        Joint::ForearmL,
        [-10.0 * draw_k - 8.0 * kick - 50.0 * recover_k, 0.0, 0.0],
    );

    // Right arm: draws string to cheek, recoils back, then reaches for quiver
    if m.t < 0.18 {
        // Drawing string back to anchor near cheek
        tools::set(
            pose,
            Joint::UpperArmR,
            [-65.0 * draw_k - up_deg * 0.3, 0.0, -68.0 * draw_k],
        );
        tools::set(pose, Joint::ForearmR, [-112.0 * draw_k, 0.0, 0.0]);
    } else if m.t < 0.30 {
        // Release recoil: hand pops back past ear
        tools::set(
            pose,
            Joint::UpperArmR,
            [-70.0 - 15.0 * kick, 0.0, -75.0 - 12.0 * kick],
        );
        tools::set(pose, Joint::ForearmR, [-85.0 + 35.0 * kick, 0.0, 0.0]);
    } else {
        // Reaching back over shoulder to quiver
        tools::set(
            pose,
            Joint::UpperArmR,
            [
                -50.0 - 80.0 * reach + 42.0 * recover_k,
                0.0,
                -30.0 + 20.0 * reach - 10.0 * recover_k,
            ],
        );
        tools::set(
            pose,
            Joint::ForearmR,
            [-45.0 - 65.0 * reach + 30.0 * recover_k, 0.0, 0.0],
        );
    }

    tools::plant(
        pose,
        p,
        tools::stance(p, 1.25, 0.14 * draw_k * (1.0 - recover_k)),
    );
}
