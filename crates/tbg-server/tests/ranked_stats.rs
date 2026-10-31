//! P61 E1 definitions (GAME-DESIGN §15 BR-ST, D-88): exact side/match
//! denominators, binary WR versus score, eligibility exclusions, Wilson
//! examples and publication gates, using the plan's known fixtures (§11).

use serde_json::json;
use tbg_server::stats::aggregate::{build, GATE_SIDES};
use tbg_server::stats::{eligible, wilson, Catalog, KindInfo, MatchObs, SideData, UnitRow};
use tbg_server::store::StoredMatch;

fn catalog() -> Catalog {
    let kind = |name: &str, faction| KindInfo {
        name: name.into(),
        purchasable: true,
        faction,
        archetype: "Tank".into(),
    };
    Catalog {
        kinds: vec![kind("Shieldbearer", 0), kind("Archer", 0)],
        augments: vec!["Iron Will".into()],
        arenas: vec!["forest".into()],
        tick_hz: 20,
    }
}

fn obs(accounts: [i64; 2], winner: Option<usize>, a_units: Vec<UnitRow>) -> MatchObs {
    let side = |units| SideData {
        units,
        arcanas: vec![0; 15],
        ..SideData::default()
    };
    MatchObs {
        accounts,
        winner,
        end_detail: "stronghold".into(),
        combat_ticks: 2_000,
        wall_ms: 120_000,
        arena: 0,
        ended_unix: 1_790_000_000,
        before: [1000, 1000],
        sides: [side(a_units), side(Vec::new())],
    }
}

fn archer(bought: u32) -> UnitRow {
    UnitRow {
        kind: 1,
        bought,
        first_tick: Some(400),
        ..UnitRow::default()
    }
}

#[test]
fn known_fixture_two_matches_four_sides() {
    // One draw, one seat-A win; seat A of the win bought two Archers.
    let matches = [
        obs([1, 2], None, Vec::new()),
        obs([3, 4], Some(0), vec![archer(2)]),
    ];
    let r = build(&matches, &catalog());
    assert_eq!((r.matches, r.sides), (2, 4));
    let o = &r.overall;
    assert_eq!((o.w, o.l, o.d), (1, 1, 2));
    assert_eq!(o.win_rate(), Some(0.25), "draws are non-wins");
    assert_eq!(o.score_rate(), Some(0.5));
    assert_eq!(r.seat_a.win_rate(), Some(0.5));
    assert_eq!(r.seat_a.score_rate(), Some(0.75));
    let archer = r.units.iter().find(|e| e.label == "Archer").unwrap();
    assert_eq!(archer.yes.n as f64 / r.sides as f64, 0.25, "presence");
    assert_eq!(
        archer.amount as f64 / r.sides as f64,
        0.5,
        "copies per side"
    );
    assert_eq!(
        archer.amount as f64 / archer.yes.n as f64,
        2.0,
        "copies per buyer"
    );
    assert_eq!(archer.yes.win_rate(), Some(1.0));
    assert!(!archer.yes.passes_gate(), "far below the publication floor");
    // Equal ratings: residual = score - 0.5 exactly.
    assert_eq!(archer.yes.residual(), Some(0.5));
}

#[test]
fn mirror_exposure_counts_two_sides_not_two_wins() {
    let mut m = obs([1, 2], Some(0), vec![archer(1)]);
    m.sides[1].units = vec![archer(1)];
    let r = build(&[m], &catalog());
    let e = r.units.iter().find(|e| e.label == "Archer").unwrap();
    assert_eq!((e.yes.n, e.yes.w, e.yes.l), (2, 1, 1));
    assert_eq!(e.yes.win_rate(), Some(0.5));
}

#[test]
fn wilson_matches_the_plan_examples() {
    let (lo, hi) = wilson(50, 100).unwrap();
    assert_eq!(
        ((lo * 1000.0).round(), (hi * 1000.0).round()),
        (404.0, 596.0)
    );
    let (lo, hi) = wilson(30, 50).unwrap();
    assert_eq!(
        ((lo * 1000.0).round(), (hi * 1000.0).round()),
        (462.0, 724.0)
    );
    assert_eq!(wilson(0, 0), None, "no NaN for an empty group");
}

#[test]
fn gate_edges_on_sides_accounts_and_concentration() {
    // 5 accounts in a ring, each match one Archer buyer: 50 buying sides.
    let ring = |n: usize| -> Vec<MatchObs> {
        (0..n)
            .map(|i| {
                let a = (i % 5) as i64;
                obs([a, (a + 1) % 5], Some(0), vec![archer(1)])
            })
            .collect()
    };
    let r = build(&ring(GATE_SIDES as usize), &catalog());
    let e = r.units.iter().find(|e| e.label == "Archer").unwrap();
    assert_eq!(e.yes.n, 50);
    assert_eq!(e.yes.max_share_pct(), 20);
    assert!(e.yes.passes_gate());
    let r = build(&ring(GATE_SIDES as usize - 1), &catalog());
    let e = r.units.iter().find(|e| e.label == "Archer").unwrap();
    assert!(!e.yes.passes_gate(), "one below the side floor");
    // One account dominating: concentration fails even with enough sides.
    let mut crowd = ring(50);
    for m in crowd.iter_mut().take(30) {
        m.accounts[0] = 9;
    }
    let r = build(&crowd, &catalog());
    let e = r.units.iter().find(|e| e.label == "Archer").unwrap();
    assert!(e.yes.max_share_pct() > 40 && !e.yes.passes_gate());
}

