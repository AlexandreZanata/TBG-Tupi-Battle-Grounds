//! Balance-lab contracts C3, C5–C9, C11 (D-33, P16-C step 10). They iterate
//! the roster, the forms, and the knobs, never named units, so new content
//! is covered by itself. Append-only: change a test only in the same commit
//! as the canon change it guards.

use tbg_balance::duels::{self, Form};
use tbg_balance::gate;
use tbg_balance::knobs;
use tbg_balance::meta;
use tbg_balance::policy::Policy;
use tbg_balance::runner::{parallel_map, run_pairs, Matchup, Setup};
use tbg_balance::stats;
use tbg_core::content;
use tbg_core::ArcanaPreset;
use tbg_core::{MatchState, Pcg32, Ruleset, Seat, UnitKind, UNIT_KINDS};

fn leak(r: Ruleset) -> &'static Ruleset {
    Box::leak(Box::new(r))
}

fn canon() -> &'static Ruleset {
    Ruleset::canon()
}

/// Seed pairs for match canaries (healers, duel fallbacks): the opponent plays
/// the same ruleset, so the shift is diluted and needs a larger sample.
const MATCH_PAIRS: u64 = 256;

fn favor_vs_bot(rules: &'static Ruleset, k: UnitKind, pairs: u64) -> f64 {
    favor_vs(rules, k, pairs, Policy::Bot)
}

/// Score of `favor-<unit>` against `opponent`. Since the Shop changes only
/// on a purchase (BR-S04, D-44) a unit is offered rarely, so the match row
/// against the shipped bot (which also buys tanks and the dearest cards)
/// dilutes a planted imbalance; the cheap-spam opponent leaves the favored
/// unit as the only variable.
fn favor_vs(rules: &'static Ruleset, k: UnitKind, pairs: u64, opponent: Policy) -> f64 {
    let m = Matchup {
        rules,
        policies: [Policy::Favor(k), opponent],
        reaction: [gate::REACTION; 2],
        setup: Setup::None,
        arcanas: [ArcanaPreset::NEUTRAL; 2],
    };
    let r = run_pairs(&[m], 0..pairs, 4).remove(0);
    r.iter().map(|p| p[0].score(0) + p[1].score(0)).sum::<f64>() / (2 * pairs) as f64
}

/// Scale one kind's base stats (and every form's flat additions stay).
fn scaled(kind: UnitKind, hp: i64, out: i64, cost: i64) -> &'static Ruleset {
    let mut r = canon().clone();
    let d = &mut r.units[kind as usize];
    d.hp = (i64::from(d.hp) * hp) as i32;
    d.damage = (i64::from(d.damage) * out) as i32;
    d.heal = (i64::from(d.heal) * out) as i32;
    d.cost = (i64::from(d.cost) * cost) as u32;
    leak(r)
}

#[test]
fn c3_results_do_not_depend_on_thread_count() {
    let m = Matchup {
        rules: canon(),
        policies: [Policy::Heavy, Policy::Bot],
        reaction: [1, 5],
        setup: Setup::None,
        arcanas: [ArcanaPreset::NEUTRAL; 2],
    };
    let one = run_pairs(&[m], 0..6, 1);
    let many = run_pairs(&[m], 0..6, 4);
    assert_eq!(one, many, "C3: output changed with the thread count");
    assert_eq!(one, run_pairs(&[m], 0..6, 3), "C3: rerun changed");
    let squares = parallel_map(&(0..100u64).collect::<Vec<_>>(), 7, |x| x * x);
    assert_eq!(squares, (0..100u64).map(|x| x * x).collect::<Vec<_>>());
}

#[test]
fn c5_every_tunable_number_is_registered() {
    let r = canon();
    // CANON is what the online game plays (content.rs wrappers).
    for k in tbg_core::ALL_KINDS {
        assert_eq!(
            content::def(k),
            r.def(k),
            "{k:?}: CANON differs from content.rs"
        );
    }
    let all = knobs::all(r);
    for k in &all {
        // Every knob is named, parseable, and writes its own field.
        let name = knobs::name(r, *k);
        let mut copy = r.clone();
        knobs::apply(&mut copy, &format!("{name}={}", knobs::get(r, *k))).expect(&name);
        assert_eq!(
            copy, *r,
            "{name}: re-applying the canon value changed the ruleset"
        );
        let mut moved = r.clone();
        knobs::set(&mut moved, *k, knobs::moved(*k, knobs::get(r, *k), 1));
        assert_ne!(moved, *r, "{name}: setting the knob changed nothing");
    }
    for kind in UNIT_KINDS {
        let i = kind as usize;
        for field in [knobs::UnitField::Cost, knobs::UnitField::Hp] {
            assert!(
                all.contains(&knobs::Knob::Unit(i, field)),
                "{kind:?}: {field:?} is not a knob"
            );
        }
        assert!(
            Policy::pool().contains(&Policy::Favor(kind)),
            "{kind:?}: no Favor policy"
        );
        assert_eq!(
            duels::forms().iter().filter(|f| f.kind == kind).count(),
            5,
            "{kind:?}: duel forms"
        );
    }
    knobs::valid(r).expect("CANON breaks a role invariant");
}

