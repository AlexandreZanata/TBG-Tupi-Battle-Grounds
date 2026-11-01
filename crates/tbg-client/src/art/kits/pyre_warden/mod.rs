//! Pyre Warden's hunched drag, braced nozzle pulse and furnace fold; authoritative flame is a separate event.
mod model;
mod motion;
use super::{Attack, Gait, Kit};
use crate::{art::anim::Clip, vfx::impacts::Burst};
use bevy::prelude::*;
pub const KIT: Kit = Kit {
    anatomy: None,
    figure: model::spec,
    pose: motion::pose,
    gait: Gait::HEAVY,
    attack: Attack::None,
    evolve: Burst::Forge,
    // Its canonical basic repeats every 0.50s; preserve release timing and shorten only recovery.
    durations: &[
        (Clip::AttackMelee, 0.48),
        (Clip::AttackRanged, 0.48),
        (Clip::Cast, 0.48),
        (Clip::Spawn, 0.6),
        (Clip::Evolve, 0.9),
        (Clip::Death, 0.9),
    ],
};
/// The visible bore, in WeaponR space; event effects share the model's dimensions.
pub(crate) fn muzzle((tier, branch): (u8, u8)) -> Vec3 {
    model::muzzle(super::common::Form::of(tier, branch))
}
