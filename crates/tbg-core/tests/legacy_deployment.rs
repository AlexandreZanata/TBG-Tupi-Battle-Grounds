//! RESTORE-01: purchases enter immediately and allies never impose a gameplay passage queue.
use tbg_core::{content, ArcanaPreset, MatchState, Phase, PlayerIntention, Seat, UnitKind};
#[test]
fn restore01_preview_purchases_enter_and_advance_without_body_blocking() {
    for seat in Seat::BOTH {
        let mut s = MatchState::with_preview(7, [ArcanaPreset::NEUTRAL; 2]);
        s.phase = Phase::Running;
        s.tick = content::RUNNING_START;
        s.players[seat.index()].energy = 600_000;
        let mut spent = 0;
        for seq in 1..=12 {
            let kind = if seq % 2 == 0 {
                UnitKind::Colossus
            } else {
                UnitKind::Archer
            };
            s.players[seat.index()].offers[0] = kind;
            spent += content::def(kind).cost;
            assert_eq!(
                s.apply_intention(seat, seq, PlayerIntention::Purchase { offer: 0 }),
                Ok(())
            );
            assert_eq!(s.army_size(seat), seq as usize);
            assert_eq!(s.view_for(seat, &[]).queued, 0);
        }
        assert_eq!(s.players[seat.index()].energy, 600_000 - spent);
        assert!(!s.physical_lanes);
        let first = s.units[0].id;
        assert_eq!(
            s.apply_intention(
                seat,
                13,
                PlayerIntention::Evolve {
                    unit: first,
                    branch: 0
                }
            ),
            Ok(())
        );
        let before: Vec<_> = s.units.iter().map(|u| (u.id, u.x)).collect();
        for _ in 0..20 {
            s.step();
        }
        assert_eq!(s.units.len(), 12);
        assert!(s.deployments.is_empty());
        for (id, x) in before {
            assert!((s.units.iter().find(|u| u.id == id).unwrap().x - x) * seat.dir() > 0);
        }
    }
}

#[cfg(feature = "training")]
#[test]
fn restore01_training_keeps_all_figures_active_without_a_lane_queue() {
    use tbg_core::training::{
        Opponent, SessionId, TrainingCommand, TrainingScenario, TrainingSession, TrainingUnitSpec,
        SCHEMA,
    };
    let scenario = TrainingScenario {
        schema: SCHEMA,
        seed: 7,
        arena: tbg_core::Arena::Forest,
        allies: (0..12)
            .map(|_| TrainingUnitSpec {
                x: Some(500),
                ..TrainingUnitSpec::base(UnitKind::Colossus)
            })
            .collect(),
        opponent: Opponent::Champions(vec![TrainingUnitSpec {
            x: Some(14500),
            ..TrainingUnitSpec::base(UnitKind::Shieldbearer)
        }]),
    };
    let mut trial = TrainingSession::new(scenario, SessionId(1)).unwrap();
    trial.command(SessionId(1), TrainingCommand::Start).unwrap();
    for _ in 0..20 {
        trial.tick();
    }
    assert!(!trial.state().physical_lanes);
    assert!(trial.state().deployments.is_empty());
    assert_eq!(trial.state().army_size(Seat::A), 12);
    assert!(trial
        .state()
        .units
        .iter()
        .filter(|u| u.seat == Seat::A)
        .all(|u| u.x > 500));
}
