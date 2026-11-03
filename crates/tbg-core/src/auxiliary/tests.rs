//! War Auxiliary contract scenarios: hire, cumulative tiers and payouts (BR-AX01–BR-AX15).

use super::*;
use crate::{ArcanaPreset, Phase, PlayerIntention};

fn running_match() -> MatchState {
    let mut s = MatchState::with_preview(77, [ArcanaPreset::NEUTRAL; 2]);
    while s.phase != Phase::Running {
        s.step();
    }
    s
}

fn hire(s: &mut MatchState, seq: u32, slot: u8) -> Result<(), Rejection> {
    s.apply_intention(Seat::A, seq, PlayerIntention::HireAuxiliary { slot })
}

/// Run `ticks` steps on `s` and a twin without the Miner, returning the Energy delta.
fn miner_gain(s: &mut MatchState, twin: &mut MatchState, ticks: u32) -> u32 {
    let (before, twin_before) = (s.players[0].energy, twin.players[0].energy);
    for _ in 0..ticks {
        // Keep both seats out of Augment pauses so only the Miner differs.
        s.augments.next_open_at = u32::MAX;
        twin.augments.next_open_at = u32::MAX;
        s.step();
        twin.step();
    }
    (s.players[0].energy - before) - (twin.players[0].energy - twin_before)
}

#[test]
fn br_ax12_tier_costs_rise_and_the_fourth_step_is_rejected() {
    let mut s = running_match();
    s.players[0].energy = 200 * ENERGY_SCALE;
    assert_eq!(hire(&mut s, 1, 2), Err(Rejection::BadAuxiliarySlot));
    let mut seq = 2;
    for (slot, costs) in [(0, [8, 14, 22]), (1, [6, 10, 16])] {
        for (tier, cost) in costs.into_iter().enumerate() {
            let before = s.players[0].energy;
            hire(&mut s, seq, slot).unwrap();
            seq += 1;
            assert_eq!(before - s.players[0].energy, cost * ENERGY_SCALE);
            assert_eq!(
                s.players[0].auxiliary_tiers[usize::from(slot)],
                tier as u8 + 1
            );
        }
        assert_eq!(
            hire(&mut s, seq, slot),
            Err(Rejection::AuxiliaryAlreadyHired)
        );
        seq += 1;
    }
    let views = views(&s);
    assert_eq!(
        (views[0][0].tier, views[0][1].tier, views[1][0].tier),
        (3, 3, 0)
    );
}

#[test]
fn br_ax12_insufficient_energy_leaves_tier_and_energy_unchanged() {
    let mut s = running_match();
    s.players[0].energy = 7 * ENERGY_SCALE;
    assert_eq!(hire(&mut s, 1, 0), Err(Rejection::NotEnoughEnergy));
    assert_eq!(s.players[0].energy, 7 * ENERGY_SCALE);
    assert_eq!(s.players[0].auxiliary_tiers[0], 0);
    assert_eq!(s.players[0].auxiliaries[0], AuxiliaryPhase::Available);
}

#[test]
fn br_ax04_to_ax06_and_ax13_miner_prepares_then_each_tier_restarts_a_stronger_window() {
    let mut s = running_match();
    let mut twin = running_match();
    s.players[0].energy = 8 * ENERGY_SCALE;
    twin.players[0].energy = 0;
    hire(&mut s, 1, 0).unwrap();
    assert_eq!(
        s.players[0].auxiliaries[0],
        AuxiliaryPhase::Preparing { ticks_left: 240 }
    );
    assert_eq!(
        miner_gain(&mut s, &mut twin, 240),
        0,
        "no Energy while building"
    );

    // Max Interest on both seats: any Interest leak would multiply Miner output (BR-AX06).
    s.players[0].energy = 30 * ENERGY_SCALE;
    twin.players[0].energy = 30 * ENERGY_SCALE;
    assert_eq!(
        miner_gain(&mut s, &mut twin, 3_600),
        180 * 90,
        "tier 1: +0.15 Energy/s"
    );
    assert_eq!(s.players[0].auxiliaries[0], AuxiliaryPhase::Completed);

    // Evolving a Completed Miner restarts a full window at the cumulative tier-2 rate.
    s.players[0].energy += 14 * ENERGY_SCALE;
    hire(&mut s, 2, 0).unwrap();
    assert_eq!(miner_gain(&mut s, &mut twin, 1_800), 90 * 180);
    s.players[0].energy += 22 * ENERGY_SCALE;
    hire(&mut s, 3, 0).unwrap();
    assert_eq!(
        miner_gain(&mut s, &mut twin, 3_600),
        180 * 300,
        "tier 3: +0.50 Energy/s"
    );
    assert_eq!(s.players[0].auxiliaries[0], AuxiliaryPhase::Completed);
}

