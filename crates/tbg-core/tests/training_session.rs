//! P62 training kernel (D-80): exact compositions, inert dummies, lifecycle,
//! determinism and attributed meters that reconcile with actual HP
//! (TROOPS-TRAINING V05–V08).
#![cfg(feature = "training")]

use tbg_core::training::{
    CommandError, DummyHp, DummyMode, Finish, Lifecycle, Opponent, Pause, ScenarioError, SessionId,
    Side, TrainingCommand, TrainingScenario, TrainingSession, TrainingUnitSpec, LANE_MAX,
    MAX_DUMMIES, MAX_UNITS_PER_SIDE, RUN_LIMIT_TICKS, SCHEMA,
};
use tbg_core::{Arena, Event, Seat, UnitKind};

fn base(kinds: &[UnitKind]) -> Vec<TrainingUnitSpec> {
    kinds.iter().map(|&k| TrainingUnitSpec::base(k)).collect()
}

fn scenario(allies: Vec<TrainingUnitSpec>, opponent: Opponent) -> TrainingScenario {
    TrainingScenario {
        schema: SCHEMA,
        seed: 62,
        arena: Arena::Forest,
        allies,
        opponent,
    }
}

fn dummies(mode: DummyMode, hp: DummyHp, count: usize) -> Opponent {
    Opponent::Dummies {
        mode,
        hp,
        positions: vec![None; count],
    }
}

/// Journey B: Archer, Shieldbearer, Bloomkeeper against Bone Brute, Hexer.
fn journey_b() -> TrainingScenario {
    scenario(
        base(&[
            UnitKind::Archer,
            UnitKind::Shieldbearer,
            UnitKind::Bloomkeeper,
        ]),
        Opponent::Champions(base(&[UnitKind::BoneBrute, UnitKind::Hexer])),
    )
}

fn session(s: TrainingScenario) -> TrainingSession {
    TrainingSession::new(s, SessionId(1)).expect("valid scenario")
}

/// Start and advance until the trial stops or `max` ticks; returns events.
fn run(s: &mut TrainingSession, max: u32) -> Vec<Event> {
    let id = s.id();
    s.command(id, TrainingCommand::Start).expect("start");
    let mut events = Vec::new();
    while s.lifecycle() == Lifecycle::Running && s.active_ticks() < max {
        s.tick();
        events.extend(s.take_events());
    }
    events
}

fn kinds_of(s: &TrainingSession, seat: Seat) -> Vec<UnitKind> {
    s.state()
        .units
        .iter()
        .filter(|u| u.seat == seat)
        .map(|u| u.kind)
        .collect()
}

/// Every row's recorded mutations explain its actual HP, and every HP
/// change is booked once on each side (V08).
fn assert_reconciles(s: &TrainingSession) {
    let m = s.metrics();
    assert_eq!(m.dropped, 0);
    for row in m.rows() {
        let actual = s
            .state()
            .unit_index(row.id)
            .map_or(0, |i| s.state().units[i].hp);
        assert_eq!(row.expected_hp(), i64::from(actual), "{row:?}");
    }
    let taken: u64 = m.rows().iter().map(|r| r.taken_total()).sum();
    let booked: u64 = m
        .rows()
        .iter()
        .map(|r| r.dealt_total() + r.dealt_friendly.iter().sum::<u64>() + r.taken_unattributed)
        .sum();
    assert_eq!(taken, booked, "damage booked once per side");
    let restored: u64 = m.rows().iter().map(|r| r.restored).sum();
    let healed: u64 = Seat::BOTH
        .iter()
        .map(|&seat| m.team(seat).healed + m.team(seat).regen + m.unattributed_heal(seat))
        .sum();
    assert_eq!(restored, healed, "healing booked once per side");
}

#[test]
fn three_champions_are_exactly_three_instances() {
    let mut s = session(journey_b());
    assert_eq!(
        kinds_of(&s, Seat::A),
        [
            UnitKind::Archer,
            UnitKind::Shieldbearer,
            UnitKind::Bloomkeeper
        ]
    );
    assert_eq!(
        kinds_of(&s, Seat::B),
        [UnitKind::BoneBrute, UnitKind::Hexer]
    );
    let events = run(&mut s, RUN_LIMIT_TICKS);
    assert!(
        matches!(
            s.lifecycle(),
            Lifecycle::Finished(_) | Lifecycle::Paused(Pause::RunLimit)
        ),
        "{:?}",
        s.lifecycle()
    );
    // No Shop, Energy or Stronghold outcome in the laboratory.
    assert!(s.state().outcome.is_none());
    assert!(!events.iter().any(|e| matches!(
        e,
        Event::AttackStart {
            from: tbg_core::Target::Stronghold(_),
            ..
        }
    )));
    assert!(s.metrics().team(Seat::A).dealt > 0);
    assert_reconciles(&s);
}

