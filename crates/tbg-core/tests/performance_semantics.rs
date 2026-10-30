//! Live-position and live-death contracts that an immutable tick-start
//! spatial snapshot would violate (BR-U01, BR-U09; ARCHITECTURE section 4).

use tbg_core::{content::WINDUP, MatchState, PlayerIntention, Seat, Target, UnitKind};

fn running() -> MatchState {
    let mut state = MatchState::new(1);
    while !state.is_live() {
        state.step();
    }
    state
}

fn spawn(state: &mut MatchState, seat: Seat, kind: UnitKind, x: i32) -> u32 {
    state.players[seat.index()].energy = 100_000;
    state.players[seat.index()].offers[0] = kind;
    state
        .apply_intention(seat, 0, PlayerIntention::Purchase { offer: 0 })
        .unwrap();
    let unit = state.units.last_mut().unwrap();
    unit.x = x;
    unit.id
}

#[test]
fn later_unit_observes_an_enemy_that_moved_into_range_this_tick() {
    let mut state = running();
    let enemy = spawn(&mut state, Seat::A, UnitKind::Shieldbearer, 0);
    let archer = spawn(&mut state, Seat::B, UnitKind::Archer, 9_000);
    let range = state.stats_of(&state.units[1]).range;
    state.units[0].x = 9_000 - range - 1;
    state.step();
    let i = state.unit_index(archer).unwrap();
    assert_eq!(state.units[i].windup, Some((Target::Unit(enemy), WINDUP)));
}

#[test]
fn later_unit_does_not_target_a_guard_killed_earlier_in_the_tick() {
    let mut state = running();
    spawn(&mut state, Seat::A, UnitKind::Shieldbearer, 7_900);
    let guard = spawn(&mut state, Seat::B, UnitKind::Rampart, 8_000);
    let archer = spawn(&mut state, Seat::A, UnitKind::Archer, 7_000);
    let survivor = spawn(&mut state, Seat::B, UnitKind::Shieldbearer, 9_000);
    state.units[0].windup = Some((Target::Unit(guard), 1));
    state.units[1].hp = 1;
    state.step();
    let i = state.unit_index(archer).unwrap();
    assert_eq!(
        state.units[i].windup,
        Some((Target::Unit(survivor), WINDUP))
    );
}
