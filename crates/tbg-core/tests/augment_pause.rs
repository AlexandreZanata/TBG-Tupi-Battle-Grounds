//! Owner revision 2026-10-29: every Augment round pauses the battle for
//! both seats (BR-AG12), only the selection clock runs until both confirm
//! or the window ends (BR-AG13), and a confirmed pick stays hidden until
//! the round resolves (BR-AG14).

use tbg_core::content::{
    augment_def, AUGMENT_COST, AUGMENT_ROUND_TICKS, AUGMENT_WINDOW_TICKS, ENERGY_SCALE,
};
use tbg_core::replay::Replay;
use tbg_core::{
    ArcanaPreset, Event, MatchState, Phase, PlayerIntention, Rejection, Seat, Unit, UnitKind,
};

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
        p.low = p.energy;
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

fn choose(s: &mut MatchState, seat: Seat, option: u8) -> Result<(), Rejection> {
    let round = s.augments.round;
    let seq = s.clock.saturating_add(1_000);
    s.apply_intention(seat, seq, PlayerIntention::ChooseAugment { round, option })
}

/// Everything combat, the economy, or the Shop could move.
fn frozen_part(s: &MatchState) -> (u32, Vec<Unit>, String) {
    let players: Vec<_> = s
        .players
        .iter()
        .map(|p| {
            (
                p.energy,
                p.low,
                p.interest_paid,
                p.offers,
                p.stronghold_hp,
                p.stronghold_cooldown,
                p.shield,
                p.synergy,
            )
        })
        .collect();
    (s.tick, s.units.clone(), format!("{players:?}"))
}

#[test]
fn br_ag12_a_round_freezes_units_attacks_cooldowns_and_the_economy() {
    let mut s = paused_battle(1);
    for seat in Seat::BOTH {
        assert!(s.units.iter().any(|u| u.seat == seat && u.hp > 0));
    }
    assert!(
        s.units.iter().any(|u| u.cooldown > 0 || u.windup.is_some()),
        "attacks are in flight when the pause starts"
    );
    let before = frozen_part(&s);
    let clock = s.clock;
    for _ in 0..300 {
        assert!(s.step().is_empty(), "a paused step emits nothing");
    }
    assert!(s.augments.selecting());
    assert_eq!(s.clock, clock + 300, "the selection clock keeps running");
    assert_eq!(frozen_part(&s), before, "combat, Energy, Interest, Shop");
    // Both seats see the same paused round at the same combat tick.
    let (a, b) = (s.view_for(Seat::A, &[]), s.view_for(Seat::B, &[]));
    assert_eq!((a.tick, a.clock), (b.tick, b.clock));
    assert_eq!(a.augment_round, b.augment_round);
}

#[test]
fn br_ag12_gameplay_intentions_are_rejected_while_paused() {
    let mut s = paused_battle(2);
    let unit = s.units.iter().find(|u| u.seat == Seat::A).unwrap().id;
    assert_eq!(
        s.apply_intention(Seat::A, 50, PlayerIntention::Purchase { offer: 0 }),
        Err(Rejection::Paused)
    );
    assert_eq!(
        s.apply_intention(Seat::B, 50, PlayerIntention::Evolve { unit, branch: 0 }),
        Err(Rejection::Paused)
    );
}

#[test]
fn br_ag13_one_confirmation_never_resumes_the_battle() {
    let mut s = paused_battle(3);
    let tick = s.tick;
    choose(&mut s, Seat::A, 0).unwrap();
    for _ in 0..300 {
        s.step();
    }
    assert!(s.augments.selecting());
    assert_eq!(s.tick, tick);
    let a = s.view_for(Seat::A, &[]);
    assert!(a.augment_round.unwrap().resolved, "A sees its confirmation");
}

#[test]
fn br_ag13_both_confirmations_resume_at_once_for_both() {
    let mut s = paused_battle(4);
    let tick = s.tick;
    choose(&mut s, Seat::A, 0).unwrap();
    s.step();
    s.apply_intention(
        Seat::B,
        900,
        PlayerIntention::Forgo {
            round: s.augments.round,
        },
    )
    .unwrap();
    s.step();
    assert!(!s.augments.selecting(), "resolved on the next step");
    assert_eq!(s.tick, tick, "the resolving step runs no combat");
    s.step();
    assert_eq!(s.tick, tick + 1, "combat resumes on the following step");
}

#[test]
fn br_ag13_the_deadline_resumes_with_an_idle_player() {
    let mut s = paused_battle(5);
    let window = s.augments.deadline - s.clock;
    assert_eq!(window, AUGMENT_WINDOW_TICKS, "20 s of selection time");
    assert_eq!(window, 20 * 20);
    let id = s.augments.options[1];
    choose(&mut s, Seat::A, 1).unwrap();
    let mut events = Vec::new();
    let mut steps = 0;
    while s.augments.selecting() {
        events.extend(s.step());
        steps += 1;
    }
    assert_eq!(steps, window, "closed exactly at the deadline");
    assert!(events.contains(&Event::AugmentChosen { seat: Seat::A, id }));
    assert!(events.contains(&Event::Forgone {
        seat: Seat::B,
        round: 1
    }));
    assert_eq!(s.players[0].copies(id), 1, "a confirmed pick survives");
    assert_eq!(s.players[1].augments.iter().sum::<u16>(), 0);
}

