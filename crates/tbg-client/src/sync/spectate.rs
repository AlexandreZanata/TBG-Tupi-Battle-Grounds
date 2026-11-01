//! P47: a watcher's `PublicView` as the `MatchView` the battle scene already
//! draws. Only public fields carry over. The fields that are private to a
//! seat (Energy, regen, Shop offers, Evolution prices, `last_seq`) get fixed
//! neutral values, and the spectator HUD never shows them (BR-LV03). Seat A is
//! always "you", so the watched match is drawn in a neutral camera.

use tbg_core::content::OFFERS;
use tbg_core::view::RoundView;
use tbg_core::{MatchView, PublicView, Seat, UnitKind};

pub fn match_view(view: &PublicView) -> MatchView {
    MatchView {
        reserve: [0; tbg_core::content::OFFERS],
        held_mask: 0,
        reroll_cost: None,
        tick: view.tick,
        clock: view.clock,
        elapsed: view.elapsed,
        phase: view.phase,
        phase_ticks_left: view.phase_ticks_left,
        you: Seat::A,
        energy: 0,
        regen_bp: 10_000,
        evolve_costs: [0; 2],
        // Never shown: the spectator HUD hides the Shop.
        offers: [UnitKind::Shieldbearer; OFFERS],
        stronghold_hp: view.stronghold_hp,
        strongholds: view.strongholds,
        shields: view.shields,
        units: view.units.clone(),
        synergy: view.synergy,
        augments: view.augments,
        // Only seat A's flag fits the seat-shaped round; the HUD says "choosing".
        augment_round: view.round.as_ref().map(|r| RoundView {
            round: r.round,
            options: r.options,
            costs: r.costs,
            ticks_left: r.ticks_left,
            resolved: r.resolved[Seat::A.index()],
        }),
        last_seq: 0,
        events: view.events.clone(),
        outcome: view.outcome,
        spectators: view.spectators,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tbg_core::{ArcanaPreset, MatchState};

    #[test]
    fn the_watched_match_keeps_its_public_state_and_drops_seat_private_fields() {
        let mut state = MatchState::with_arcanas(5, [ArcanaPreset::NEUTRAL; 2]);
        let events = (0..40).flat_map(|_| state.step()).collect::<Vec<_>>();
        let public = state.public_view(&events, 4);
        let view = match_view(&public);
        assert_eq!(view.you, Seat::A);
        assert_eq!(view.tick, public.tick);
        assert_eq!(view.units, public.units);
        assert_eq!(view.events, public.events);
        assert_eq!(view.spectators, 4);
        assert_eq!(
            (view.energy, view.last_seq),
            (0, 0),
            "seat-private fields are neutral"
        );
        assert_eq!(view.evolve_costs, [0, 0]);
    }
}
