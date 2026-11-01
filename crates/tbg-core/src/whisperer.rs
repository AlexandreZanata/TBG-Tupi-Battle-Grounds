//! R11's fear pulse applies the ordinary bounded Slow, never damage or redirection.

use crate::{
    content::Ability,
    hits,
    state::{Event, MatchState, Unit},
    types::{OnHit, UnitId},
};

pub(crate) fn resolve(state: &mut MatchState, caster: &Unit, primary: UnitId, ev: &mut Vec<Event>) {
    for ability in state
        .rules()
        .def(caster.kind)
        .ability(caster.tier, caster.branch)
    {
        if let Ability::FearPulse { bp, ticks, .. } = ability {
            let _applied =
                hits::apply_on_hit(state, primary, OnHit::Slow { bp, ticks }, caster.seat, ev);
            #[cfg(feature = "lab")]
            crate::lab::status(state, caster, _applied);
        }
    }
}
