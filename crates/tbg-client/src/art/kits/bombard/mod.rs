//! Bombard kit: a heavy gunner who braces, hip-fires a mortar so the bomb
//! lobs along its own arc, is shoved back by the recoil, then drops the
//! next bomb down the barrel. Heavy stomping walk, a hard landing.

mod model;

use bevy::prelude::*;

use super::common::{moves, moves::*, Form};
use super::{tools, Attack, Flash, Gait, Kit, Moment};
use crate::art::{
    anim::{Clip, Pose},
    rig::{Joint, Node, Proportions, Socket},
};
use crate::vfx::{
    impacts::{Burst, Muzzle},
    shots::{self, ShotKind},
};
use model::{plan, FORE};

pub const KIT: Kit = Kit {
    anatomy: None,
    figure: model::spec,
    pose,
    gait: Gait::HEAVY,
    attack: Attack::Shot {
        loaded: None,
        from: Node::Prop(Socket::WeaponR),
        muzzle: |tier, branch| model::muzzle(Form::of(tier, branch)),
        shot: |_, _| ShotKind::Bomb,
        flash,
    },
    evolve: Burst::Forge,
    durations: moves::DURATIONS,
};

fn flash(tier: u8, branch: u8) -> Flash {
    let f = Form::of(tier, branch);
    let muzzle = if f.a && f.t2 {
        Muzzle::GreatBombard
    } else {
        Muzzle::Bombard
    };
    Some((muzzle, None))
}

pub fn pose(m: &Moment, pose: &mut Pose) {
    let f = Form::of(m.tier, m.branch);
    let p = plan(m.tier);
    match m.clip {
        Clip::Idle => {
            moves::idle(m, pose, &p, 0.7);
            let ready = m.aim.map_or(0.0, |a| a.weight);
            let inspect =
                tools::hold((m.clock * 0.24).fract(), [0.1, 0.22, 0.35, 0.55]) * (1.0 - ready);
            carry(pose, &p, 20.0 + 20.0 * ready + 8.0 * inspect, 0.03 * ready);
            tools::add(pose, Joint::Head, [12.0 * inspect, -8.0 * inspect, 0.0]);
        }
        Clip::Walk => {
            moves::walk(m, pose, 4.0, 4.0);
            let step = (m.walk.unwrap_or(0.0) * std::f32::consts::TAU).sin();
            carry(pose, &p, 18.0 + 3.0 * step.abs(), 0.008 * step.abs());
            tools::set(pose, Joint::Head, [-3.0, 2.0 * step, -3.0 * step]);
        }
        Clip::AttackRanged => fire(m, pose, &p, f),
        Clip::Hit => {
            moves::hit(m, pose, 0.6);
            carry(pose, &p, 10.0, 0.0);
        }
        Clip::Death => {
            moves::death(m, pose, &p, Fall::Back);
            carry(pose, &p, 30.0, 0.0);
        }
        Clip::Spawn => {
            moves::spawn(m, pose, &p, Entry::Drop);
            carry(pose, &p, 10.0, 0.0);
        }
        Clip::Evolve => {
            moves::evolve(m, pose, &p, Power::Burst);
            let up = 60.0 * tools::hold(m.t / 0.9, [0.3, 0.45, 0.75, 0.95]);
            carry(pose, &p, 20.0 + up, 0.12 * up / 60.0);
        }
        Clip::Victory => {
            let salute = 0.5 - 0.5 * (m.t / 0.8 * std::f32::consts::TAU).cos();
            tools::set(pose, Joint::Chest, [-4.0 * salute, 0.0, 2.0 * salute]);
            tools::set(pose, Joint::Head, [-14.0 * salute, -8.0, 0.0]);
            carry(pose, &p, 35.0 + 25.0 * salute, 0.025 * salute);
            tools::plant(pose, &p, tools::stance(&p, 1.3, 0.04));
        }
        _ => carry(pose, &p, 15.0, 0.0),
    }
}

/// The mortar at the hip in both hands, its barrel `up` degrees above
/// level, raised by `lift`.
fn carry(pose: &mut Pose, p: &Proportions, up: f32, lift: f32) {
    let turn = Quat::from_rotation_x(-up.to_radians());
    moves::shoulder_arms(pose, p, Vec3::new(-0.06, 0.11 + lift, 0.14), turn, FORE);
}

/// Brace and raise the barrel to the bomb's launch angle (0–0.15 s),
/// Release at 0.18 s with a recoil that shoves the whole body back, then
/// the left hand drops the next bomb down the barrel.
fn fire(m: &Moment, pose: &mut Pose, p: &Proportions, f: Form) {
    let k = (m.t / ATTACK).clamp(0.0, 1.0);
    let brace = tools::hold(k, [0.0, 0.22, 0.75, 1.0]);
    let kick = tools::snap(m.t, 0.18, 0.21) * (1.0 - tools::window(m.t, 0.22, 0.5));
    let up = moves::aim_level(m, pose, 0.7, shots::arc(ShotKind::Bomb)).max(0.55);
    tools::shift(pose, Joint::Hips, [0.0, -0.06 * brace, -0.07 * kick]);
    tools::add(pose, Joint::Spine, [6.0 * brace - 12.0 * kick, 0.0, 0.0]);
    tools::add(pose, Joint::Head, [-6.0 * kick, 0.0, 0.0]);
    let turn = Quat::from_rotation_x(-up * brace.max(0.4) - 0.3 * kick);
    let at = Vec3::new(-0.06, 0.11 + 0.04 * brace, 0.14) - turn * Vec3::Z * 0.07 * kick;
    moves::shoulder_arms(pose, p, at, turn, FORE);
    let load = tools::hold(k, [0.5, 0.66, 0.82, 0.96]);
    if load > 0.0 {
        let grip = at + turn * FORE;
        let mouth = at + turn * (model::muzzle(f) + Vec3::Y * 0.1);
        tools::reach(
            pose,
            true,
            grip.lerp(mouth, load),
            Some(turn),
            (p, Vec3::new(1.0, -0.4, -0.3)),
        );
    }
    tools::plant(pose, p, tools::stance(p, 1.5, 0.12));
}