#[test]
fn duplicates_are_distinct_instances() {
    let s = session(scenario(
        base(&[UnitKind::Archer, UnitKind::Archer]),
        dummies(DummyMode::Finite, DummyHp::Light, 1),
    ));
    let archers: Vec<_> = s
        .metrics()
        .rows()
        .iter()
        .filter(|r| r.kind == UnitKind::Archer)
        .map(|r| r.id)
        .collect();
    assert_eq!(archers.len(), 2);
    assert_ne!(archers[0], archers[1]);
}

#[test]
fn drafts_are_validated_atomically() {
    let one = || base(&[UnitKind::Archer]);
    let target = || dummies(DummyMode::Finite, DummyHp::Light, 1);
    let cases = [
        (
            scenario(Vec::new(), target()),
            ScenarioError::EmptySide(Side::Allies),
        ),
        (
            scenario(
                vec![TrainingUnitSpec::base(UnitKind::Archer); MAX_UNITS_PER_SIDE + 1],
                target(),
            ),
            ScenarioError::TooMany(Side::Allies),
        ),
        (
            scenario(base(&[UnitKind::Beetling]), target()),
            ScenarioError::NotPlayable {
                side: Side::Allies,
                index: 0,
            },
        ),
        (
            scenario(
                vec![TrainingUnitSpec {
                    branch: 1,
                    ..TrainingUnitSpec::base(UnitKind::Archer)
                }],
                target(),
            ),
            ScenarioError::BadForm {
                side: Side::Allies,
                index: 0,
            },
        ),
        (
            scenario(
                vec![TrainingUnitSpec {
                    tier: 3,
                    ..TrainingUnitSpec::base(UnitKind::Archer)
                }],
                target(),
            ),
            ScenarioError::BadForm {
                side: Side::Allies,
                index: 0,
            },
        ),
        (
            scenario(
                vec![TrainingUnitSpec {
                    x: Some(LANE_MAX + 1),
                    ..TrainingUnitSpec::base(UnitKind::Archer)
                }],
                target(),
            ),
            ScenarioError::BadPosition {
                side: Side::Allies,
                index: 0,
            },
        ),
        (
            scenario(one(), Opponent::Champions(Vec::new())),
            ScenarioError::EmptySide(Side::Opponents),
        ),
        (
            scenario(
                one(),
                dummies(DummyMode::Finite, DummyHp::Light, MAX_DUMMIES + 1),
            ),
            ScenarioError::TooMany(Side::Opponents),
        ),
        (
            TrainingScenario {
                schema: SCHEMA + 1,
                ..scenario(one(), target())
            },
            ScenarioError::Schema(SCHEMA + 1),
        ),
    ];
    for (draft, error) in cases {
        assert_eq!(draft.validate(), Err(error));
    }
    // A rejected draft leaves the running trial untouched.
    let mut s = session(journey_b());
    let id = s.id();
    let before = s.state().hash();
    let bad = scenario(Vec::new(), target());
    assert_eq!(
        s.command(id, TrainingCommand::ReplaceDraft(Box::new(bad))),
        Err(CommandError::Scenario(ScenarioError::EmptySide(
            Side::Allies
        )))
    );
    assert_eq!((s.id(), s.state().hash()), (id, before));
    // The selection never becomes a Shop pool: canonical offers stay five.
    assert_eq!(s.state().players[0].offers.len(), 5);
}

#[test]
fn lifecycle_pause_step_resume_and_stale_ids() {
    let mut s = session(journey_b());
    let id = s.id();
    s.tick();
    assert_eq!(s.active_ticks(), 0, "Ready does not advance");
    assert_eq!(s.command(id, TrainingCommand::Step), Ok(id));
    assert_eq!(
        (s.active_ticks(), s.lifecycle()),
        (1, Lifecycle::Paused(Pause::Player))
    );
    s.tick();
    assert_eq!(s.active_ticks(), 1, "a paused trial is inert");
    assert_eq!(s.command(id, TrainingCommand::Step), Ok(id));
    assert_eq!(s.active_ticks(), 2, "one Step is one simulation tick");
    assert_eq!(s.command(id, TrainingCommand::Resume), Ok(id));
    assert_eq!(
        s.command(id, TrainingCommand::Start),
        Err(CommandError::InvalidState)
    );
    s.tick();
    assert_eq!(s.active_ticks(), 3);
    let fresh = s.command(id, TrainingCommand::Reset).expect("reset");
    assert_ne!(fresh, id);
    assert_eq!((s.active_ticks(), s.lifecycle()), (0, Lifecycle::Ready));
    assert_eq!(
        s.command(id, TrainingCommand::Start),
        Err(CommandError::StaleSession)
    );
}

