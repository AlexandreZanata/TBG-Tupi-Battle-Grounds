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

/// P44–P47 (Proposed, staged, GAME-DESIGN §11): waves A–D in `UnitKind`
/// order. Not in the Shop pool until the P53 owner switch; checked against
/// the §11 starting values in the document's units.
#[rustfmt::skip]
const STAGED: [Row; 15] = {
    use Archetype::*; use Faction::*; use UnitKind::*;
    #[allow(clippy::too_many_arguments)]
    const fn r(kind: UnitKind, faction: Faction, archetype: Archetype,
        cost: u32, hp: i32, damage: i32, heal: i32, every_cs: u32, range_ct: i32,
        speed_ct: i32, splash_ct: i32, on_hit: Option<OnHit>) -> Row {
        Row { kind, faction, archetype, cost, hp, damage, heal, every_cs, range_ct,
              speed_ct, splash_ct, on_hit }
    }
    [
        r(Longaxe,      Humans,   Dps,     4,  360, 100, 0, 240, 160, 70,    0, None),
        r(Farwatch,     Humans,   Ranged,  3,  150, 70,  0, 400, 1200, 90,   0, None),
        r(Bombard,      Machines, Ranged,  5, 1000, 24,  0, 220, 400, 50, 100, None),
        r(Rotcap,       Plants,   Ranged,  3,  200, 12,  0, 150, 350, 90,    0, None),
        r(OldFinn,      Humans,   Control, 3,  240, 8,   0, 200, 350, 90,    0, None),
        r(AntidoteBrewer, Plants, Support, 3,  260, 0,   0, 400, 300, 90,    0, None),
        // P46: speed 28 (0.56 t/s) stands in for the model's 0.55: milli-tiles
        // are integers, and ±0.01 t/s is model noise.
        r(Shatterback,  Plants,   Tank,    4,  560, 30,  0, 150, 80,  56,    0, None),
        r(BloodFurnace, Undead,   Dps,     4,  440, 36,  0, 180, 80,  90,    0, None),
        r(Bloodweaver,  Undead,   Support, 3,  300, 0,   0, 500, 300, 90,    0, None),
        // P47: Drumguard range is the 3.0-tile hold radius (the bond reach
        // 2.0 lives in March.reach); Oilpress Every is 2.5 s.
        r(Oilpress,     Machines, Tank,    7, 3600, 25,  0, 250, 80,  30,    0, None),
        r(Drumguard,    Humans,   Support, 3,  260, 0,   0, 100, 300, 100,   0, None),
        r(Leechroot,    Plants,   Dps,     3,  220, 10,  0, 160, 120, 90,    0, None),
        r(CargoCrab,    Machines, Tank,    5, 1400, 20,  0, 150, 80,  50,    0, None),
        r(Puppetmaster, Undead,   Control, 4,  260, 0,   0, 800, 400, 90,    0, None),
        r(Smuggler,     Humans,   Support, 6,  320, 0,   0, 2000, 500, 60,    0, None),
    ]
};

