//! The HUD army count matches the living conventional units (D-32 test 5).

use crate::i18n::Lang;
use tbg_core::content::HATCH_EVERY;
use tbg_core::{MatchState, Phase, PlayerIntention, Seat, UnitKind};

#[test]
fn army_label_counts_living_units_and_summons_apart() {
    let mut s = MatchState::new(5);
    while s.phase != Phase::Running {
        s.step();
    }
    for kind in [UnitKind::Nester, UnitKind::Rampart]
        .into_iter()
        .chain([UnitKind::Archer; 10])
    {
        s.players[0].offers[0] = kind;
        s.players[0].energy = 6_000;
        s.apply_intention(Seat::A, 1, PlayerIntention::Purchase { offer: 0 })
            .unwrap();
    }
    assert_eq!(
        super::hud_update::army_label(&s.view_for(Seat::A, &[]), Lang::En),
        "Army 12"
    );
    for _ in 0..HATCH_EVERY {
        s.step();
    }
    let view = s.view_for(Seat::A, &[]);
    assert_eq!(
        super::hud_update::army_label(&view, Lang::En),
        "Army 12 | Brood 1"
    );
    assert_eq!(
        super::hud_update::army_label(&view, Lang::PtBr),
        "Ex\u{E9}rcito 12 | Ninhada 1"
    );
}
