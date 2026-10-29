//! Guard tests for the starter roster (docs/GAME-DESIGN.md §5 "Starter
//! roster" and "Evolution trees"). Base stats are checked against the
//! document's table in its own units; every evolution form is frozen field
//! by field. A failure means a balance value changed: update GAME-DESIGN
//! (**Proposed**) in the same commit, then this file. Content tables are
//! exempt from the file-size cap (AGENTS.md).

use tbg_core::content::{def, FormDef, UnitDef, ENERGY_SCALE, TICK_HZ, UNITS};
use tbg_core::{Archetype, Faction, OnHit, UnitKind, ALL_KINDS, UNIT_KINDS};

/// One §5 roster row: cost in Energy, attack period in centiseconds, range
/// and speed in centi-tiles (per second for speed), splash in centi-tiles.
struct Row {
    kind: UnitKind,
    faction: Faction,
    archetype: Archetype,
    cost: u32,
    hp: i32,
    damage: i32,
    heal: i32,
    every_cs: u32,
    range_ct: i32,
    speed_ct: i32,
    splash_ct: i32,
    on_hit: Option<OnHit>,
}

#[rustfmt::skip]
const ROSTER: [Row; 11] = {
    use Archetype::*; use Faction::*; use UnitKind::*;
    #[allow(clippy::too_many_arguments)] // one call per roster row reads as a table
    const fn r(kind: UnitKind, faction: Faction, archetype: Archetype,
        cost: u32, hp: i32, damage: i32, heal: i32, every_cs: u32, range_ct: i32,
        speed_ct: i32, splash_ct: i32, on_hit: Option<OnHit>) -> Row {
        Row { kind, faction, archetype, cost, hp, damage, heal, every_cs, range_ct,
              speed_ct, splash_ct, on_hit }
    }
    [
        r(Shieldbearer, Humans,   Tank,    4,  650, 30,  0, 120, 80,  80,   0, None),
        r(Archer,       Humans,   Ranged,  3,  220, 45,  0, 100, 450, 100,  0, None),
        r(BoneBrute,    Undead,   Tank,    4, 1300, 75,  0, 150, 100, 60,   0, None),
        // slow 40 % for 2 s
        r(Hexer,        Undead,   Control, 4,  300, 29,  0, 140, 350, 100,  0,
          Some(OnHit::Slow { bp: 4_000, ticks: 40 })),
        r(Sawbot,       Machines, Dps,     4,  231, 55,  0, 105, 80,  130,  0, None),
        r(SiegeWalker,  Machines, Ranged,  5,  700, 85,  0, 250, 600, 66, 150, None),
        // heals 40 instead of attacking
        r(Bloomkeeper,  Plants,   Support, 3,  280,  0, 40, 150, 300, 90,   0, None),
        // root 0.8 s
        r(Rootcaller,   Plants,   Control, 4,  300, 20,  0, 160, 300, 90,   0,
          Some(OnHit::Root { ticks: 16 })),
        // P16-B (D-32, Proposed): Hatch every 4 s; Guard (BR-U09).
        r(Nester,       Plants,   Support, 4,  300, 15,  0, 150, 250, 80,   0, None),
        r(Rampart,      Machines, Tank,    6, 1500, 35,  0, 150, 80,  50,   0, None),
        // summon (BR-U08): never sold, lifetime 10 s
        r(Beetling,     Plants,   Dps,     0,   90, 18,  0, 80,  60,  140,  0, None),
    ]
};

#[test]
fn base_stats_match_game_design_table() {
    for row in &ROSTER {
        let u: &UnitDef = def(row.kind);
        let name = u.name;
        assert_eq!(u.kind, row.kind, "UNITS order must follow UnitKind");
        assert_eq!(
            (u.faction, u.archetype),
            (row.faction, row.archetype),
            "{name}"
        );
        assert_eq!(u.cost, row.cost * ENERGY_SCALE, "{name} cost");
        assert_eq!(
            (u.hp, u.damage, u.heal),
            (row.hp, row.damage, row.heal),
            "{name} hp/dmg/heal"
        );
        assert_eq!(
            u32::from(u.interval) * 100,
            row.every_cs * TICK_HZ,
            "{name} attack period"
        );
        assert_eq!(u.range, row.range_ct * 10, "{name} range");
        assert_eq!(u.speed * TICK_HZ as i32, row.speed_ct * 10, "{name} speed");
        assert_eq!(u.splash, row.splash_ct * 10, "{name} splash");
        assert_eq!(u.on_hit, row.on_hit, "{name} on-hit");
    }
}

