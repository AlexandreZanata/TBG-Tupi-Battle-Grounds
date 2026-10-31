//! Siege Walker kit: ogre gunner with a shoulder bombard that aims along
//! its own shell's flight; five forms with their own shells and muzzles.

mod model;
mod motion;

use super::{Attack, Flash, Gait, Kit};
use crate::art::{
    anim::Clip,
    rig::{Node, Socket},
};
use crate::vfx::{impacts::Muzzle, shots::ShotKind};
use model::{bombard, Form};

pub const KIT: Kit = Kit {
    anatomy: None,
    figure: model::spec,
    pose: motion::pose,
    gait: Gait::HEAVY,
    attack: Attack::Shot {
        loaded: None,
        from: Node::Prop(Socket::WeaponR),
        muzzle: |tier, branch| bombard(Form::of(tier, branch)).muzzle,
        shot,
        flash,
    },
    // Large: a ×1.25 recovery; Release stays at 0.18 s.
    evolve: crate::vfx::impacts::Burst::Forge,
    durations: &[
        (Clip::Evolve, 0.9),
        (Clip::AttackRanged, 0.62),
        (Clip::Spawn, 0.5),
    ],
};

/// Base shell, Longshot long shell, Artillery great shell, Shrapnel
/// burst, Concussion hammer.
pub fn shot(tier: u8, branch: u8) -> ShotKind {
    let f = Form::of(tier, branch);
    match (f.a, f.b, f.t2) {
        (true, _, false) => ShotKind::LongShell,
        (true, _, true) => ShotKind::GreatShell,
        (_, true, false) => ShotKind::Shrapnel,
        (_, true, true) => ShotKind::Hammer,
        _ => ShotKind::Shell,
    }
}

fn flash(tier: u8, branch: u8) -> Flash {
    let f = Form::of(tier, branch);
    let muzzle = match (f.a, f.b, f.t2) {
        (true, _, true) => Muzzle::GreatBombard,
        (_, true, false) => Muzzle::Blunderbuss,
        _ => Muzzle::Bombard,
    };
    Some((muzzle, Some(bombard(f).breech)))
}
