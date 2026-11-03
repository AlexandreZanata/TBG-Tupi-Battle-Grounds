//! Seat bots: they think twice per second and act only through `MobaIntention`s (M-26).

use crate::content::{self, MAX_TIER, TROOP_CAP};
use crate::intent::MobaIntention;
use crate::sim::spawn::nearest_camp;
use crate::state::MobaMatch;
use crate::types::{Destination, Lane, Role, Seat, TroopKind};

/// Troops each role prefers, tried in rotation.
fn preferences(role: Role) -> &'static [TroopKind] {
    use TroopKind::*;
    match role {
        Role::Top => &[Guardian, Colossus, Summoner],
        Role::Jungle => &[Guardian, Summoner, Healer],
        Role::Mid => &[Summoner, Bard, Guardian],
        Role::Bot => &[Guardian, Summoner, Bard],
        Role::Support => &[Healer, Bard, Guardian],
    }
}

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
    let prefs = preferences(seat.role());
    // Same role, same habits on both teams; the seed varies them per match.
    let flavour = (m.seed >> (seat.team().index() * 8)) as u32 % 3;
    let start = (m.tick / 10 + seat.role() as u32 + flavour) as usize;
    let troop = prefs[start % prefs.len()];
    (p.gold >= content::troop_cost(troop)).then(|| MobaIntention::BuyTroop {
        troop,
        to: destination(m, seat),
    })
}
