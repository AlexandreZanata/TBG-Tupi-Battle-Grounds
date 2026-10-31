//! Ranked eligibility contract (GAME-DESIGN §14 BR-RI02, D-87): a budget
//! decision keeps both baseline deltas or zeroes both, never one side, and
//! never changes the Elo formula (P60 §2).

use tbg_core::ranked_integrity::{
    gate, BudgetFacts, BudgetPolicy, RatingDisposition, NEW_SOURCE_LIMIT_A, NEW_SOURCE_LIMIT_B,
    PAIR_LIMIT_A, PAIR_LIMIT_B,
};
use tbg_core::rating::{apply, delta, Score};

const POLICY: BudgetPolicy = BudgetPolicy {
    pair_limit: 120,
    immature_limit: 120,
};

/// Applied baseline deltas `b_i = apply(R_i, d_i) - R_i` (P60 §2).
fn base(ra: u16, rb: u16, ga: u32, gb: u32, a: Score, b: Score) -> [i32; 2] {
    let da = delta(ra, rb, ga, a);
    let db = delta(rb, ra, gb, b);
    [
        i32::from(apply(ra, da)) - i32::from(ra),
        i32::from(apply(rb, db)) - i32::from(rb),
    ]
}

#[test]
fn baseline_examples_match_the_plan_table() {
    use Score::{Loss, Win};
    assert_eq!(base(1000, 1000, 0, 0, Win, Loss), [20, -20]);
    assert_eq!(base(1000, 1000, 20, 20, Win, Loss), [12, -12]);
    assert_eq!(base(1000, 1000, 0, 20, Win, Loss), [20, -12]);
    assert_eq!(base(1000, 1000, 20, 0, Win, Loss), [12, -20]);
    assert_eq!(base(1000, 100, 20, 20, Win, Loss), [1, 0], "floor");
    assert_eq!(base(1400, 1000, 20, 20, Win, Loss), [2, -2]);
}

#[test]
fn whole_pair_binary_choice_br_ri02() {
    let cases: [([i32; 2], BudgetFacts, [i32; 2], u32); 6] = [
        // Under budget: unchanged.
        ([20, -20], BudgetFacts::default(), [20, -20], 0),
        // Exactly the limit is allowed.
        (
            [20, -20],
            BudgetFacts {
                pair_positive: [100, 0],
                ..BudgetFacts::default()
            },
            [20, -20],
            0,
        ),
        // One point over: both zero, never a partial award.
        (
            [20, -20],
            BudgetFacts {
                pair_positive: [101, 0],
                ..BudgetFacts::default()
            },
            [0, 0],
            PAIR_LIMIT_A,
        ),
        // A capped loser does not suppress the eligible winner's gain.
        (
            [20, -20],
            BudgetFacts {
                pair_positive: [0, 120],
                ..BudgetFacts::default()
            },
            [20, -20],
            0,
        ),
        // The newcomer-source budget applies only against an immature opponent.
        (
            [12, -20],
            BudgetFacts {
                immature_positive: [115, 0],
                opponent_immature: [false, true],
                ..BudgetFacts::default()
            },
            [12, -20],
            0,
        ),
        // Draw beneficiaries are checked too; seat B's bits are its own.
        (
            [-3, 3],
            BudgetFacts {
                pair_positive: [0, 118],
                immature_positive: [0, 118],
                opponent_immature: [false, true],
            },
            [0, 0],
            PAIR_LIMIT_B | NEW_SOURCE_LIMIT_B,
        ),
    ];
    for (base, facts, applied, bits) in cases {
        let d = gate(base, &facts, &POLICY);
        assert_eq!(
            (d.applied, d.reason_bits),
            (applied, bits),
            "{base:?} {facts:?}"
        );
        assert_eq!(d.disposition() == RatingDisposition::Applied, bits == 0);
    }
}

#[test]
fn seat_swap_is_symmetric() {
    for (pa, pb) in [(0, 0), (100, 0), (101, 119), (120, 121)] {
        for (ia, ib) in [(false, false), (true, false), (true, true)] {
            let facts = BudgetFacts {
                pair_positive: [pa, pb],
                immature_positive: [pa, pb],
                opponent_immature: [ia, ib],
            };
            let swapped = BudgetFacts {
                pair_positive: [pb, pa],
                immature_positive: [pb, pa],
                opponent_immature: [ib, ia],
            };
            for b in [[20, -20], [-13, 22], [1, 0], [0, 0]] {
                let d = gate(b, &facts, &POLICY);
                let s = gate([b[1], b[0]], &swapped, &POLICY);
                assert_eq!(d.applied, [s.applied[1], s.applied[0]]);
                let swap_bits = |x: u32| {
                    ((x & PAIR_LIMIT_A) << 1)
                        | ((x & PAIR_LIMIT_B) >> 1)
                        | ((x & NEW_SOURCE_LIMIT_A) << 1)
                        | ((x & NEW_SOURCE_LIMIT_B) >> 1)
                };
                assert_eq!(d.reason_bits, swap_bits(s.reason_bits));
            }
        }
    }
}
