//! INT-06 / BR-AX01–BR-AX12: authoritative auxiliary state, replay and private network projection.
use tbg_core::{
    auxiliary::AuxiliaryPhase, content::ENERGY_SCALE, MatchState, Phase, PlayerIntention,
    Rejection, Seat,
};

fn running() -> MatchState {
    let mut state = MatchState::new(74);
    while state.phase != Phase::Running {
        state.step();
    }
    state
}

#[test]
fn auxiliary_state_and_free_rerolls_survive_serialization_and_affect_hashes() {
    let base = running();
    let mut changed = base.clone();
    changed.players[1].auxiliaries[0] = AuxiliaryPhase::Preparing { ticks_left: 137 };
    assert_ne!(
        base.hash(),
        changed.hash(),
        "BR-AX04: construction belongs to deterministic state"
    );
    let before = changed.hash();
    changed.players[1].free_rerolls = 2;
    assert_ne!(
        before,
        changed.hash(),
        "BR-AX08: free currency belongs to deterministic state"
    );
    let restored: MatchState =
        postcard::from_bytes(&postcard::to_allocvec(&changed).unwrap()).unwrap();
    // Offline LabStats are deliberately transient; competitive serialized state is not.
    assert_eq!(
        restored.players[1].auxiliaries,
        changed.players[1].auxiliaries
    );
    assert_eq!(
        restored.players[1].free_rerolls,
        changed.players[1].free_rerolls
    );
    assert_eq!(restored.hash(), changed.hash());
}

#[test]
fn merchant_free_rerolls_work_at_zero_energy_and_remain_private_over_the_wire() {
    let mut state = running();
    state.players[1].energy = 100 * ENERGY_SCALE;
    state
        .apply_intention(Seat::B, 1, PlayerIntention::HireAuxiliary { slot: 1 })
        .unwrap();
    for seq in 2..=4 {
        state
            .apply_intention(Seat::B, seq, PlayerIntention::Purchase { offer: 0 })
            .unwrap();
    }
    assert_eq!(state.players[1].free_rerolls, 1);
    state.players[1].energy = 0;
    state.players[1].reroll_streak = 2;
    let view = state.view_for(Seat::B, &[]);
    let wire =
        tbg_core::protocol::encode(&tbg_core::protocol::ServerMsg::View(Box::new(view.clone())));
    assert_eq!(
        tbg_core::protocol::decode_server(&wire).unwrap(),
        tbg_core::protocol::ServerMsg::View(Box::new(view))
    );
    assert_eq!(state.view_for(Seat::B, &[]).reroll_cost, Some(0));
    assert_eq!(state.view_for(Seat::A, &[]).free_rerolls, 0);
    let public = state.public_view(&[], 0);
    let mut other = state.clone();
    other.players[1].free_rerolls = 3;
    assert_eq!(
        public,
        other.public_view(&[], 0),
        "BR-AX11: free rerolls never enter PublicView"
    );
    state
        .apply_intention(Seat::B, 5, PlayerIntention::Reroll { cost: 0 })
        .unwrap();
    assert_eq!(state.players[1].energy, 0);
    assert_eq!(state.players[1].free_rerolls, 0);
    assert_eq!(
        state.players[1].reroll_streak, 2,
        "BR-AX09: a free reroll preserves the paid streak"
    );
}

#[test]
fn auxiliary_clocks_and_hiring_freeze_during_augment_selection() {
    let mut state = running();
    state.players[0].energy = 100 * ENERGY_SCALE;
    state
        .apply_intention(Seat::A, 1, PlayerIntention::HireAuxiliary { slot: 0 })
        .unwrap();
    state.augments.round = 1;
    state.augments.deadline = state.clock + 10;
    let phase = state.players[0].auxiliaries;
    let tick = state.tick;
    for _ in 0..5 {
        state.step();
    }
    assert_eq!(state.tick, tick, "BR-AX04: production follows combat time");
    assert_eq!(state.players[0].auxiliaries, phase);
    assert_eq!(
        state.apply_intention(Seat::A, 2, PlayerIntention::HireAuxiliary { slot: 1 }),
        Err(Rejection::Paused)
    );
    assert_eq!(state.players[0].auxiliaries[1], AuxiliaryPhase::Available);
}
