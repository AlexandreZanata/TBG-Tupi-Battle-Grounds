//! BR-S06–S14 preview contracts; CANON remains disabled until owner approval.
use tbg_core::{MatchState, PlayerIntention as I, Rejection as R, Seat};
fn state() -> MatchState {
    let mut s = MatchState::with_preview(29, [tbg_core::ArcanaPreset::NEUTRAL; 2]);
    s.phase = tbg_core::Phase::Running;
    s.players[0].energy = 100_000;
    s
}
fn reserve(s: &mut MatchState, offer: u8) -> Result<(), R> {
    let kind = s.players[0].offers[offer as usize];
    s.apply_intention(Seat::A, 0, I::Reserve { offer, kind })
}
fn roll(s: &mut MatchState) -> Result<(), R> {
    let cost = s.view_for(Seat::A, &[]).reroll_cost.unwrap();
    s.apply_intention(Seat::A, 0, I::Reroll { cost })
}
#[test]
fn br_s06_stale_limit_and_already_held() {
    let mut s = state();
    let old = s.players[0].offers[0];
    let wrong = tbg_core::FULL_POOL.into_iter().find(|k| *k != old).unwrap();
    assert_eq!(
        s.apply_intention(
            Seat::A,
            0,
            I::Reserve {
                offer: 0,
                kind: wrong
            }
        ),
        Err(R::StaleOffer)
    );
    reserve(&mut s, 0).unwrap();
    reserve(&mut s, 1).unwrap();
    assert_eq!(reserve(&mut s, 2), Err(R::ReserveFull));
    assert_eq!(reserve(&mut s, 0), Err(R::AlreadyHeld));
    assert_eq!(
        s.apply_intention(Seat::A, 0, I::Release { offer: 5 }),
        Err(R::BadOffer)
    );
}
#[test]
fn br_s07_s08_charges_expire_after_three_new_shops_without_redrawing() {
    let mut s = state();
    let offers = s.players[0].offers;
    reserve(&mut s, 0).unwrap();
    reserve(&mut s, 1).unwrap();
    for charges in [2, 1, 0] {
        roll(&mut s).unwrap();
        assert_eq!(&s.players[0].offers[..2], &offers[..2]);
        assert_eq!(&s.players[0].reserve[..2], &[charges; 2]);
    }
    assert_eq!(reserve(&mut s, 0), Err(R::AlreadyHeld));
    roll(&mut s).unwrap();
    assert_eq!(s.players[0].held_mask, 0);
}
#[test]
fn br_s09_purchase_resets_price_consumes_held_slot_and_charges_other() {
    let mut s = state();
    reserve(&mut s, 0).unwrap();
    reserve(&mut s, 1).unwrap();
    let kept = s.players[0].offers[1];
    roll(&mut s).unwrap();
    s.apply_intention(Seat::A, 2, I::Purchase { offer: 0 })
        .unwrap();
    assert_eq!(s.players[0].reserve[0], 0);
    assert_eq!(s.players[0].reserve[1], 1);
    assert_eq!(s.players[0].offers[1], kept);
    assert_eq!(s.view_for(Seat::A, &[]).reroll_cost, Some(600));
}
#[test]
fn br_s09_s10_release_leaves_card_visible_and_cannot_renew() {
    let mut s = state();
    let card = s.players[0].offers[0];
    reserve(&mut s, 0).unwrap();
    s.apply_intention(Seat::A, 0, I::Release { offer: 0 })
        .unwrap();
    assert_eq!(s.players[0].offers[0], card);
    assert_eq!(reserve(&mut s, 0), Err(R::AlreadyHeld));
    assert_eq!(
        s.apply_intention(Seat::A, 0, I::Release { offer: 0 }),
        Err(R::NoReserve)
    );
    roll(&mut s).unwrap();
    reserve(&mut s, 0).unwrap();
}
#[test]
fn br_s11_prices_spend_exactly_and_stale_or_insufficient_is_free() {
    let mut s = state();
    for (i, cost) in [600, 1200, 1800].into_iter().enumerate() {
        let energy = s.players[0].energy;
        roll(&mut s).unwrap();
        assert_eq!(s.players[0].energy, energy - cost);
        assert_eq!(s.players[0].reroll_streak, i as u8 + 1);
    }
    let before = s.players[0].clone();
    assert_eq!(
        s.apply_intention(Seat::A, 0, I::Reroll { cost: 600 }),
        Err(R::StalePrice)
    );
    assert_eq!(s.players[0], before);
    s.players[0].energy = 0;
    assert_eq!(roll(&mut s), Err(R::NotEnoughEnergy));
    assert_eq!(s.players[0].energy, 0);
}
#[test]
fn br_s12_outside_running_and_disabled() {
    let mut s = MatchState::new(1);
    let kind = s.players[0].offers[0];
    for intent in [
        I::Reserve { offer: 0, kind },
        I::Release { offer: 0 },
        I::Reroll { cost: 600 },
    ] {
        assert_eq!(s.apply_intention(Seat::A, 0, intent), Err(R::NotRunning));
        s.phase = tbg_core::Phase::Running;
        assert_eq!(s.apply_intention(Seat::A, 0, intent), Err(R::Unavailable));
        s.phase = tbg_core::Phase::Countdown;
    }
}
#[test]
fn br_s13_only_own_view_changes_and_public_bytes_are_unchanged() {
    let mut s = state();
    let other = s.view_for(Seat::B, &[]);
    let public = s.public_view(&[], 0);
    reserve(&mut s, 0).unwrap();
    roll(&mut s).unwrap();
    assert_eq!(s.view_for(Seat::B, &[]), other);
    assert_eq!(s.public_view(&[], 0), public);
}
#[test]
fn br_s14_each_intention_sees_the_previous_shop_and_is_deterministic() {
    let mut s = state();
    let mut twin = state();
    for s in [&mut s, &mut twin] {
        roll(s).unwrap();
        let kind = s.players[0].offers[0];
        s.apply_intention(Seat::A, 3, I::Purchase { offer: 0 })
            .unwrap();
        assert_eq!(s.units.last().unwrap().kind, kind);
    }
    assert_eq!(s.hash(), twin.hash());
}
#[test]
fn enabled_but_unused_preserves_canon_behavior() {
    let mut a = MatchState::new(7);
    let mut b = MatchState::with_preview(7, [tbg_core::ArcanaPreset::NEUTRAL; 2]);
    for _ in 0..100 {
        assert_eq!(a.step(), b.step());
    }
    for _ in 0..10 {
        for s in [&mut a, &mut b] {
            s.players[0].energy = 100_000;
            s.apply_intention(Seat::A, 0, I::Purchase { offer: 0 })
                .unwrap();
        }
        assert_eq!(a.players, b.players);
        assert_eq!(a.units, b.units);
    }
    assert_eq!(a.hash(), b.hash());
}
