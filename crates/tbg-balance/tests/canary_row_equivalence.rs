//! C7's measured row and consumed flags must equal the full reference matrix exactly.
#[path = "common/canary_row.rs"]
mod canary_row;
use tbg_balance::duels::{self, Form};
use tbg_core::{Ruleset, UnitKind};

#[test]
fn targeted_rows_preserve_every_budget_seat_average_and_consumed_flag() {
    let forms: Vec<Form> = [
        UnitKind::Archer,
        UnitKind::SiegeWalker,
        UnitKind::BoneBrute,
        UnitKind::Bloomkeeper,
    ]
    .into_iter()
    .map(|kind| Form {
        kind,
        tier: 0,
        branch: 0,
    })
    .collect();
    for (hp, damage, cost) in [(1, 1, 1), (2, 2, 1), (1, 1, 3)] {
        let mut rules = Ruleset::canon().clone();
        let unit = &mut rules.units[UnitKind::SiegeWalker as usize];
        unit.hp *= hp;
        unit.damage *= damage;
        unit.cost *= cost;
        let full = duels::matrix_of(&rules, forms.clone(), 4);
        let row = canary_row::measure(&rules, &forms, 1);
        for budget in 0..duels::BUDGETS.len() {
            assert_eq!(row.scores[budget][1], full.scores[budget][1]);
            assert_eq!(
                row.average_among(budget, 1, |_| true),
                full.average_among(budget, 1, |_| true)
            );
        }
        let a = full.flags(&rules);
        let b = row.flags(&rules);
        assert_eq!(a.oppressive.contains(&1), b.oppressive.contains(&1));
        assert_eq!(a.weak.contains(&1), b.weak.contains(&1));
    }
}
