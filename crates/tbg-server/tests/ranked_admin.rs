//! P60 §7 corrections (GAME-DESIGN §14 BR-RI, D-87/D-89): frozen-delta
//! replay of only the two affected accounts, append-only decisions,
//! idempotent apply, revision lease, floor behavior and the P61 cohort
//! following the latest status.

use rusqlite::Connection;
use tbg_core::ranked_integrity::RiskState;
use tbg_core::{ArcanaPreset, EndReason, Outcome, Seat};
use tbg_server::ranked::{EndDetail, IntegritySeat, JournalEntry, MatchKey, RankedFinal};
use tbg_server::stats::eligible;
use tbg_server::store::integrity_admin::{apply_plan, plan, Action, AdminError, Applied};
use tbg_server::store::{AccountId, Store};

struct World {
    path: std::path::PathBuf,
    store: Store,
    ids: [AccountId; 3],
    seq: i64,
}

fn final_for(key: MatchKey, accounts: [AccountId; 2], winner: Seat) -> RankedFinal {
    RankedFinal {
        key,
        accounts,
        outcome: Outcome {
            winner: Some(winner),
            reason: EndReason::StrongholdDestroyed,
        },
        end_detail: EndDetail::Stronghold,
        combat_ticks: 2_000,
        wall_ms: 100_000,
        reached_running: true,
        ruleset_version: tbg_core::ruleset_version::RULESET_VERSION,
        ruleset_source_hash: tbg_server::ranked::source_hash(),
        seats: [IntegritySeat::default(); 2],
        factions: [[0; 7]; 2],
        arcanas: [ArcanaPreset::DEFAULT; 2],
        stats: None,
    }
}