#[test]
fn same_scenario_and_reset_replay_exactly() {
    let mut a = session(journey_b());
    let mut b = session(journey_b());
    run(&mut a, 400);
    run(&mut b, 400);
    assert_eq!(a.state().hash(), b.state().hash());
    assert_eq!(a.metrics(), b.metrics());
    let id = a.id();
    a.command(id, TrainingCommand::Reset).expect("reset");
    assert_eq!(
        a.metrics()
            .rows()
            .iter()
            .map(|r| r.taken_total())
            .sum::<u64>(),
        0
    );
    run(&mut a, 400);
    assert_eq!(a.state().hash(), b.state().hash());
    assert_eq!(a.metrics(), b.metrics());
}

#[test]
fn finite_dummy_is_inert_and_reports_its_kill() {
    let mut s = session(scenario(
        base(&[UnitKind::Archer, UnitKind::Archer, UnitKind::Archer]),
        dummies(DummyMode::Finite, DummyHp::Light, 1),
    ));
    let dummy = s
        .state()
        .units
        .iter()
        .find(|u| s.is_dummy(u.id))
        .copied()
        .expect("dummy");
    let id = s.id();
    s.command(id, TrainingCommand::Start).expect("start");
    let mut events = Vec::new();
    while s.lifecycle() == Lifecycle::Running {
        if let Some(i) = s.state().unit_index(dummy.id) {
            let u = s.state().units[i];
            assert_eq!(
                (u.x, u.max_hp),
                (dummy.x, DummyHp::Light.hp()),
                "never moves or rescales"
            );
        }
        s.tick();
        events.extend(s.take_events());
    }
    assert_eq!(s.lifecycle(), Lifecycle::Finished(Finish::TargetsDestroyed));
    let row = s.metrics().row(dummy.id).expect("dummy row");
    assert!(row.dummy);
    assert_eq!(
        row.taken_total(),
        DummyHp::Light.hp() as u64,
        "overkill excluded"
    );
    assert!(row.died_at.is_some());
    for e in &events {
        match *e {
            Event::CrateOpened { unit, .. } => assert_ne!(unit, dummy.id, "a dummy never opens"),
            Event::AttackStart {
                from: tbg_core::Target::Unit(u),
                ..
            } => {
                assert_ne!(u, dummy.id, "a dummy never attacks")
            }
            _ => {}
        }
    }
    assert_reconciles(&s);
}

#[test]
fn renewable_dummy_respawns_with_a_new_id_next_tick() {
    let mut s = session(scenario(
        base(&[UnitKind::Archer, UnitKind::Archer, UnitKind::Archer]),
        dummies(DummyMode::Renewable, DummyHp::Light, 1),
    ));
    let id = s.id();
    s.command(id, TrainingCommand::Start).expect("start");
    let lives = |s: &TrainingSession| s.metrics().rows().iter().filter(|r| r.dummy).count();
    while s.lifecycle() == Lifecycle::Running && lives(&s) < 3 {
        s.tick();
    }
    assert_eq!(
        s.lifecycle(),
        Lifecycle::Running,
        "an awaiting target never ends the trial"
    );
    let rows: Vec<_> = s.metrics().rows().iter().filter(|r| r.dummy).collect();
    assert_eq!(rows.len(), 3);
    for pair in rows.windows(2) {
        assert_ne!(pair[0].id, pair[1].id);
        assert_eq!(
            pair[0].died_at,
            Some(pair[1].spawned_at),
            "respawn on the next tick"
        );
        assert_eq!(pair[0].taken_total(), DummyHp::Light.hp() as u64);
    }
    assert_reconciles(&s);
}

#[test]
fn real_smuggler_crates_still_open_beside_dummies() {
    let mut s = session(scenario(
        base(&[UnitKind::Smuggler]),
        dummies(DummyMode::Renewable, DummyHp::Heavy, 1),
    ));
    let events = run(&mut s, 3_000);
    let opened = events.iter().any(|e| {
        matches!(*e, Event::CrateOpened { unit, .. } if s.metrics().row(unit).is_some_and(|r| !r.dummy))
    });
    assert!(opened, "Smuggler cargo keeps its canonical timer");
    assert_reconciles(&s);
}
