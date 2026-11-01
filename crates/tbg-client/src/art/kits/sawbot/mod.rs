//! Sawbot kit: a twitchy automaton whose saw never stops spinning (faster
//! when it cuts). It scuttles forward bent low, slashes the spinning saw
//! through its target in a spray of sparks, and topples on its face.

mod model;

use super::common::{moves, moves::*, Form};
use super::{tools, Attack, Gait, Kit, Moment};
use crate::art::{
    anim::{Clip, Pose},
    rig::{Joint, Node},
};
use crate::vfx::impacts::{Burst, Impact};
use model::plan;

pub const KIT: Kit = Kit {
    anatomy: None,
    figure: model::spec,
    pose,
    gait: Gait::LIGHT,
    attack: Attack::Melee {
        from: Node::Body(Joint::HandR),
        tip: model::tip,
        impact: |_, _| Impact::Sparks,
    },
    evolve: Burst::Forge,
    durations: moves::DURATIONS,
};

pub fn pose(m: &Moment, pose: &mut Pose) {
    let f = Form::of(m.tier, m.branch);
    let p = plan(m.tier);
    let mut rpm = 900.0;
    match m.clip {
        Clip::Idle => {
            moves::idle(m, pose, &p, 0.3);
            // Mechanical twitches of the head.
            let twitch = (m.clock * 0.8).sin().signum()
                * tools::hold((m.clock * 0.8).rem_euclid(1.0), [0.0, 0.03, 0.4, 0.43]);
            tools::add(pose, Joint::Head, [0.0, 25.0 * twitch, 0.0]);
            saw_ready(pose, 0.6);
        }
        Clip::Walk => {
            moves::walk(m, pose, 9.0, 3.0);
            saw_ready(pose, 0.9);
        }
        Clip::AttackMelee => {
            moves::strike(m, pose, &p, SLASH, 1.2);
            rpm = 2200.0;
        }
        Clip::Hit => moves::hit(m, pose, 0.9),
        Clip::Death => {
            moves::death(m, pose, &p, Fall::Forward);
            rpm = 900.0 * (1.0 - (m.t / 0.9).clamp(0.0, 1.0));
        }
        Clip::Spawn => moves::spawn(m, pose, &p, Entry::Drop),
        Clip::Evolve => moves::evolve(m, pose, &p, Power::Burst),
        Clip::Victory => moves::victory(m, pose, &p, Cheer::Dance),
        _ => saw_ready(pose, 0.4),
    }
    let spin = (m.clock * rpm).rem_euclid(360.0);
    tools::add(pose, Joint::HandR, [0.0, spin, 0.0]);
    if f.a && f.t2 {
        tools::add(pose, Joint::HandL, [0.0, -spin, 0.0]);
    }
}

/// The saw arm held out front (`w` 0..1).
fn saw_ready(pose: &mut Pose, w: f32) {
    tools::add(pose, Joint::UpperArmR, [-45.0 * w, 0.0, -10.0 * w]);
    tools::add(pose, Joint::ForearmR, [-40.0 * w, 0.0, 0.0]);
}
