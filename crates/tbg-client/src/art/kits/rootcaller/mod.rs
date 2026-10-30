//! Rootcaller kit: treant shaman whose cast drives its claws into the
//! ground; a root ridge runs to the target and roots erupt around it for
//! the real root duration; spores on the B branch.

mod model;
mod motion;

use super::{Attack, Gait, Kit};
use crate::art::{
    anim::Clip,
    rig::{Node, Socket},
};
use crate::vfx::shots::ShotKind;
use model::Form;

pub const KIT: Kit = Kit {
    figure: model::spec,
    pose: motion::pose,
    gait: Gait::LIGHT,
    attack: Attack::Shot {
        from: Node::Prop(Socket::WeaponR),
        muzzle: |tier, branch| model::tips(Form::of(tier, branch)),
        shot,
        flash: |_, _| None,
    },
    evolve: crate::vfx::impacts::Burst::Bloom,
    durations: &[(Clip::Evolve, 0.9), (Clip::Spawn, 0.55)],
};

fn shot(tier: u8, branch: u8) -> ShotKind {
    let f = Form::of(tier, branch);
    match (f.a, f.b, f.t2) {
        (true, _, true) => ShotKind::Strangle,
        (_, true, _) => ShotKind::Spores,
        _ => ShotKind::Roots,
    }
}