#[test]
fn roster_composition() {
    // §5 (D-32): ten purchasable units plus the Beetling summon, in
    // `UnitKind` order; every archetype present; the Shop pool is exactly the
    // purchasable units (BR-U08: summons are never sold).
    assert_eq!(UNITS.len(), 11);
    assert_eq!(UNIT_KINDS.len(), 10);
    assert!(UNIT_KINDS.iter().all(|k| !k.is_summon()));
    assert_eq!(
        UNITS.iter().map(|u| u.kind).collect::<Vec<_>>(),
        ALL_KINDS.to_vec()
    );
    for (f, n) in [
        (Faction::Humans, 2),
        (Faction::Undead, 2),
        (Faction::Machines, 3),
        (Faction::Plants, 3),
    ] {
        let sold = UNITS
            .iter()
            .filter(|u| u.faction == f && !u.kind.is_summon());
        assert_eq!(sold.count(), n, "{f:?}");
    }
    for a in [
        Archetype::Tank,
        Archetype::Dps,
        Archetype::Ranged,
        Archetype::Support,
        Archetype::Control,
    ] {
        assert!(UNITS.iter().any(|u| u.archetype == a), "{a:?} missing");
    }
}

fn hit(h: Option<OnHit>) -> String {
    match h {
        None => "-".into(),
        Some(OnHit::Slow { bp, ticks }) => format!("slow {bp}bp/{ticks}t"),
        Some(OnHit::Root { ticks }) => format!("root {ticks}t"),
    }
}

fn form_line(unit: &str, f: &FormDef) -> String {
    format!(
        "{unit}/{}: hp={} dmg={} atk={} speed={} range+={} splash+={} heal={} dmg+={} hit={}",
        f.name,
        f.hp_bp,
        f.dmg_bp,
        f.atk_bp,
        f.speed_bp,
        f.range_add,
        f.splash_add,
        f.heal_bp,
        f.dmg_add,
        hit(f.on_hit)
    )
}

