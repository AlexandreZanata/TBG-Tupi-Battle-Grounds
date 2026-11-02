//! AB-01 bounded D-33 screen of staged kinds and the owner-requested Trident transfer.
use tbg_balance::duels::{self, Form};
use tbg_core::{Faction, Ruleset, UnitKind::*};
fn main() {
    let current = Ruleset::canon();
    let mut before = current.clone();
    before.units[Trident as usize].faction = Faction::Plants;
    let current = duels::arena(&current);
    let before = duels::arena(&before);
    println!("revision,kind,tier,branch,opponent,seat_a_score,seat_b_score,mean_score,budget");
    if std::env::args().any(|a| a == "--changed-only") {
        for kind in [NacreDuelist, PearlSlinger] {
            if std::env::args().any(|a| a == "--pearl-only") && kind != PearlSlinger {
                continue;
            }
            for opponent in [Shieldbearer, Archer, Longaxe] {
                row(
                    "candidate-final",
                    current,
                    Form {
                        kind,
                        tier: 0,
                        branch: 0,
                    },
                    Form {
                        kind: opponent,
                        tier: 0,
                        branch: 0,
                    },
                );
            }
        }
        return;
    }
    for (tier, branch) in [(0, 0), (1, 0), (2, 0), (1, 1), (2, 1)] {
        for kind in [Shieldbearer, Archer, Longaxe] {
            for (label, rules) in [("before-transfer", before), ("proposed-transfer", current)] {
                row(
                    label,
                    rules,
                    Form {
                        kind: Trident,
                        tier,
                        branch,
                    },
                    Form {
                        kind,
                        tier: 0,
                        branch: 0,
                    },
                );
            }
        }
    }
    for kind in [
        NacreDuelist,
        ReefLancer,
        TrenchFist,
        PearlSlinger,
        AbyssHarpooner,
        UndertowHerald,
        TrenchLamplighter,
        PodSnapper,
    ] {
        for opponent in [Shieldbearer, Archer, Longaxe] {
            row(
                "new-preview",
                current,
                Form {
                    kind,
                    tier: 0,
                    branch: 0,
                },
                Form {
                    kind: opponent,
                    tier: 0,
                    branch: 0,
                },
            );
        }
    }
}
fn row(label: &str, rules: &'static Ruleset, a: Form, b: Form) {
    let first = duels::duel(rules, a, b, 20).unwrap();
    let second = 1.0 - duels::duel(rules, b, a, 20).unwrap();
    println!(
        "{label},{:?},{},{},{:?},{first},{second},{},20",
        a.kind,
        a.tier,
        a.branch,
        b.kind,
        (first + second) / 2.0
    );
}
