//! GD-01 bounded seat-swapped counter panel, with Evolution cost included in Energy 24.
use tbg_balance::duels::{self, Form};
use tbg_core::{Ruleset, UnitKind::*};
fn main() {
    let tier = if std::env::args().any(|a| a == "--base") {
        0
    } else {
        2
    };
    let rules = duels::arena(Ruleset::canon());
    let god = Form {
        kind: Colossus,
        tier: 0,
        branch: 0,
    };
    println!("opponent,tier,branch,energy,seat,colossus_score");
    for (kind, branch) in [(Sawbot, 0), (Ravager, 1), (Farwatch, 0), (Mantis, 0)] {
        let other = Form { kind, tier, branch };
        for swapped in [false, true] {
            let score = if swapped {
                1.0 - duels::preview_duel(rules, other, god, 24).unwrap()
            } else {
                duels::preview_duel(rules, god, other, 24).unwrap()
            };
            println!(
                "{kind:?},{tier},{branch},24,{},{score}",
                if swapped { "B" } else { "A" }
            );
        }
    }
}
