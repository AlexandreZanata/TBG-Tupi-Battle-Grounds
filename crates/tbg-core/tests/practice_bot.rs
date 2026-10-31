//! P62 local difficulty contract: baseline parity, deterministic public decisions and disclosed assistance.
use tbg_core::{bot::Bot, practice_bot::PracticeBot, MatchState, Phase, Seat};
#[test]
fn medium_delegates_without_changing_the_baseline() {
    let mut state = MatchState::new(62);
    let mut old = Bot::new(17);
    let mut new = PracticeBot::new(17, 1);
    for _ in 0..500 {
        let events = state.step();
        let view = state.view_for(Seat::B, &events);
        assert_eq!(old.decide(&view), new.decide(&view));
        assert_eq!(new.regen_bonus(&state), 0);
    }
}
#[test]
fn assistance_is_local_exact_and_frozen_during_selection() {
    for difficulty in 0..4 {
        let mut state = MatchState::new(7);
        while state.phase != Phase::Running {
            state.step();
        }
        state.players[1].energy = 0;
        let mut bot = PracticeBot::new(7, difficulty);
        let initial = state.hash();
        let mut bonus = 0;
        for _ in 0..20 {
            bonus += bot.regen_bonus(&state);
        }
        assert_eq!(
            bonus,
            if difficulty == 3 {
                state.rules().regen_running * 20
            } else {
                0
            }
        );
        assert_eq!(
            state.hash(),
            initial,
            "policy never mutates canonical state"
        );
    }
}

#[test]
fn impossible_matches_each_actual_regen_gain_including_the_running_transition() {
    let mut state = MatchState::new(13).with_arena(tbg_core::Arena::Storm);
    let mut bot = PracticeBot::new(13, 3);
    for _ in 0..300 {
        let energy = state.players[1].energy;
        let bonus = bot.regen_bonus(&state);
        state.step();
        assert_eq!(bonus, state.players[1].energy - energy);
    }
}
