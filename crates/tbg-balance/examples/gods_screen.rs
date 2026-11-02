//! GD-04 bounded seat-swapped screen: Volthar against swarms, Astreon against tanks,
//! and the three Gods against each other, with Evolution cost inside each budget.
use tbg_balance::duels::{self, Form};
use tbg_core::{Ruleset, UnitKind, UnitKind::*};

fn form(kind: UnitKind, tier: u8, branch: u8) -> Form {
    Form { kind, tier, branch }
}

/// Mean score of `a` over both seat orders, or `None` when a side cannot field a unit.
fn score(rules: &'static Ruleset, a: Form, b: Form, budget: u32) -> Option<f64> {
    let first = duels::duel(rules, a, b, budget)?;
    let second = 1.0 - duels::duel(rules, b, a, budget)?;
    Some((first + second) / 2.0)
}

fn main() {
    let rules = duels::arena(Ruleset::canon());
    let gods = [
        form(Volthar, 0, 0),
        form(Volthar, 2, 0),
        form(Volthar, 2, 1),
        form(Astreon, 0, 0),
        form(Astreon, 2, 0),
        form(Astreon, 2, 1),
    ];
    let swarm = [
        Archer,
        Shieldbearer,
        Sawbot,
        Ravager,
        Bloomkeeper,
        Farwatch,
        Mantis,
    ];
    let tanks = [Colossus, BoneBrute, Rampart, Bonehorn, Shieldbearer];
    println!("god,tier,branch,opponent,opp_tier,budget,god_score");
    for god in gods {
        let panel: &[UnitKind] = if god.kind == Volthar { &swarm } else { &tanks };
        for &kind in panel.iter().chain([Colossus, Volthar, Astreon].iter()) {
            if kind == god.kind {
                continue;
            }
            for (tier, budget) in [(0, 24), (0, 48), (2, 48)] {
                let other = form(kind, tier, 0);
                if let Some(s) = score(rules, god, other, budget) {
                    println!(
                        "{:?},{},{},{kind:?},{tier},{budget},{s}",
                        god.kind, god.tier, god.branch
                    );
                }
            }
        }
    }
}
