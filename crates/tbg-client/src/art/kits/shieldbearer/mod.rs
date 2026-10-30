//! Shieldbearer kit: a man-at-arms who keeps his shield up, marches behind
//! it, and cuts round its edge with his sword; he falls back, strides in
//! and raises the blade when he evolves or wins.

mod model;

use super::common::{moves, moves::*, Form};
use super::{tools, Attack, Gait, Kit, Moment};
use crate::art::{
    anim::{Clip, Pose},
    rig::{Joint, Node, Socket},
};
use crate::vfx::impacts::{Burst, Impact};
use model::plan;

pub const KIT: Kit = Kit {
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
            shield_up(pose, 0.5 + 0.5 * ready);
            moves::guard(pose, 0.3 * ready);
        }
        Clip::Walk => {
            moves::walk(m, pose, 4.0, 3.0);
            shield_up(pose, 0.6);
        }
        Clip::AttackMelee => {
            moves::strike(m, pose, &p, SLASH, 1.3);
            shield_up(pose, 1.0);
        }
        Clip::Hit => {
            moves::hit(m, pose, 0.6);
            shield_up(pose, 1.0);
        }
        Clip::Death => moves::death(m, pose, &p, Fall::Back),
        Clip::Spawn => moves::spawn(m, pose, &p, Entry::March),
        Clip::Evolve => moves::evolve(m, pose, &p, Power::Raise),
        Clip::Victory => moves::victory(m, pose, &p, Cheer::Raise),
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
