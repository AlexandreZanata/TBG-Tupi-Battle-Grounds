//! Bone Brute kit: hunched giant skeleton; alternating heavy swings per
//! Evolution, ground impacts at the blade, Reaper's crescent sweep.

mod model;
mod motion;

use super::{Attack, Gait, Kit};
use crate::art::{
    anim::Clip,
    rig::{Node, Socket},
};
use crate::vfx::impacts::Impact;
use model::Form;

pub const KIT: Kit = Kit {
    figure: model::spec,
    pose: motion::pose,
    gait: Gait::HEAVY,
    attack: Attack::Melee {
        from: Node::Prop(Socket::WeaponR),
        tip: |tier, branch| model::tip(Form::of(tier, branch)),
        impact,
    },
    evolve: crate::vfx::impacts::Burst::Soul,
    durations: &[(Clip::Evolve, 0.9), (Clip::AttackMelee, motion::ATTACK), (Clip::Spawn, 0.6)],
};

fn impact(tier: u8, branch: u8) -> Impact {
    let f = Form::of(tier, branch);
    match (f.a, f.b, f.t2) {
        (true, _, true) => Impact::Crush,
        (_, true, false) => Impact::Butcher,
        (_, true, true) => Impact::Reap,
        _ => Impact::Cleave,
    }
}
