//! INT-05: the C7 healing control has headroom and detects the unchanged planted mutation.
#[path = "common/wounded_army.rs"]
mod wounded_army;

use tbg_core::{Ruleset, UnitKind};

#[test]
fn controlled_wounded_armies_detect_double_hp_and_heal_on_256_pairs() {
    let base = Ruleset::canon();
    let mut strong = base.clone();
    let healer = UnitKind::Bloomkeeper;
    strong.units[healer as usize].hp *= 2;
    strong.units[healer as usize].heal *= 2;
    let plain = wounded_army::score(base, healer, 256);
    let buffed = wounded_army::score(&strong, healer, 256);
    println!("C7 wounded armies: {plain:.6} -> {buffed:.6}");
    assert!(
        plain <= 0.90,
        "the control must leave 10 percentage points of headroom"
    );
    assert!(
        buffed >= plain + 0.10,
        "C7: doubled HP/heal must improve at least 10 points"
    );
}
