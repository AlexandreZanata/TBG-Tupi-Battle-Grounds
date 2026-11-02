//! OG-01 bounded D-33 evidence: requested Ogre Archer range change, not a competitive release gate.
use tbg_balance::duels::{self, Form};
use tbg_core::{Faction, Ruleset, UnitKind::*};
fn main() {
    let proposed = Ruleset::canon();
    let mut previous = proposed.clone();
    let old = &mut previous.units[SiegeWalker as usize];
    old.faction = Faction::Machines;
    old.range = 6000;
    old.forms[0].range_add = 2000;
    let proposed = duels::arena(&proposed);
    let previous = duels::arena(&previous);
    println!("revision,kind,tier,branch,opponent,seat_a_score,seat_b_score,mean_score,budget");
    for (tier, branch) in [(0, 0), (1, 0), (2, 0), (1, 1), (2, 1)] {
        let a = Form {
            kind: SiegeWalker,
            tier,
            branch,
        };
        for kind in [Shieldbearer, Archer, Longaxe, OldFinn, Mantis] {
            let b = Form {
                kind,
                tier: 0,
                branch: 0,
            };
            for (label, rules) in [("before", previous), ("proposed", proposed)] {
                row(label, rules, a, b);
            }
        }
    }
    for kind in [
        BasaltBulwark,
        CinderMaul,
        HookBrawler,
        FlintRunner,
        ClayBinder,
        GustDrummer,
        PebbleSlinger,
        RivetBallista,
    ] {
        for opponent in [Shieldbearer, Archer, Longaxe] {
            row(
                "new-preview",
                proposed,
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
    let one = duels::duel(rules, a, b, 20).unwrap();
    let two = 1.0 - duels::duel(rules, b, a, 20).unwrap();
    println!(
        "{label},{:?},{},{},{:?},{one},{two},{},20",
        a.kind,
        a.tier,
        a.branch,
        b.kind,
        (one + two) / 2.0
    );
}
