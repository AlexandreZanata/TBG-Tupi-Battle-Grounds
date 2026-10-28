//! Elo rating contract (GAME-DESIGN BR-R03, BR-R05, §5 "Ranked rating").

use tbg_core::rating::{
    apply, delta, expected, Score, K_PLACEMENT, K_STANDARD, PLACEMENT_GAMES, RATING_FLOOR,
    START_RATING,
};

#[test]
fn br_r05_constants() {
    assert_eq!(
        (
            START_RATING,
            RATING_FLOOR,
            PLACEMENT_GAMES,
            K_PLACEMENT,
            K_STANDARD
        ),
        (1000, 100, 20, 40.0, 24.0)
    );
}

#[test]
fn br_r03_equal_win_moves_by_k_half() {
    // Equal 1000 vs 1000, win: 40 × 0.5 = +20 placement, 24 × 0.5 = +12 after.
    assert_eq!(delta(1000, 1000, 0, Score::Win), 20);
    assert_eq!(delta(1000, 1000, 20, Score::Win), 12);
    assert_eq!(delta(1000, 1000, 0, Score::Loss), -20);
    assert_eq!(delta(1000, 1000, 20, Score::Loss), -12);
}

#[test]
fn br_r03_favorite_gains_little_underdog_loses_little() {
    // 1400 beats 1000 (standard): 24 × (1 − 0.909) rounds to +2.
    assert_eq!(delta(1400, 1000, 30, Score::Win), 2);
    // The 1000 player losing to 1400 loses less than to an equal (−12).
    let loss = delta(1000, 1400, 30, Score::Loss);
    assert!(loss > -12 && loss < 0, "{loss}");
    // 2000 beats 1000: 24 × 0.08 rounds to 0, raised by the +1 rule.
    assert_eq!(delta(2000, 1000, 30, Score::Win), 1);
    // Symmetrically, the 1000 player losing to 2000 still loses at most −1.
    assert_eq!(delta(1000, 2000, 30, Score::Loss), -1);
}

#[test]
fn br_r03_draw_moves_toward_each_other() {
    // 1200 draws 1000: the 1200 player drops, the 1000 player climbs.
    assert!(delta(1200, 1000, 30, Score::Draw) < 0);
    assert!(delta(1000, 1200, 30, Score::Draw) > 0);
    // Equal draw changes nothing.
    assert_eq!(delta(1000, 1000, 30, Score::Draw), 0);
}

#[test]
fn br_r05_floor_holds() {
    assert_eq!(apply(100, -5000), 100);
    assert_eq!(apply(RATING_FLOOR, -1), RATING_FLOOR);
    assert_eq!(apply(1000, 20), 1020);
    // expected() sanity at the extremes.
    assert!(expected(2000, 1000) > 0.99);
    assert!(expected(1000, 2000) < 0.01);
}