/// Every evolution form (A1, A2, B1, B2 per unit). Basis points stack on the
/// base unit; T2 adds to T1 on the same branch.
///
/// Known open canon conflict (P32 reproduces it; the owner decides): the
/// executable P2 spec and the code give Rootcaller **Deep Roots root 32 t
/// (1.6 s)** and **Overgrowth root 28 t (1.4 s)**, while GAME-DESIGN §5
/// prose reads "+0.6 s" / "+0.4 s" (28 t / 24 t). This test freezes the
/// shipped values; change it only together with that decision.
#[rustfmt::skip]
const FORMS: &str = "\
Shieldbearer/Bulwark: hp=5000 dmg=0 atk=0 speed=0 range+=0 splash+=0 heal=0 dmg+=0 hit=-
Shieldbearer/Fortress: hp=5000 dmg=0 atk=0 speed=0 range+=0 splash+=0 heal=0 dmg+=0 hit=slow 3000bp/30t
Shieldbearer/Vanguard: hp=0 dmg=4000 atk=0 speed=2000 range+=0 splash+=0 heal=0 dmg+=0 hit=-
Shieldbearer/Champion: hp=0 dmg=4000 atk=0 speed=0 range+=0 splash+=1000 heal=0 dmg+=0 hit=-
Archer/Longbow: hp=0 dmg=2000 atk=0 speed=0 range+=1500 splash+=0 heal=0 dmg+=0 hit=-
Archer/Marksman: hp=0 dmg=6000 atk=0 speed=0 range+=0 splash+=0 heal=0 dmg+=0 hit=-
Archer/Volley: hp=0 dmg=0 atk=4000 speed=0 range+=0 splash+=0 heal=0 dmg+=0 hit=-
Archer/Arrow Rain: hp=0 dmg=0 atk=0 speed=0 range+=0 splash+=1000 heal=0 dmg+=0 hit=-
Bone Brute/Ossified: hp=5000 dmg=0 atk=0 speed=0 range+=0 splash+=0 heal=0 dmg+=0 hit=-
Bone Brute/Bone Wall: hp=5000 dmg=0 atk=0 speed=0 range+=0 splash+=0 heal=0 dmg+=0 hit=slow 3000bp/30t
Bone Brute/Butcher: hp=0 dmg=5000 atk=0 speed=0 range+=0 splash+=0 heal=0 dmg+=0 hit=-
Bone Brute/Reaper: hp=0 dmg=3000 atk=0 speed=0 range+=0 splash+=1200 heal=0 dmg+=0 hit=-
Hexer/Withering: hp=0 dmg=0 atk=0 speed=0 range+=0 splash+=0 heal=0 dmg+=0 hit=slow 6000bp/40t
Hexer/Doom: hp=0 dmg=6000 atk=0 speed=0 range+=0 splash+=0 heal=0 dmg+=0 hit=-
Hexer/Plague: hp=0 dmg=0 atk=0 speed=0 range+=0 splash+=1000 heal=0 dmg+=0 hit=-
Hexer/Pestilence: hp=0 dmg=4000 atk=0 speed=0 range+=0 splash+=500 heal=0 dmg+=0 hit=-
Sawbot/Overclock: hp=0 dmg=0 atk=4000 speed=0 range+=0 splash+=0 heal=0 dmg+=0 hit=-
Sawbot/Shredder: hp=0 dmg=0 atk=4000 speed=3000 range+=0 splash+=0 heal=0 dmg+=0 hit=-
Sawbot/Plating: hp=6000 dmg=0 atk=0 speed=0 range+=0 splash+=0 heal=0 dmg+=0 hit=-
Sawbot/Juggernaut: hp=6000 dmg=0 atk=0 speed=0 range+=0 splash+=800 heal=0 dmg+=0 hit=-
Siege Walker/Longshot: hp=0 dmg=0 atk=0 speed=0 range+=2000 splash+=0 heal=0 dmg+=0 hit=-
Siege Walker/Artillery: hp=0 dmg=5000 atk=0 speed=0 range+=0 splash+=0 heal=0 dmg+=0 hit=-
Siege Walker/Shrapnel: hp=0 dmg=0 atk=0 speed=0 range+=0 splash+=1000 heal=0 dmg+=0 hit=-
Siege Walker/Concussion: hp=0 dmg=0 atk=0 speed=0 range+=0 splash+=0 heal=0 dmg+=0 hit=root 16t
Bloomkeeper/Blossom: hp=0 dmg=0 atk=0 speed=0 range+=0 splash+=0 heal=5000 dmg+=0 hit=-
Bloomkeeper/Grove: hp=0 dmg=0 atk=0 speed=0 range+=0 splash+=1500 heal=0 dmg+=0 hit=-
Bloomkeeper/Bramble: hp=5000 dmg=0 atk=0 speed=0 range+=0 splash+=0 heal=0 dmg+=25 hit=-
Bloomkeeper/Thornwarden: hp=0 dmg=0 atk=0 speed=0 range+=0 splash+=0 heal=0 dmg+=0 hit=root 12t
Rootcaller/Deep Roots: hp=0 dmg=0 atk=0 speed=0 range+=0 splash+=0 heal=0 dmg+=0 hit=root 32t
Rootcaller/Strangle: hp=0 dmg=8000 atk=0 speed=0 range+=0 splash+=0 heal=0 dmg+=0 hit=-
Rootcaller/Spores: hp=0 dmg=0 atk=0 speed=0 range+=0 splash+=1000 heal=0 dmg+=0 hit=-
Rootcaller/Overgrowth: hp=0 dmg=0 atk=0 speed=0 range+=1000 splash+=0 heal=0 dmg+=0 hit=root 28t
Nester/Brood: hp=0 dmg=0 atk=0 speed=0 range+=0 splash+=0 heal=0 dmg+=0 hit=-
Nester/Hive: hp=0 dmg=0 atk=0 speed=0 range+=0 splash+=0 heal=0 dmg+=0 hit=-
Nester/Venom: hp=0 dmg=0 atk=0 speed=0 range+=0 splash+=0 heal=0 dmg+=0 hit=-
Nester/Blight: hp=0 dmg=0 atk=0 speed=0 range+=0 splash+=0 heal=0 dmg+=0 hit=-
Rampart/Bastion: hp=5000 dmg=0 atk=0 speed=0 range+=0 splash+=0 heal=0 dmg+=0 hit=-
Rampart/Citadel: hp=5000 dmg=0 atk=0 speed=0 range+=0 splash+=0 heal=0 dmg+=0 hit=slow 3000bp/30t
Rampart/Spiked: hp=0 dmg=6000 atk=0 speed=0 range+=0 splash+=0 heal=0 dmg+=0 hit=-
Rampart/Siegebreaker: hp=0 dmg=4000 atk=0 speed=0 range+=0 splash+=1000 heal=0 dmg+=0 hit=-
Beetling/Brood: hp=0 dmg=0 atk=0 speed=0 range+=0 splash+=0 heal=0 dmg+=0 hit=-
Beetling/Hive: hp=0 dmg=0 atk=0 speed=0 range+=0 splash+=0 heal=0 dmg+=0 hit=-
Beetling/Venom: hp=0 dmg=0 atk=0 speed=0 range+=0 splash+=0 heal=0 dmg+=0 hit=slow 3000bp/20t
Beetling/Blight: hp=5000 dmg=5000 atk=0 speed=0 range+=0 splash+=0 heal=0 dmg+=0 hit=-";

#[test]
fn evolution_forms_are_frozen() {
    let shipped: Vec<String> = UNITS
        .iter()
        .flat_map(|u| u.forms.iter().map(move |f| form_line(u.name, f)))
        .collect();
    let frozen: Vec<&str> = FORMS.lines().collect();
    assert_eq!(shipped.len(), frozen.len(), "form count changed");
    for (got, want) in shipped.iter().zip(frozen) {
        assert_eq!(got, want);
    }
}