#[test]
fn br_ax13_evolving_a_preparing_miner_keeps_building() {
    let mut s = running_match();
    s.players[0].energy = 22 * ENERGY_SCALE;
    hire(&mut s, 1, 0).unwrap();
    s.step();
    hire(&mut s, 2, 0).unwrap();
    assert_eq!(
        s.players[0].auxiliaries[0],
        AuxiliaryPhase::Preparing { ticks_left: 239 }
    );
    assert_eq!(s.players[0].auxiliary_tiers[0], 2);
}

#[test]
fn br_ax07_to_ax10_and_ax14_merchant_tiers_pay_cumulative_rerolls_and_energy() {
    let mut s = running_match();
    s.players[0].energy = 200 * ENERGY_SCALE;
    // A purchase before hiring never counts.
    s.apply_intention(Seat::A, 1, PlayerIntention::Purchase { offer: 0 })
        .unwrap();
    let paid = s.view_for(Seat::A, &[]).reroll_cost.unwrap();
    s.apply_intention(Seat::A, 2, PlayerIntention::Reroll { cost: paid })
        .unwrap();
    assert_eq!(s.players[0].reroll_streak, 1);
    hire(&mut s, 3, 1).unwrap();
    let mut seq = 4;
    let mut buy = |s: &mut MatchState, n: u32| {
        for _ in 0..n {
            s.apply_intention(Seat::A, seq, PlayerIntention::Purchase { offer: 0 })
                .unwrap();
            seq += 1;
        }
    };

    // Tier 1: one free reroll per 3 purchases, no Energy.
    buy(&mut s, 3);
    assert_eq!(s.players[0].free_rerolls, 1);
    assert_eq!(s.view_for(Seat::A, &[]).reroll_cost, Some(0));
    let (energy, streak) = (s.players[0].energy, s.players[0].reroll_streak);
    s.apply_intention(Seat::A, 100, PlayerIntention::Reroll { cost: 0 })
        .unwrap();
    assert_eq!(
        (s.players[0].energy, s.players[0].free_rerolls),
        (energy, 0)
    );
    assert_eq!(
        s.players[0].reroll_streak, streak,
        "a free reroll never raises the streak (BR-AX09)"
    );

    // Tier 2: every 2 purchases pay 1 reroll + 1 Energy. Purchases are
    // recorded directly so their own Energy cost does not blur the refund.
    hire(&mut s, 101, 1).unwrap();
    let record = |s: &mut MatchState, n: u32| (0..n).for_each(|_| on_champion_purchase(s, Seat::A));
    let before = s.players[0].energy;
    record(&mut s, 2);
    assert_eq!(s.players[0].free_rerolls, 1);
    assert_eq!(s.players[0].energy - before, ENERGY_SCALE);

    // Tier 3: every 2 purchases pay 2 rerolls + 2 Energy, cap 12 activations total.
    hire(&mut s, 102, 1).unwrap();
    let before = s.players[0].energy;
    record(&mut s, 2);
    assert_eq!(s.players[0].free_rerolls, 3);
    assert_eq!(s.players[0].energy - before, 2 * ENERGY_SCALE);
    record(&mut s, 30);
    assert_eq!(s.players[0].auxiliaries[1], AuxiliaryPhase::Completed);
    assert_eq!(
        s.players[0].free_rerolls,
        3 + 2 * 9,
        "activations 4..=12 pay 2 rerolls each"
    );
}

#[test]
fn br_ax13_merchant_completed_at_tier_one_reopens_with_its_history() {
    let mut s = running_match();
    s.players[0].energy = 200 * ENERGY_SCALE;
    hire(&mut s, 1, 1).unwrap();
    for seq in 2..14 {
        s.apply_intention(Seat::A, seq, PlayerIntention::Purchase { offer: 0 })
            .unwrap();
    }
    assert_eq!(s.players[0].auxiliaries[1], AuxiliaryPhase::Completed);
    assert_eq!(s.players[0].free_rerolls, 4);
    hire(&mut s, 20, 1).unwrap();
    assert_eq!(
        s.players[0].auxiliaries[1],
        AuxiliaryPhase::Active {
            ticks_left: 0,
            progress: 0,
            activations_used: 4
        }
    );
}
