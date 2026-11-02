//! Old Finn's raincoat, creel and rod kit; fish sizes are cosmetic while release timing stays fixed.

mod model;
mod motion;

use super::common::moves;
use super::{Attack, Gait, Kit};
use crate::art::rig::{Node, Socket};
use crate::vfx::{impacts::Burst, shots::ShotKind};
use model::TIP;

pub const KIT: Kit = Kit {
    anatomy: None,
    figure: model::spec,
    pose: motion::pose,
    gait: Gait {
        stride: 0.98,
        lift: 0.16,
        bob: 0.018,
        sink: 0.11,
        stomp: false,
    },
    attack: Attack::Shot {
        loaded: None,
        from: Node::Prop(Socket::WeaponR),
        muzzle: |_, _| TIP,
        shot: |tier, _| ShotKind::Fish(tier.min(2)),
        flash: |_, _| None,
    },
    evolve: Burst::Valor,
    durations: moves::DURATIONS,
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::art::{
        anim::{Clip, ClipEvent},
        curves::events_between,
        FORMS,
    };

    #[test]
    fn old_finn_all_forms_select_growing_fish() {
        // P70 S2: both branches carry their visual tier; Cast still releases at 0.22 s.
        let Attack::Shot { shot, muzzle, .. } = KIT.attack else {
            panic!("Finn must cast fish")
        };
        for (tier, branch) in FORMS {
            assert_eq!(shot(tier, branch), ShotKind::Fish(tier));
            assert_eq!(muzzle(tier, branch), TIP);
        }
        assert_eq!(events_between(Clip::Cast, 0.21, 0.22), [ClipEvent::Release]);
        assert_eq!(KIT.duration(Clip::Cast), moves::ATTACK);
    }
}
