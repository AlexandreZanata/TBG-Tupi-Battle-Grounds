//! Shieldbearer kit: a man-at-arms who keeps his shield up, marches behind
//! it, and cuts round its edge with his sword; he falls back, strides in
//! and raises the blade when he evolves or wins.

mod model;

use bevy::prelude::*;

use super::common::{moves, moves::*, Form};
use super::{tools, Attack, Gait, Kit, Moment};
use crate::art::{
    anim::{Clip, Pose},
    rig::{Joint, Node, Socket},
};
use crate::vfx::impacts::{Burst, Impact};
use model::plan;

pub const KIT: Kit = Kit {
    anatomy: None,
    figure: model::spec,
    pose,
    gait: Gait::LIGHT,
    attack: Attack::Melee {
        from: Node::Prop(Socket::WeaponR),
        tip: |tier, branch| model::tip(Form::of(tier, branch)),
        impact: |tier, branch| {
            if Form::of(tier, branch).b {
                Impact::Cleave
            } else {
                Impact::Crush
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
            moves::idle(m, pose, &p, 0.5);
            let ready = m.aim.map_or(0.0, |a| a.weight);
            let check = tools::hold((m.clock * 0.25).rem_euclid(1.0), [0.15, 0.30, 0.42, 0.60])
                * (1.0 - ready);
            shield_up(pose, 0.42 + 0.38 * ready + 0.10 * check);
            tools::add(pose, Joint::Head, [7.0 * check, 14.0 * check, 0.0]);
            moves::guard(pose, 0.3 * ready);
        }
        Clip::Walk => {
            moves::walk(m, pose, 4.0, 3.0);
            let step = (m.walk.unwrap_or(0.0) * std::f32::consts::TAU).sin();
            shield_up(pose, 0.48 + 0.05 * step);
            tools::set(pose, Joint::UpperArmR, [-18.0, 0.0, -12.0]);
            tools::set(pose, Joint::ForearmR, [-58.0, 0.0, 0.0]);
            tools::add(pose, Joint::Chest, [0.0, 2.0 * step, -2.0 * step]);
        }
        Clip::AttackMelee => {
            moves::strike(m, pose, &p, SLASH, 1.3);
            let brace = tools::hold(m.t, [0.0, 0.14, 0.34, 0.62]);
            shield_up(pose, 0.48 + 0.25 * brace);
            tools::add(pose, Joint::Head, [-3.0 * brace, -8.0 * brace, 0.0]);
            tools::add(
                pose,
                Joint::HandR,
                [0.0, -10.0 * tools::hold(m.t, [0.18, 0.30, 0.36, 0.62]), 0.0],
            );
        }
        Clip::Hit => {
            moves::hit(m, pose, 0.6);
            shield_up(pose, 1.0);
        }
        Clip::Death => moves::death(m, pose, &p, Fall::Back),
        Clip::Spawn => moves::spawn(m, pose, &p, Entry::March),
        Clip::Evolve => moves::evolve(m, pose, &p, Power::Raise),
        Clip::Victory => salute(m, pose, &p),
        _ => shield_up(pose, 0.5),
    }
}

/// The shield arm brought across the front (`w` 0..1).
fn shield_up(pose: &mut Pose, w: f32) {
    tools::set(
        pose,
        Joint::UpperArmL,
        [-20.0 - 35.0 * w, 0.0, 10.0 + 8.0 * w],
    );
    tools::set(pose, Joint::ForearmL, [-30.0 - 50.0 * w, 0.0, 0.0]);
    tools::set(pose, Joint::HandL, [0.0, -50.0 * w, 0.0]);
}

/// A grounded shield salute: the blade approaches the outside rim, then withdraws.
fn salute(m: &Moment, pose: &mut Pose, p: &crate::art::rig::Proportions) {
    pose.fill((Vec3::ZERO, Vec3::ZERO));
    let lift = tools::hold(m.t.rem_euclid(0.8), [0.0, 0.18, 0.42, 0.76]);
    let tap = tools::hold(m.t.rem_euclid(0.8), [0.20, 0.30, 0.34, 0.52]);
    shield_up(pose, 0.35 + 0.25 * lift);
    tools::set(
        pose,
        Joint::UpperArmR,
        [-35.0 - 25.0 * lift, 0.0, -28.0 + 18.0 * tap],
    );
    tools::set(pose, Joint::ForearmR, [-62.0 + 20.0 * tap, 0.0, 0.0]);
    tools::set(pose, Joint::HandR, [0.0, -12.0 * tap, 0.0]);
    tools::set(pose, Joint::Head, [6.0 * tap, 8.0 * lift, 0.0]);
    tools::plant(pose, p, tools::stance(p, 1.2, 0.04));
}
