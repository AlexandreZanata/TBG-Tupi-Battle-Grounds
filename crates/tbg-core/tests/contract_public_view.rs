//! P47 contract (BR-LV03, BR-S05, BR-AG14): the spectator's `PublicView`
//! carries no private state, a pending Augment pick stays hidden until its
//! round resolves, and the match's arena travels with it.

use tbg_core::content::ENERGY_SCALE;
use tbg_core::{ArcanaPreset, Arena, MatchState, PlayerIntention, Seat};

/// Combat ticks between the purchases and the draw: the armies meet.
const LEAD: u32 = 300;

/// A running match with both armies fighting when round 1 opens.
fn paused_battle(seed: u64) -> MatchState {
    let mut s = MatchState::with_arcanas(seed, [ArcanaPreset::NEUTRAL; 2]);
    while s.tick + LEAD < s.augments.next_open_at {
        s.step();
    }
    for p in &mut s.players {
        p.energy = 1_000 * ENERGY_SCALE;
    }
    for seq in 1..=4 {
        for seat in Seat::BOTH {
            s.apply_intention(seat, seq, PlayerIntention::Purchase { offer: 0 })
                .unwrap();
        }
    }
    while !s.augments.selecting() {
        s.step();
    }
    s
}

#[test]
fn private_state_never_reaches_the_public_view() {
    let mut a = MatchState::with_arcanas(7, [ArcanaPreset::NEUTRAL; 2]);
    for _ in 0..200 {
        a.step();
    }
    // Same battle, but both seats hold different Energy and a different
    // last applied sequence number: none of it may show in the public view.
    let mut b = a.clone();
    b.players[0].energy += 555 * ENERGY_SCALE;
    b.players[1].energy = 0;
    b.players[0].last_seq = 41;
    let events = a.step();
    b.step();
    let va = a.public_view(&events, 3);
    let vb = b.public_view(&events, 3);
    assert_eq!(va, vb, "BR-LV03: Energy and last_seq are not public");
    assert_eq!(va, a.public_view(&events, 3), "same state, same view");
}

#[test]
fn a_pending_pick_is_invisible_until_its_round_resolves() {
    let mut s = paused_battle(3);
    let round = s.augments.round;
    let before = s.public_view(&[], 0);
    assert_eq!(
        before.round.as_ref().map(|r| r.resolved),
        Some([false, false])
    );

    let seq = s.clock.saturating_add(1_000);
    s.apply_intention(
        Seat::A,
        seq,
        PlayerIntention::ChooseAugment { round, option: 0 },
    )
    .unwrap();
    let confirmed = s.public_view(&[], 0);
    // Only the resolved flag moves; the choice and the Augment copies stay put.
    assert_eq!(
        confirmed.round.as_ref().map(|r| r.resolved),
        Some([true, false])
    );
    assert_eq!(confirmed.augments, before.augments);
    assert_eq!(
        confirmed.round.as_ref().map(|r| r.options),
        before.round.as_ref().map(|r| r.options)
    );

    // Stepping until the round closes: no AugmentChosen before resolution.
    loop {
        let events = s.step();
        let view = s.public_view(&events, 0);
        let resolved = !s.augments.selecting();
        let chosen = events
            .iter()
            .any(|e| matches!(e, tbg_core::Event::AugmentChosen { .. }));
        assert!(
            resolved || !chosen,
            "a pick leaked before its round resolved"
        );
        if resolved {
            assert!(
                view.round.is_none(),
                "the public round closes with the pause"
            );
            assert!(chosen, "the confirmed pick becomes public at resolution");
            break;
        }
    }
}

#[test]
fn the_view_carries_the_match_arena_and_the_spectator_count() {
    for arena in Arena::ALL {
        let mut s = MatchState::with_arcanas(11, [ArcanaPreset::NEUTRAL; 2]).with_arena(arena);
        let events = s.step();
        let view = s.public_view(&events, 17);
        assert_eq!(view.arena, arena);
        assert_eq!(view.spectators, 17);
        assert_eq!(
            view.events, events,
            "every combat event is public (BR-LV03)"
        );
    }
}

#[cfg(feature = "lab")]
#[test]
fn champion_display_keeps_protocol20_carrier_and_expiring_shield() {
    // D-61/D-63 + D-68: both views preserve carrier/shield while adding fx display bits.
    let mut state = MatchState::lab(
        31,
        tbg_core::Ruleset::canon(),
        [tbg_core::ArcanaPreset::NEUTRAL; 2],
    );
    let carrier = state.lab_spawn_at(Seat::A, tbg_core::UnitKind::CargoCrab, 0, 0, 4_000);
    let rider = state.lab_spawn_at(Seat::A, tbg_core::UnitKind::Leechroot, 0, 0, 4_000);
    let index = state.unit_index(rider).unwrap();
    state.units[index].fx.carrier = Some(carrier);
    state.units[index].fx.plates = 3;
    state.units[index].fx.essence = 5;
    state.units[index].fx.shield = 90;
    state.units[index].fx.shield_until = 5;
    let player = state.view_for(Seat::A, &[]);
    let public = state.public_view(&[], 0);
    assert_eq!(player.units, public.units);
    let unit = player.units.iter().find(|u| u.id == rider).unwrap();
    assert_eq!((unit.carrier, unit.shield), (Some(carrier), 90));
    assert!(unit.is_carried());
    assert_eq!(((unit.fx >> 8) & 7, (unit.fx >> 11) & 7), (3, 5));
    state.tick = 5;
    assert_eq!(state.view_for(Seat::A, &[]).units[index].shield, 0);
    assert_eq!(state.public_view(&[], 0).units[index].shield, 0);
}