impl World {
    async fn new(name: &str) -> World {
        let dir = std::env::temp_dir().join(format!("tbg-p60-admin-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("db.sqlite3");
        let store = Store::open(&path).unwrap();
        let mut ids = [0; 3];
        for (i, name) in ["Ari", "Bel", "Cai"].into_iter().enumerate() {
            ids[i] = store
                .register(name.into(), [i as u8 + 1; 32])
                .await
                .unwrap()
                .0;
        }
        World {
            path,
            store,
            ids,
            seq: 0,
        }
    }

    /// Settle `a` vs `b` with `winner`; returns the frozen applied deltas.
    async fn play(&mut self, key: u8, a: usize, b: usize, winner: Seat) -> [i32; 2] {
        self.seq += 1;
        let r = self
            .store
            .settle_ranked(JournalEntry {
                completion_seq: self.seq,
                ended_unix: 1_790_000_000 + self.seq,
                result: final_for([key; 16], [self.ids[a], self.ids[b]], winner),
            })
            .await
            .unwrap();
        r.results
            .map(|(before, after, _)| i32::from(after) - i32::from(before))
    }

    fn conn(&self) -> Connection {
        Connection::open(&self.path).unwrap()
    }

    fn rating(&self, i: usize) -> (u16, u32) {
        self.conn()
            .query_row(
                "SELECT rating, ranked_games FROM accounts WHERE id = ?1",
                [self.ids[i]],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap()
    }
}

fn op(n: u8) -> String {
    format!("{n:032x}")
}

#[tokio::test]
async fn invalidate_then_restore_replays_frozen_deltas_only_for_the_pair() {
    let mut w = World::new("restore").await;
    let d1 = w.play(1, 0, 1, Seat::A).await; // Ari beats Bel
    let d2 = w.play(2, 0, 2, Seat::A).await; // Ari beats Cai
    let d3 = w.play(3, 1, 0, Seat::A).await; // Bel beats Ari
    let original = [w.rating(0), w.rating(1), w.rating(2)];
    let mut conn = w.conn();
    let p = plan(
        &conn,
        &[1; 16],
        Action::Invalidate,
        "test evidence",
        "owner",
        op(1),
        0,
    )
    .unwrap();
    assert_eq!(p.risk_after, RiskState::Invalidated);
    // Ari keeps match 2 then match 3 (frozen deltas); Bel keeps only match 3.
    assert_eq!(p.after[0].rating as i32, 1000 + d2[0] + d3[1]);
    assert_eq!(
        (p.after[0].games, p.after[0].wins, p.after[0].losses),
        (2, 1, 1)
    );
    assert_eq!(p.after[1].rating as i32, 1000 + d3[0]);
    assert_eq!(apply_plan(&mut conn, &p).unwrap(), Applied::Committed);
    assert_eq!(apply_plan(&mut conn, &p).unwrap(), Applied::AlreadyApplied);
    assert_eq!(
        w.rating(2),
        original[2],
        "Cai's match 2 delta is frozen, never rerated"
    );
    let zero: i64 = conn
        .query_row(
            "SELECT sum(abs(effective_delta)) FROM ranked_sides WHERE match_id = 1",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(zero, 0);
    // Restore: the same log replays to exactly the original state.
    let r = plan(
        &conn,
        &[1; 16],
        Action::Restore,
        "appeal upheld",
        "owner",
        op(2),
        0,
    )
    .unwrap();
    assert_eq!(apply_plan(&mut conn, &r).unwrap(), Applied::Committed);
    assert_eq!([w.rating(0), w.rating(1), w.rating(2)], original);
    let originals: i64 = conn
        .query_row(
            "SELECT original_delta FROM ranked_sides WHERE match_id = 1 AND seat = 0",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(originals as i32, d1[0], "original rows never change");
    let decisions: i64 = conn
        .query_row("SELECT count(*) FROM ranked_decisions", [], |r| r.get(0))
        .unwrap();
    assert_eq!(decisions, 2);
}

#[tokio::test]
async fn stale_plans_and_invalid_transitions_are_refused() {
    let mut w = World::new("lease").await;
    w.play(1, 0, 1, Seat::A).await;
    let mut conn = w.conn();
    assert!(matches!(
        plan(
            &conn,
            &[1; 16],
            Action::Clear,
            "nothing to clear",
            "owner",
            op(1),
            0
        ),
        Err(AdminError::BadTransition(RiskState::Normal, Action::Clear))
    ));
    assert!(matches!(
        plan(
            &conn,
            &[9; 16],
            Action::Review,
            "unknown",
            "owner",
            op(1),
            0
        ),
        Err(AdminError::UnknownMatch)
    ));
    assert!(matches!(
        plan(&conn, &[1; 16], Action::Review, "", "owner", op(1), 0),
        Err(AdminError::BadInput(_))
    ));
    let p = plan(
        &conn,
        &[1; 16],
        Action::Review,
        "manual look",
        "owner",
        op(1),
        0,
    )
    .unwrap();
    // A settlement after planning moves the revision: the lease fails.
    w.play(2, 1, 2, Seat::B).await;
    assert!(matches!(
        apply_plan(&mut conn, &p),
        Err(AdminError::StalePlan)
    ));
    let fresh = plan(
        &conn,
        &[1; 16],
        Action::Review,
        "manual look",
        "owner",
        op(1),
        0,
    )
    .unwrap();
    assert_eq!(apply_plan(&mut conn, &fresh).unwrap(), Applied::Committed);
}

#[tokio::test]
async fn replay_respects_the_floor_so_a_correction_is_not_a_subtraction() {
    let mut w = World::new("floor").await;
    let d1 = w.play(1, 0, 1, Seat::A).await; // Bel loses -20
    let d2 = w.play(2, 1, 2, Seat::A).await; // Bel wins
                                             // A legacy baseline near the floor (pre-cutover history).
    w.conn()
        .execute(
            "UPDATE ranked_baseline SET rating = 105 WHERE account_id = ?1",
            [w.ids[1]],
        )
        .unwrap();
    let conn = w.conn();
    let p = plan(&conn, &[2; 16], Action::Review, "check", "owner", op(1), 0).unwrap();
    // Match 2 is Bel (seat A) vs Cai: Bel is `after[0]`. Replaying match 1
    // from 105: apply(105, -20) clamps at 100, then +d2.
    assert_eq!(d1[1], -20);
    assert_eq!(p.after[0].id, w.ids[1]);
    assert_eq!(p.after[0].rating as i32, 100 + d2[0]);
    assert_ne!(p.after[0].rating as i32, 105 + d1[1] + d2[0]);
}

#[tokio::test]
async fn invalidation_and_restore_move_the_match_out_of_and_back_into_the_cohort() {
    let mut w = World::new("cohort").await;
    w.play(1, 0, 1, Seat::A).await;
    let mut conn = w.conn();
    let version = tbg_core::ruleset_version::RULESET_VERSION;
    let cohort = |conn: &Connection| {
        let (catalog, rows) = tbg_server::store::stats::read_cohort(conn, version).unwrap();
        let catalog = tbg_server::stats::Catalog::parse(&catalog.unwrap()).unwrap();
        eligible(&rows, &catalog).1
    };
    let p = plan(
        &conn,
        &[1; 16],
        Action::Invalidate,
        "evidence",
        "owner",
        op(1),
        0,
    )
    .unwrap();
    apply_plan(&mut conn, &p).unwrap();
    assert_eq!(cohort(&conn).invalidated, 1);
    let r = plan(
        &conn,
        &[1; 16],
        Action::Restore,
        "cleared",
        "owner",
        op(2),
        0,
    )
    .unwrap();
    apply_plan(&mut conn, &r).unwrap();
    let cov = cohort(&conn);
    assert_eq!(
        (cov.invalidated, cov.missing_summary),
        (0, 1),
        "back in, no summary stored"
    );
}
