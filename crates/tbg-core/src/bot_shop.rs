//! Conservative preview Shop choices from the same private view a player sees.
use crate::{content::OFFERS, MatchView, PlayerIntention, Ruleset, UnitKind};

pub(crate) fn decide(
    view: &MatchView,
    rules: &Ruleset,
    reserve: bool,
    reroll: bool,
    rolls: u8,
    wanted: impl Fn(UnitKind) -> bool,
) -> Option<PlayerIntention> {
    let rule = rules.shop?;
    if view.augment_round.is_some() {
        return None;
    }
    if reserve && view.reserve.iter().filter(|&&n| n > 0).count() < usize::from(rule.reserve_slots)
    {
        let slot = (0..OFFERS)
            .filter(|&i| {
                view.held_mask & (1 << i) == 0
                    && wanted(view.offers[i])
                    && rules.def(view.offers[i]).cost > view.energy
            })
            .max_by_key(|&i| (rules.def(view.offers[i]).cost, std::cmp::Reverse(i)));
        if let Some(i) = slot {
            return Some(PlayerIntention::Reserve {
                offer: i as u8,
                kind: view.offers[i],
            });
        }
    }
    let useful = view
        .offers
        .iter()
        .any(|&k| wanted(k) && rules.def(k).cost <= view.energy);
    let cheapest = rules.pool.iter().map(|&k| rules.def(k).cost).min()?;
    let cost = view.reroll_cost?;
    (reroll && !useful && rolls < 2 && view.energy >= cost.saturating_add(cheapest))
        .then_some(PlayerIntention::Reroll { cost })
}
