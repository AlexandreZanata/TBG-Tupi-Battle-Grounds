//! Nester Hatch and summon lifetime (BR-U08). These limits belong to the
//! ability, never to the army: bought units stay uncapped (BR-U07, D-32).

use crate::augment;
use crate::content::BEETLES_PER_PLAYER;
use crate::state::{Event, MatchState, Unit};
use crate::types::{Status, UnitId, UnitKind};

/// Summons whose lifetime ended drop to 0 HP; `remove_dead` reports them.
pub(crate) fn expire(state: &mut MatchState) {
    let tick = state.tick;
    for u in &mut state.units {
        if u.expires_at.is_some_and(|end| tick >= end) {
            u.hp = 0;
        }
    }
}

/// Hard safety bound for the summon entity count per seat. Augment copies
/// raise the gameplay limit up to this fixed array size (no per-tick alloc).
const MAX_SUMMONS: usize = 64;

/// The living summons of one seat, by summoner.
struct Brood {
    by: [UnitId; MAX_SUMMONS],
    len: usize,
}

impl Default for Brood {
    fn default() -> Brood {
        Brood {
            by: [0; MAX_SUMMONS],
            len: 0,
        }
    }
}

impl Brood {
    fn of(&self, nester: UnitId) -> usize {
        self.by[..self.len]
            .iter()
            .filter(|&&id| id == nester)
            .count()
    }

    fn push(&mut self, nester: UnitId, limit: usize) {
        if self.len < limit.min(MAX_SUMMONS) {
            self.by[self.len] = nester;
            self.len += 1;
        }
    }
}

fn limit(state: &MatchState, seat: crate::types::Seat) -> usize {
    let add = augment::summon_limit_add(&state.player(seat).augments);
    (BEETLES_PER_PLAYER + usize::from(add)).min(MAX_SUMMONS)
}

/// Each living Nester, in id order, hatches when its timer is done and both
/// limits allow; a blocked Nester stays ready and hatches once space frees.
pub(crate) fn hatch(state: &mut MatchState, ev: &mut Vec<Event>) {
    let mut brood = [Brood::default(), Brood::default()];
    for u in state.units.iter().filter(|u| u.hp > 0) {
        if let Some(nester) = u.summoner {
            brood[u.seat.index()].push(nester, MAX_SUMMONS);
        }
    }
    for idx in 0..state.units.len() {
        let n = state.units[idx];
        if n.kind != UnitKind::Nester || n.hp <= 0 {
            continue;
        }
        // Count down and hatch on the same tick: exactly `every` ticks apart.
        let cd = n.action_cd.saturating_sub(1);
        state.units[idx].action_cd = cd;
        if cd > 0 {
            continue;
        }
        let (every, per_nester) = state.rules.hatch_rule(n.tier, n.branch);
        let cap = limit(state, n.seat);
        let seat_brood = &mut brood[n.seat.index()];
        if seat_brood.len >= cap || seat_brood.of(n.id) >= per_nester {
            continue;
        }
        seat_brood.push(n.id, cap);
        state.units[idx].action_cd = every;
        spawn_beetling(state, &n, ev);
    }
}

/// A Beetling takes its Nester's tier and branch (its Venom/Blight forms)
/// and position; ids keep growing, so `units` stays sorted by id.
fn spawn_beetling(state: &mut MatchState, nester: &Unit, ev: &mut Vec<Event>) {
    let kind = UnitKind::Beetling;
    let bonus = augment::unit_bonus(
        &state.player(nester.seat).augments,
        state.player(nester.seat).synergy,
        kind,
        nester.tier,
    );
    let max_hp = state
        .rules
        .stats_with(
            kind,
            nester.tier,
            nester.branch,
            0,
            &crate::arcana::ArcanaPreset::NEUTRAL,
            &bonus,
        )
        .max_hp;
    let id = state.next_unit_id;
    state.next_unit_id += 1;
    state.units.push(Unit {
        id,
        seat: nester.seat,
        kind,
        tier: nester.tier,
        branch: nester.branch,
        x: nester.x,
        hp: max_hp,
        max_hp,
        cooldown: 0,
        windup: None,
        status: Status::None,
        summoner: Some(nester.id),
        expires_at: Some(state.tick + state.rules.beetle_lifetime),
        action_cd: 0,
        haste_until: 0,
        heal_cut_until: 0,
        heal_cut_bp: 0,
        second_wind_used: 0,
    });
    ev.push(Event::Spawn {
        unit: id,
        seat: nester.seat,
        kind,
    });
}