/// State hash after `kind` with a Bone Brute escort meets its mirror (the
/// escort hits harder than any heal, so healing is never capped away).
fn sandbox_hash(rules: &Ruleset, kind: UnitKind) -> u64 {
    let arena = duels::arena(rules);
    let mut s = MatchState::with_rules(0, arena);
    while s.tick < content::RUNNING_START {
        s.step();
    }
    // An escort on both sides gives healers someone to heal.
    for (seat, units) in [
        (Seat::A, [kind, UnitKind::BoneBrute]),
        (Seat::B, [UnitKind::BoneBrute, kind]),
    ] {
        for k in units {
            s.lab_spawn(seat, k, 0, 0);
        }
    }
    for _ in 0..600 {
        s.step();
    }
    s.hash()
}

#[test]
fn c6_the_simulation_reads_the_ruleset() {
    for kind in UNIT_KINDS {
        let base = sandbox_hash(canon(), kind);
        assert_ne!(
            sandbox_hash(scaled(kind, 3, 1, 1), kind),
            base,
            "{kind:?}: HP is a hidden constant"
        );
        assert_ne!(
            sandbox_hash(scaled(kind, 1, 3, 1), kind),
            base,
            "{kind:?}: damage/heal is a hidden constant"
        );
    }
}

#[test]
fn c7_planted_imbalances_are_caught_for_every_unit() {
    let forms: Vec<Form> = duels::forms().into_iter().filter(|f| f.tier == 0).collect();
    let base = |k: UnitKind| {
        forms
            .iter()
            .position(|f| {
                *f == Form {
                    kind: k,
                    tier: 0,
                    branch: 0,
                }
            })
            .unwrap()
    };
    let plain = duels::matrix_of(canon(), forms.clone(), 4);
    for kind in UNIT_KINDS {
        if canon().def(kind).damage > 0 {
            let i = base(kind);
            let avg = |m: &duels::Matrix, b: usize| m.average_among(b, i, |_| true).unwrap_or(0.0);
            let budgets = 0..duels::BUDGETS.len();
            // A shift must be at least 20 points, or half the room left
            // (a unit already winning most duels cannot gain 20 more).
            let moved = |shift: f64, room: f64| shift >= 0.20f64.min(room / 2.0);
            // Doubled HP and damage: flagged oppressive, or clearly stronger
            // against the other base forms at every budget.
            let strong = duels::matrix_of(scaled(kind, 2, 2, 1), forms.clone(), 4);
            let caught = strong.flags(canon()).oppressive.contains(&i)
                || budgets
                    .clone()
                    .all(|b| moved(avg(&strong, b) - avg(&plain, b), 1.0 - avg(&plain, b)));
            // Duels alone may miss it (a tank still loses to range): then
            // the match row must move (`favor-<unit>` vs `bot`).
            let caught = caught
                || favor_vs_bot(scaled(kind, 2, 2, 1), kind, MATCH_PAIRS)
                    >= favor_vs_bot(canon(), kind, MATCH_PAIRS) + 0.10
                || favor_vs(scaled(kind, 2, 2, 1), kind, MATCH_PAIRS, Policy::Cheap)
                    >= favor_vs(canon(), kind, MATCH_PAIRS, Policy::Cheap) + 0.10;
            assert!(caught, "{kind:?} ×2 HP and damage was not caught");
            // A tripled cost: flagged weak, or clearly weaker everywhere.
            let dear = duels::matrix_of(scaled(kind, 1, 1, 3), forms.clone(), 4);
            let caught = dear.flags(canon()).weak.contains(&i)
                || budgets
                    .clone()
                    .all(|b| moved(avg(&plain, b) - avg(&dear, b), avg(&plain, b)));
            let caught = caught
                || favor_vs_bot(scaled(kind, 1, 1, 3), kind, MATCH_PAIRS)
                    <= favor_vs_bot(canon(), kind, MATCH_PAIRS) - 0.10
                || favor_vs(scaled(kind, 1, 1, 3), kind, MATCH_PAIRS, Policy::Cheap)
                    <= favor_vs(canon(), kind, MATCH_PAIRS, Policy::Cheap) - 0.10;
            assert!(caught, "{kind:?} ×3 cost was not caught");
        } else {
            // A pure healer cannot win a duel: measured in matches instead.
            let canon_score = favor_vs_bot(canon(), kind, MATCH_PAIRS);
            let strong = favor_vs_bot(scaled(kind, 2, 2, 1), kind, MATCH_PAIRS);
            assert!(
                strong >= canon_score + 0.10,
                "{kind:?} ×2 HP and heal: {canon_score:.2} -> {strong:.2}"
            );
        }
    }
}