#[test]
fn staged_waves_match_section_11() {
    for row in &STAGED {
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
    // `UnitKind` order; every archetype present. P44 stages three more
    // conventional kinds, P45 another three, P46 another three and P47 two
    // more (§11, not in the Shop pool until P53); the pool itself stays
    // exactly the ten purchasable units.
    // D-66: the four Monsters base kinds follow the 27 of `ALL_KINDS`, in
    // `UnitKind` order, and stay out of the pool and out of `ALL_KINDS`.
    let kinds: Vec<UnitKind> = UNITS.iter().map(|u| u.kind).collect();
    assert_eq!(UNITS.len(), 31);
    assert_eq!(&kinds[..ALL_KINDS.len()], &ALL_KINDS[..]);
    assert_eq!(
        &kinds[ALL_KINDS.len()..],
        &[
            UnitKind::Bonehorn,
            UnitKind::Ravager,
            UnitKind::Acidmaw,
            UnitKind::Whisperer,
        ][..]
    );
    assert_eq!(UNIT_KINDS.len(), 10);
    assert!(UNIT_KINDS.iter().all(|k| k.is_conventional()));
    for (f, n) in [
        (Faction::Humans, 7),
        (Faction::Undead, 5),
        (Faction::Machines, 6),
        (Faction::Plants, 7),
    ] {
        let sold = UNITS
            .iter()
            .filter(|u| u.faction == f && u.kind.is_conventional());
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

/// The Smuggler's cargo crate (P48-B): a summon kind with 120 HP, no
/// attack and no movement, never in the Shop pool.
#[test]
fn cargo_crate_is_a_summon_kind() {
    let u = def(UnitKind::Crate);
    assert!(u.kind.is_summon());
    assert_eq!((u.cost, u.hp, u.damage), (0, 120, 0));
    assert_eq!(u.speed, 0, "crates never move");
    assert!(!UNIT_KINDS.contains(&u.kind), "never in the Shop");
    assert!(
        !ALL_KINDS
            .into_iter()
            .filter(|k| k.is_conventional())
            .any(|k| k == u.kind),
        "never counted as conventional"
    );
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
Beetling/Blight: hp=5000 dmg=5000 atk=0 speed=0 range+=0 splash+=0 heal=0 dmg+=0 hit=-
Longaxe/Deep Cut: hp=0 dmg=3000 atk=0 speed=0 range+=0 splash+=0 heal=0 dmg+=0 hit=-
Longaxe/Crushing Edge: hp=0 dmg=3000 atk=0 speed=0 range+=0 splash+=0 heal=0 dmg+=0 hit=-
Longaxe/Sweeping Arc: hp=0 dmg=0 atk=0 speed=0 range+=0 splash+=800 heal=0 dmg+=0 hit=-
Longaxe/Line Breaker: hp=0 dmg=0 atk=0 speed=0 range+=0 splash+=400 heal=0 dmg+=0 hit=-
Farwatch/Steady Hands: hp=0 dmg=0 atk=2500 speed=0 range+=0 splash+=0 heal=0 dmg+=0 hit=-
Farwatch/Finishing Shot: hp=0 dmg=0 atk=0 speed=0 range+=0 splash+=0 heal=0 dmg+=0 hit=-
Farwatch/Crippling Arrow: hp=0 dmg=0 atk=0 speed=0 range+=0 splash+=0 heal=0 dmg+=0 hit=slow 2500bp/30t
Farwatch/Deep Wound: hp=0 dmg=0 atk=0 speed=0 range+=0 splash+=0 heal=0 dmg+=0 hit=-
Bombard/Bigger Blast: hp=0 dmg=0 atk=0 speed=0 range+=0 splash+=500 heal=0 dmg+=0 hit=-
Bombard/Heavy Ordnance: hp=0 dmg=2500 atk=0 speed=0 range+=0 splash+=500 heal=0 dmg+=0 hit=-
Bombard/Reinforced Hull: hp=5000 dmg=0 atk=0 speed=0 range+=0 splash+=0 heal=0 dmg+=0 hit=-
Bombard/Relentless Shelling: hp=0 dmg=0 atk=4300 speed=0 range+=0 splash+=0 heal=0 dmg+=0 hit=-
Rotcap/Potent Toxin: hp=0 dmg=0 atk=0 speed=0 range+=0 splash+=0 heal=0 dmg+=0 hit=-
Rotcap/Deadly Venom: hp=0 dmg=0 atk=0 speed=0 range+=0 splash+=0 heal=0 dmg+=0 hit=-
Rotcap/Spore Split: hp=0 dmg=0 atk=0 speed=0 range+=0 splash+=0 heal=0 dmg+=0 hit=-
Rotcap/Toxic Rain: hp=0 dmg=0 atk=0 speed=0 range+=0 splash+=0 heal=0 dmg+=0 hit=-
Old Finn/Big Catch: hp=0 dmg=0 atk=0 speed=0 range+=0 splash+=0 heal=0 dmg+=0 hit=-
Old Finn/Giant Catfish: hp=0 dmg=0 atk=0 speed=0 range+=0 splash+=0 heal=0 dmg+=0 hit=-
Old Finn/Double Catch: hp=0 dmg=0 atk=0 speed=0 range+=0 splash+=0 heal=0 dmg+=0 hit=-
Old Finn/Fish Shower: hp=0 dmg=0 atk=0 speed=0 range+=0 splash+=0 heal=0 dmg+=0 hit=-
Antidote Brewer/Master Antidote: hp=0 dmg=0 atk=0 speed=0 range+=0 splash+=0 heal=0 dmg+=0 hit=-
Antidote Brewer/Perfect Remedy: hp=0 dmg=0 atk=0 speed=0 range+=0 splash+=0 heal=0 dmg+=0 hit=-
Antidote Brewer/Quick Mixing: hp=0 dmg=0 atk=3333 speed=0 range+=0 splash+=0 heal=0 dmg+=0 hit=-
Antidote Brewer/Double Flask: hp=0 dmg=0 atk=0 speed=0 range+=0 splash+=0 heal=0 dmg+=0 hit=-
Shatterback/Thornplates: hp=0 dmg=0 atk=0 speed=0 range+=0 splash+=0 heal=0 dmg+=0 hit=-
Shatterback/Thorn Shrapnel: hp=0 dmg=0 atk=0 speed=0 range+=0 splash+=0 heal=0 dmg+=0 hit=-
Shatterback/Unleashed: hp=0 dmg=0 atk=0 speed=0 range+=0 splash+=0 heal=0 dmg+=0 hit=-
Shatterback/Rampage: hp=0 dmg=0 atk=0 speed=0 range+=0 splash+=0 heal=0 dmg+=0 hit=-
Blood Furnace/Infernal Core: hp=0 dmg=0 atk=0 speed=0 range+=0 splash+=0 heal=0 dmg+=0 hit=-
Blood Furnace/Dying Sun: hp=0 dmg=0 atk=0 speed=0 range+=0 splash+=0 heal=0 dmg+=0 hit=-
Blood Furnace/Efficient Combustion: hp=0 dmg=0 atk=0 speed=0 range+=0 splash+=0 heal=0 dmg+=0 hit=-
Blood Furnace/Eternal Embers: hp=0 dmg=0 atk=0 speed=0 range+=0 splash+=0 heal=0 dmg+=0 hit=-
Bloodweaver/Deep Transfer: hp=0 dmg=0 atk=0 speed=0 range+=0 splash+=0 heal=0 dmg+=0 hit=-
Bloodweaver/Last Offering: hp=0 dmg=0 atk=0 speed=0 range+=0 splash+=0 heal=0 dmg+=0 hit=-
Bloodweaver/Rapid Circulation: hp=0 dmg=0 atk=2500 speed=0 range+=0 splash+=0 heal=0 dmg+=0 hit=-
Bloodweaver/Shared Blood: hp=0 dmg=0 atk=0 speed=0 range+=0 splash+=0 heal=0 dmg+=0 hit=-
Oilpress/Reinforced Hull: hp=35000 dmg=0 atk=0 speed=0 range+=0 splash+=0 heal=0 dmg+=0 hit=-
Oilpress/Immovable Mass: hp=35000 dmg=0 atk=0 speed=0 range+=0 splash+=0 heal=0 dmg+=0 hit=-
Oilpress/Oil Leak: hp=0 dmg=0 atk=0 speed=0 range+=0 splash+=0 heal=0 dmg+=0 hit=slow 3000bp/40t
Oilpress/Black Mire: hp=0 dmg=0 atk=0 speed=0 range+=0 splash+=1000 heal=0 dmg+=0 hit=slow 3000bp/40t
Drumguard/War March: hp=0 dmg=0 atk=0 speed=0 range+=0 splash+=0 heal=0 dmg+=0 hit=-
Drumguard/Grand March: hp=0 dmg=0 atk=0 speed=0 range+=0 splash+=0 heal=0 dmg+=0 hit=-
Drumguard/Iron Rhythm: hp=0 dmg=0 atk=0 speed=0 range+=0 splash+=0 heal=0 dmg+=0 hit=-
Drumguard/Unbreakable Rhythm: hp=0 dmg=0 atk=0 speed=0 range+=0 splash+=0 heal=0 dmg+=0 hit=-
Leechroot/Faster Feeding: hp=0 dmg=0 atk=0 speed=0 range+=0 splash+=0 heal=0 dmg+=0 hit=-
Leechroot/Endless Growth: hp=0 dmg=0 atk=0 speed=0 range+=0 splash+=0 heal=0 dmg+=0 hit=-
Leechroot/Life Drain: hp=0 dmg=0 atk=0 speed=0 range+=0 splash+=0 heal=0 dmg+=0 hit=-
Leechroot/Blood Hunger: hp=0 dmg=0 atk=0 speed=0 range+=0 splash+=0 heal=0 dmg+=0 hit=-
Cargo Crab/Reinforced Shell: hp=0 dmg=0 atk=0 speed=0 range+=0 splash+=0 heal=0 dmg+=0 hit=-
Cargo Crab/Emergency Shield: hp=0 dmg=0 atk=0 speed=0 range+=0 splash+=0 heal=0 dmg+=0 hit=-
Cargo Crab/Faster Legs: hp=0 dmg=0 atk=0 speed=0 range+=0 splash+=0 heal=0 dmg+=0 hit=-
Cargo Crab/Overdrive: hp=0 dmg=0 atk=0 speed=0 range+=0 splash+=0 heal=0 dmg+=0 hit=-
Puppetmaster/Painful Betrayal: hp=0 dmg=0 atk=0 speed=0 range+=0 splash+=0 heal=0 dmg+=0 hit=-
Puppetmaster/Cruel Stumble: hp=0 dmg=0 atk=0 speed=0 range+=0 splash+=0 heal=0 dmg+=0 hit=-
Puppetmaster/Faster Strings: hp=0 dmg=0 atk=3333 speed=0 range+=0 splash+=0 heal=0 dmg+=0 hit=-
Puppetmaster/Double Puppet: hp=0 dmg=0 atk=0 speed=0 range+=0 splash+=0 heal=0 dmg+=0 hit=-
Smuggler/Better Connections: hp=0 dmg=0 atk=0 speed=0 range+=0 splash+=0 heal=0 dmg+=0 hit=-
Smuggler/Black Market: hp=0 dmg=0 atk=0 speed=0 range+=0 splash+=0 heal=0 dmg+=0 hit=-
Smuggler/Faster Deliveries: hp=0 dmg=0 atk=1100 speed=0 range+=0 splash+=0 heal=0 dmg+=0 hit=-
Smuggler/Double Shipment: hp=0 dmg=0 atk=0 speed=0 range+=0 splash+=0 heal=0 dmg+=0 hit=-
Crate/Crate: hp=0 dmg=0 atk=0 speed=0 range+=0 splash+=0 heal=0 dmg+=0 hit=-
Crate/Crate: hp=0 dmg=0 atk=0 speed=0 range+=0 splash+=0 heal=0 dmg+=0 hit=-
Crate/Crate: hp=0 dmg=0 atk=0 speed=0 range+=0 splash+=0 heal=0 dmg+=0 hit=-
Crate/Crate: hp=0 dmg=0 atk=0 speed=0 range+=0 splash+=0 heal=0 dmg+=0 hit=-
Bonehorn/Ancestral Carapace: hp=0 dmg=0 atk=0 speed=0 range+=0 splash+=0 heal=0 dmg+=0 hit=-
Bonehorn/Stone Fangs: hp=0 dmg=0 atk=0 speed=0 range+=0 splash+=0 heal=0 dmg+=0 hit=-
Bonehorn/Ancestral Carapace: hp=0 dmg=0 atk=0 speed=0 range+=0 splash+=0 heal=0 dmg+=0 hit=-
Bonehorn/Stone Fangs: hp=0 dmg=0 atk=0 speed=0 range+=0 splash+=0 heal=0 dmg+=0 hit=-
Ravager/Voracious Hunt: hp=0 dmg=0 atk=0 speed=0 range+=0 splash+=0 heal=0 dmg+=0 hit=-
Ravager/Deep Frenzy: hp=0 dmg=0 atk=0 speed=0 range+=0 splash+=0 heal=0 dmg+=0 hit=-
Ravager/Voracious Hunt: hp=0 dmg=0 atk=0 speed=0 range+=0 splash+=0 heal=0 dmg+=0 hit=-
Ravager/Deep Frenzy: hp=0 dmg=0 atk=0 speed=0 range+=0 splash+=0 heal=0 dmg+=0 hit=-
Acidmaw/Abyssal Jaw: hp=0 dmg=0 atk=0 speed=0 range+=0 splash+=0 heal=0 dmg+=0 hit=-
Acidmaw/Corrosive Volley: hp=0 dmg=0 atk=0 speed=0 range+=0 splash+=0 heal=0 dmg+=0 hit=-
Acidmaw/Abyssal Jaw: hp=0 dmg=0 atk=0 speed=0 range+=0 splash+=0 heal=0 dmg+=0 hit=-
Acidmaw/Corrosive Volley: hp=0 dmg=0 atk=0 speed=0 range+=0 splash+=0 heal=0 dmg+=0 hit=-
Whisperer/Echo of Fear: hp=0 dmg=0 atk=0 speed=0 range+=0 splash+=0 heal=0 dmg+=0 hit=-
Whisperer/Mental Pulse: hp=0 dmg=0 atk=0 speed=0 range+=0 splash+=0 heal=0 dmg+=0 hit=-
Whisperer/Echo of Fear: hp=0 dmg=0 atk=0 speed=0 range+=0 splash+=0 heal=0 dmg+=0 hit=-
Whisperer/Mental Pulse: hp=0 dmg=0 atk=0 speed=0 range+=0 splash+=0 heal=0 dmg+=0 hit=-";

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
