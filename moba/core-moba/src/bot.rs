//! Seat bots: they think twice per second and act only through `MobaIntention`s (M-26).

use crate::content::{self, MAX_TIER, TROOP_CAP};
use crate::intent::MobaIntention;
use crate::sim::spawn::nearest_camp;
use crate::state::MobaMatch;
use crate::types::{Champion, Destination, Lane, Seat};

fn destination(m: &MobaMatch, seat: Seat) -> Destination {
    match seat.role().lane() {
        Some(lane) => Destination::Lane(lane),
        None => {
            let base = crate::map::stronghold(seat.team());
            if nearest_camp(m, base, Some(seat.team())).is_some() {
                Destination::Jungle
            } else {
                Destination::Lane(Lane::Mid)
            }
        }
    }
}

/// The bot's next intention for `seat`, if any. Pure: reads the match only.
pub fn decide(m: &MobaMatch, seat: Seat) -> Option<MobaIntention> {
    if m.outcome.is_some() || !(m.tick + seat.0 as u32 * 3).is_multiple_of(10) {
        return None;
    }
    let p = m.player(seat);
    if p.tier < MAX_TIER && p.essence >= content::upgrade_cost(p.tier) {
        return Some(MobaIntention::Upgrade);
    }
    if m.troops_alive(seat) >= TROOP_CAP {
        return None;
    }
    let affordable = |k: &&Champion| p.gold >= content::troop_cost(**k);
    let wanted = content::slot_rules(seat.role())[0][0];
    let pick = p
        .offers
        .iter()
        .position(|k| affordable(&k) && tbg_core::content::def(*k).archetype == wanted)
        .or_else(|| p.offers.iter().position(|k| affordable(&k)));
    match pick {
        Some(slot) => Some(MobaIntention::BuyTroop {
            slot: slot as u8,
            to: destination(m, seat),
        }),
        // Nothing worth buying and plenty of gold: reroll like a 1v1 player.
        None if p.gold >= 8 * content::REROLL_COST => Some(MobaIntention::Reroll),
        None => None,
    }
}