fn stored(id: i64, risk: u8, disposition: u8, running: bool, payload: Option<&str>) -> StoredMatch {
    StoredMatch {
        id,
        accounts: [1, 2],
        winner: Some(0),
        end_detail: "stronghold".into(),
        combat_ticks: 2_000,
        ended_unix: 1_790_000_000,
        before: [1000, 1000],
        risk,
        disposition,
        reached_running: running,
        payload: payload.map(str::to_string),
    }
}

#[test]
fn eligibility_counts_every_exclusion() {
    let side = json!({
        "units": [[1, 1, 40, 0, 0, 0, 0]], "synergy_max": [0, 0, 0, 0],
        "synergy_ticks": [[0, 0], [0, 0], [0, 0], [0, 0]], "offered": [], "chosen": [],
        "rounds": [0, 0], "stronghold": [2, 0, 0], "arcanas": vec![0; 15], "complete": true,
    });
    let good = json!({"schema": 1, "catalog_hash": "0", "arena": 0, "combat_ticks": 2000,
                      "wall_ms": 1, "sides": [side.clone(), side]})
    .to_string();
    let newer = good.replace("\"schema\":1", "\"schema\":2");
    let rows = [
        stored(1, 0, 0, true, Some(&good)),
        stored(2, 1, 0, true, Some(&good)),  // Review
        stored(3, 2, 0, true, Some(&good)),  // Invalidated
        stored(4, 0, 1, true, Some(&good)),  // NoChange
        stored(5, 0, 0, false, Some(&good)), // before Running
        stored(6, 0, 0, true, None),         // missing summary
        stored(7, 0, 0, true, Some(&newer)), // unknown schema: gap, not zeros
    ];
    let (obs, cov) = eligible(&rows, &catalog());
    assert_eq!(obs.len(), 1);
    assert_eq!(
        (
            cov.stored,
            cov.eligible,
            cov.review,
            cov.invalidated,
            cov.no_change
        ),
        (7, 1, 1, 1, 1)
    );
    assert_eq!(
        (cov.not_started, cov.missing_summary, cov.unreadable_summary),
        (1, 1, 1)
    );
}

#[tokio::test]
async fn stored_catalog_and_summary_round_trip_into_the_cohort() {
    use tbg_core::{ArcanaPreset, EndReason, Outcome, Seat};
    use tbg_server::ranked::{EndDetail, IntegritySeat, JournalEntry, RankedFinal};
    use tbg_server::store::Store;

    let dir = std::env::temp_dir().join(format!("tbg-p61-cohort-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("db.sqlite3");
    let store = Store::open(&path).unwrap();
    let (a, _) = store.register("Cora".into(), [1; 32]).await.unwrap();
    let (b, _) = store.register("Dax".into(), [2; 32]).await.unwrap();
    let side = |units| {
        json!({
            "units": units, "synergy_max": [1, 0, 0, 0],
            "synergy_ticks": [[900, 0], [0, 0], [0, 0], [0, 0]], "offered": [[3, 1]],
            "chosen": [[3, 1]], "rounds": [1, 0], "stronghold": [2, 1, 0],
            "arcanas": vec![0; 15], "complete": true,
        })
    };
    let summary = json!({"schema": 1, "catalog_hash": "0", "arena": 2, "combat_ticks": 2400,
        "wall_ms": 130_000, "sides": [side(json!([[1, 3, 200, 1, 1, 1, 0]])), side(json!([]))]});
    let receipt = store
        .settle_ranked(JournalEntry {
            completion_seq: 1,
            ended_unix: 1_790_000_000,
            result: RankedFinal {
                key: [5; 16],
                accounts: [a, b],
                outcome: Outcome {
                    winner: Some(Seat::A),
                    reason: EndReason::StrongholdDestroyed,
                },
                end_detail: EndDetail::Stronghold,
                combat_ticks: 2_400,
                wall_ms: 130_000,
                reached_running: true,
                ruleset_version: tbg_core::ruleset_version::RULESET_VERSION,
                ruleset_source_hash: tbg_server::ranked::source_hash(),
                seats: [IntegritySeat::default(); 2],
                factions: [[3, 0, 0, 0, 0, 0, 0], [0; 7]],
                arcanas: [ArcanaPreset::DEFAULT; 2],
                stats: Some(summary.clone()),
            },
        })
        .await
        .unwrap();
    store
        .save_ranked_stats(receipt.sequence, summary)
        .await
        .unwrap();
    let conn = rusqlite::Connection::open(&path).unwrap();
    let (catalog_json, rows) =
        tbg_server::store::stats::read_cohort(&conn, tbg_core::ruleset_version::RULESET_VERSION)
            .unwrap();
    let catalog = Catalog::parse(&catalog_json.expect("registered catalog")).expect("catalog");
    assert_eq!(catalog.kinds.len(), tbg_core::types::ALL_KINDS.len());
    let (obs, cov) = eligible(&rows, &catalog);
    assert_eq!((cov.stored, cov.eligible), (1, 1));
    let r = build(&obs, &catalog);
    let archer = &r.units[1];
    assert_eq!(
        (archer.label.as_str(), archer.yes.n, archer.amount),
        ("Archer", 1, 3)
    );
    assert_eq!(r.arenas[2].matches, 1);
}