#[test]
fn br_ag14_a_pick_stays_hidden_until_the_round_resolves() {
    let mut s = paused_battle(6);
    let id = s.augments.options[2];
    choose(&mut s, Seat::A, 2).unwrap();
    let events = s.step();
    let b = s.view_for(Seat::B, &events);
    assert!(!b.augment_round.as_ref().unwrap().resolved);
    assert_eq!(b.augments[Seat::A.index()].get(usize::from(id)), 0);
    assert!(events.is_empty(), "no event reveals the pick");
    choose(&mut s, Seat::B, 0).unwrap();
    let events = s.step();
    let b = s.view_for(Seat::B, &events);
    assert_eq!(b.augments[Seat::A.index()].get(usize::from(id)), 1);
    assert!(b.augment_round.is_none());
    assert!(events.contains(&Event::AugmentChosen { seat: Seat::A, id }));
}

#[test]
fn br_ag05_a_confirmed_pick_cannot_change_or_repeat() {
    let mut s = paused_battle(7);
    let id = s.augments.options[0];
    let cost = AUGMENT_COST[augment_def(id).unwrap().rarity.index()];
    let energy = s.players[0].energy;
    choose(&mut s, Seat::A, 0).unwrap();
    assert_eq!(choose(&mut s, Seat::A, 0), Err(Rejection::RoundClosed));
    assert_eq!(choose(&mut s, Seat::A, 3), Err(Rejection::RoundClosed));
    let round = s.augments.round;
    assert_eq!(
        s.apply_intention(Seat::A, 9_999, PlayerIntention::Forgo { round }),
        Err(Rejection::RoundClosed)
    );
    assert_eq!(s.players[0].energy, energy - cost, "charged once");
    while s.augments.selecting() {
        s.step();
    }
    assert_eq!(s.players[0].copies(id), 1);
    assert_eq!(choose(&mut s, Seat::A, 0), Err(Rejection::RoundClosed));
}

#[test]
fn br_ag01_the_next_draw_counts_only_effective_combat() {
    let mut s = MatchState::new(8);
    while !s.augments.selecting() {
        s.step();
    }
    let (first_tick, first_clock) = (s.augments.opened_at, s.clock);
    // Round 1 runs to its deadline; round 2 must still be 60 s of combat away.
    while s.augments.round < 2 {
        assert_ne!(s.phase, Phase::Ended);
        s.step();
    }
    assert_eq!(s.augments.opened_at - first_tick, AUGMENT_ROUND_TICKS);
    assert_eq!(
        s.clock - first_clock,
        AUGMENT_WINDOW_TICKS + AUGMENT_ROUND_TICKS,
        "20 s of selection plus 60 s of combat"
    );
}

#[test]
fn br_ag13_a_replay_keeps_paused_choices_in_order() {
    // The server records ops by selection clock; a choice made after a
    // long wait inside the pause must replay at the same step.
    let mut s = MatchState::with_arcanas(9, [ArcanaPreset::NEUTRAL; 2]);
    let mut replay = Replay {
        seed: 9,
        arcanas: [ArcanaPreset::NEUTRAL; 2],
        ..Replay::default()
    };
    let mut seq = 0;
    let mut act = |s: &mut MatchState, seat: Seat, intent: PlayerIntention| {
        seq += 1;
        replay.record(s, seat, Some((seq, intent)));
        let _ = s.apply_intention(seat, seq, intent);
    };
    while s.augments.round < 3 && s.phase != Phase::Ended {
        if s.clock.is_multiple_of(400) {
            act(&mut s, Seat::A, PlayerIntention::Purchase { offer: 1 });
            act(&mut s, Seat::B, PlayerIntention::Purchase { offer: 0 });
        }
        if s.augments.selecting() {
            let round = s.augments.round;
            let waited = s.clock + AUGMENT_WINDOW_TICKS - s.augments.deadline;
            if waited == 40 {
                act(
                    &mut s,
                    Seat::B,
                    PlayerIntention::ChooseAugment { round, option: 3 },
                );
            }
            if waited == 300 && round == 1 {
                act(&mut s, Seat::A, PlayerIntention::Forgo { round });
            }
        }
        s.step();
    }
    assert_eq!(s.augments.round, 3, "two paused rounds were replayed");
    assert!(s.units.iter().any(|u| u.kind != UnitKind::Beetling));
    replay.record(&s, Seat::A, None);
    s.forfeit(Seat::A);
    replay.final_hash = s.hash();
    let rerun = replay.run();
    assert_eq!(rerun.hash(), replay.final_hash);
    assert_eq!((rerun.clock, rerun.tick), (s.clock, s.tick));
}
