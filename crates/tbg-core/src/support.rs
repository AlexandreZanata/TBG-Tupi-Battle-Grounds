//! Ally-support actions (P46): Bloodweaver transfers. Drumguard bonds and
//! Cargo Crab boarding join in later waves (ROSTER-EXPANSION §F.3).

use std::cmp::Reverse;

use crate::afflictions::abilities_of;
use crate::content::Ability;
use crate::state::{Event, MatchState, Unit};
use crate::types::UnitId;

/// The attacker's Transfer params, if any (absolute per form slot, like Brew).
pub(crate) fn transfer_of(state: &MatchState, attacker: &Unit) -> Option<(u16, u16, bool)> {
    abilities_of(state, attacker)
        .into_iter()
        .find_map(|ability| match ability {
            Ability::Transfer {
                amount,
                low_bp,
                second,
            } => Some((amount, low_bp, second)),
            _ => None,
        })
}

/// The transfer pair (§C.14): among allied **conventional** units in range
/// (itself included; summons excluded as donors and receivers), the donor
/// is the highest HP % and the receiver the lowest HP % below full; they
/// must be different units and the donor's % higher. Ties: nearest, then
/// lowest id. `exclude` skips two units (the Double Flask second transfer).
pub(crate) fn transfer_pair(
    state: &MatchState,
    u: &Unit,
    range: i32,
    exclude: [Option<UnitId>; 2],
) -> Option<(UnitId, UnitId, i32)> {
    let mut donor: Option<(i32, i32, UnitId, i32)> = None;
    let mut receiver: Option<(i32, i32, UnitId, i32)> = None;
    for a in state.units.iter().filter(|a| {
        a.seat == u.seat
            && a.hp > 0
            && !a.is_summon()
            && Some(a.id) != exclude[0]
            && Some(a.id) != exclude[1]
            && (a.x - u.x).abs() <= range
    }) {
        // `pct` in basis points of max HP; the receiver also tracks HP
        // missing below full.
        let pct = (i64::from(a.hp) * 10_000 / i64::from(a.max_hp.max(1))) as i32;
        let dist = (a.x - u.x).abs();
        // Donor: highest %, then nearest, then lowest id.
        let better_donor = match donor {
            None => true,
            Some((p, d, id, _)) => (pct, Reverse(dist), Reverse(id)) > (p, Reverse(d), Reverse(id)),
        };
        if better_donor {
            donor = Some((pct, dist, a.id, a.hp - 1));
        }
        // Receiver: lowest % below full, then nearest, then lowest id.
        if a.hp < a.max_hp {
            let better_receiver = match receiver {
                None => true,
                Some((p, d, id, _)) => (pct, dist, id) < (p, d, id),
            };
            if better_receiver {
                receiver = Some((pct, dist, a.id, a.max_hp - a.hp));
            }
        }
    }
    let (dpct, _, donor_id, donor_headroom) = donor?;
    let (rpct, _, receiver_id, missing) = receiver?;
    if donor_id == receiver_id || dpct <= rpct {
        return None;
    }
    // A pair that would move nothing is no pair: the weaver waits ready.
    let headroom = donor_headroom.min(missing);
    if headroom < 1 {
        return None;
    }
    Some((donor_id, receiver_id, headroom))
}

/// The Bloodweaver's cast (§C.14): the donor loses `amount` HP directly (no
/// reduction, plates or Second Wind; never below 1 HP); the receiver gains
/// it minus heal cut (Grievous Wounds, Deep Wound turn transfers into a net
/// loss). Not a heal (no Arcana/Augment bonus, no Purification) and not
/// damage (no on-hit, lifesteal, Essence).
pub(crate) fn transfer_cast(
    state: &mut MatchState,
    attacker: &Unit,
    range: i32,
    amount_param: u16,
    low_bp: u16,
    second: bool,
    ev: &mut Vec<Event>,
) {
    if let Some((donor, receiver, headroom)) = transfer_pair(state, attacker, range, [None, None]) {
        transfer_one(state, donor, receiver, amount_param, low_bp, headroom, ev);
        if second {
            if let Some((donor2, receiver2, headroom2)) =
                transfer_pair(state, attacker, range, [Some(donor), Some(receiver)])
            {
                transfer_one(
                    state,
                    donor2,
                    receiver2,
                    amount_param,
                    low_bp,
                    headroom2,
                    ev,
                );
            }
        }
    }
}

/// One donor→receiver move. `headroom` is min(donor HP − 1, receiver missing
/// HP) at selection time; the A2 double is capped by it so donors survive.
fn transfer_one(
    state: &mut MatchState,
    donor: UnitId,
    receiver: UnitId,
    amount_param: u16,
    low_bp: u16,
    headroom: i32,
    ev: &mut Vec<Event>,
) {
    let Some(di) = state.unit_index(donor) else {
        return;
    };
    let Some(ri) = state.unit_index(receiver) else {
        return;
    };
    let mut amount = i32::from(amount_param).min(headroom);
    let receiver_pct =
        i64::from(state.units[ri].hp) * 10_000 / i64::from(state.units[ri].max_hp.max(1));
    if low_bp > 0 && receiver_pct < i64::from(low_bp) {
        // Last Offering: double, still capped so the donor survives.
        amount = (amount * 2).min(state.units[di].hp - 1);
    }
    if amount <= 0 {
        return;
    }
    state.units[di].hp -= amount;
    let cut = if state.tick < state.units[ri].heal_cut_until {
        state.units[ri].heal_cut_bp
    } else {
        0
    };
    let gain = amount - amount * cut.min(10_000) / 10_000;
    let u = &mut state.units[ri];
    u.hp = (u.hp + gain).min(u.max_hp);
    ev.push(Event::Transfer {
        from: donor,
        to: receiver,
        amount,
    });
}