#[test]
fn c8_a_flat_roster_flags_nothing() {
    // Every conventional unit gets the Shieldbearer's base stats, forms and
    // (empty) abilities; no Synergy bonus. P44: staged kinds join the flat
    // roster, or their own forms and abilities would break the symmetry.
    let mut r = canon().clone();
    let template = *r.def(UnitKind::Shieldbearer);
    for kind in tbg_core::ALL_KINDS
        .into_iter()
        .filter(|k| k.is_conventional())
    {
        let d = &mut r.units[kind as usize];
        *d = tbg_core::content::UnitDef {
            kind,
            name: d.name,
            faction: d.faction,
            archetype: d.archetype,
            ..template
        };
    }
    for s in &mut r.synergies {
        s.1 = [0, 0];
    }
    r.hatch_every = u16::MAX;
    r.hatch_every_hive = u16::MAX;
    let rules = leak(r);
    let m = duels::matrix(rules, 4);
    for (i, f) in m.forms.iter().enumerate().filter(|(_, f)| f.tier == 0) {
        for (j, g) in m.forms.iter().enumerate().filter(|(_, g)| g.tier == 0) {
            for b in 0..duels::BUDGETS.len() {
                assert_eq!(
                    m.scores[b][i][j],
                    Some(0.5),
                    "{f:?} vs {g:?}: flat roster not 50 %"
                );
            }
        }
    }
    let flags = m.flags(rules);
    assert!(
        flags.weak.is_empty() && flags.oppressive.is_empty(),
        "{flags:?}"
    );
    assert!(
        flags.counterless.iter().all(|&i| m.forms[i].tier > 0),
        "{flags:?}"
    );
}

#[test]
fn c9_statistics_are_sound() {
    // A known rate: 2,000 Bernoulli(0.3) clusters; the interval covers it.
    let mut rng = Pcg32::new(9, 9);
    let v: Vec<f64> = (0..2_000)
        .map(|_| f64::from(u8::from(rng.below(10) < 3)))
        .collect();
    let e = stats::mean(&v, 2_000, 1);
    assert!(e.lo < 0.3 && 0.3 < e.hi, "{e:?}");
    assert!(e.half_width() < 0.03, "{e:?}");
    assert_eq!(
        stats::mean(&v, 2_000, 1),
        e,
        "bootstrap is not deterministic"
    );
    // Rock-paper-scissors: one third each.
    let rps = vec![
        vec![0.5, 0.0, 1.0],
        vec![1.0, 0.5, 0.0],
        vec![0.0, 1.0, 0.5],
    ];
    for w in meta::nash(&rps, 30_000) {
        assert!((w - 1.0 / 3.0).abs() < 0.02, "rps weight {w}");
    }
    // A strategy dominated by another gets no weight.
    let dom = vec![
        vec![0.5, 0.2, 0.9],
        vec![0.8, 0.5, 0.95],
        vec![0.1, 0.05, 0.5],
    ];
    let mix = meta::nash(&dom, 10_000);
    assert!(
        mix[0] < 0.01 && mix[2] < 0.01 && meta::support(&mix) == 1,
        "{mix:?}"
    );
}

#[test]
fn c11_every_policy_plays_legal_complete_matches() {
    let mut all = Policy::pool();
    all.push(Policy::Random);
    for p in all {
        assert_eq!(
            Policy::parse(&p.name()),
            Some(p),
            "{p:?}: name does not parse back"
        );
        let m = Matchup {
            rules: canon(),
            policies: [p, Policy::Bot],
            reaction: [1, 1],
            setup: Setup::None,
            arcanas: [ArcanaPreset::NEUTRAL; 2],
        };
        // `play` panics if a match does not end; rejections are fine.
        let r = run_pairs(&[m], 0..2, 4).remove(0);
        let bought: u32 = r
            .iter()
            .flat_map(|g| g.iter())
            .flat_map(|g| g.sides[0].purchases.iter())
            .map(|&n| u32::from(n))
            .sum();
        assert!(bought > 0, "{p:?} never bought anything");
    }
}
