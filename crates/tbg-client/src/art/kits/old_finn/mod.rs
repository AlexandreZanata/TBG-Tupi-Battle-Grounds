//! Old Finn kit: a stooped fisherman who whips his rod overhead and flings
//! a fish at the target (it lands as a splash); idle pipe puffs and a
//! slow rod bob; he kneels when he falls.

mod model;

use bevy::prelude::*;

use super::common::{moves, moves::*, Form};
use super::{tools, Attack, Gait, Kit, Moment};
use crate::art::{
    anim::{Clip, Pose},
    rig::{Joint, Node, Proportions, Socket},
};
use crate::vfx::{impacts::Burst, shots::ShotKind};
use model::{plan, TIP};

pub const KIT: Kit = Kit {
    anatomy: None,
    figure: model::spec,
    pose,
    gait: Gait::LIGHT,
    attack: Attack::Shot {
        loaded: None,
        from: Node::Prop(Socket::WeaponR),
        muzzle: |_, _| TIP,
        shot: |_, _| ShotKind::Fish,
        flash: |_, _| None,
    },
    evolve: Burst::Valor,
    durations: moves::DURATIONS,
};

pub fn pose(m: &Moment, pose: &mut Pose) {
    let _ = Form::of(m.tier, m.branch);
    let p = plan(m.tier);
    // A lifetime of hauling nets: the back stays bent.
    tools::add(pose, Joint::Spine, [10.0, 0.0, 0.0]);
    tools::add(pose, Joint::Head, [-10.0, 0.0, 0.0]);
    match m.clip {
        Clip::Idle => {
            moves::idle(m, pose, &p, 0.4);
            let bob = (m.clock * 1.3).sin();
            rod(pose, &p, 35.0 + 6.0 * bob);
        }
        Clip::Walk => {
            moves::walk(m, pose, 6.0, 4.0);
            rod(pose, &p, -30.0);
        }
        Clip::Cast => cast(m, pose, &p),
        Clip::Hit => {
            moves::hit(m, pose, 1.1);
            rod(pose, &p, 30.0);
        }
        Clip::Death => {
            moves::death(m, pose, &p, Fall::Kneel);
            rod(pose, &p, 80.0);
        }
        Clip::Spawn => {
            moves::spawn(m, pose, &p, Entry::March);
            rod(pose, &p, -30.0);
        }
        Clip::Evolve => {
            moves::evolve(m, pose, &p, Power::Raise);
            rod(pose, &p, -20.0 + 20.0 * (m.t / 0.9));
        }
        Clip::Victory => {
            moves::victory(m, pose, &p, Cheer::Dance);
            rod(pose, &p, -10.0);
        }
        _ => rod(pose, &p, 35.0),
    }
}

/// The rod in the right hand, tilted `tilt` degrees forward.
fn rod(pose: &mut Pose, p: &Proportions, tilt: f32) {
    let grip = Quat::from_rotation_x(tilt.to_radians());
    tools::reach(
        pose,
        false,
        Vec3::new(-0.2, 0.06, 0.16),
        Some(grip),
        (p, Vec3::new(-1.0, -0.6, -0.3)),
    );
}

/// Rod back over the shoulder (0–0.15 s), whipped forward to the Release
/// at 0.22 s, the line following through.
fn cast(m: &Moment, pose: &mut Pose, p: &Proportions) {
    let k = (m.t / ATTACK).clamp(0.0, 1.0);
    let back = tools::window(k, 0.0, 0.25);
    let whip = tools::snap(k, 0.25, 0.355);
    let rest = tools::window(k, 0.6, 1.0);
    let tilt = ((-75.0 * back) * (1.0 - whip) + 70.0 * whip) * (1.0 - rest) + 35.0 * rest;
    let lift = (0.25 * back * (1.0 - whip) + 0.1 * whip) * (1.0 - rest);
    tools::add(
        pose,
        Joint::Chest,
        [
            (-10.0 * back + 14.0 * whip) * (1.0 - rest),
            (-20.0 * back + 15.0 * whip) * (1.0 - rest),
            0.0,
        ],
    );
    let grip = Quat::from_rotation_x(tilt.to_radians());
    tools::reach(
        pose,
        false,
        Vec3::new(-0.2, 0.06 + lift, 0.16 + 0.1 * whip),
        Some(grip),
        (p, Vec3::new(-1.0, -0.6, -0.3)),
    );
    tools::face(pose, m.aim, Vec3::ZERO);
    tools::plant(pose, p, tools::stance(p, 1.2, 0.08));
}
