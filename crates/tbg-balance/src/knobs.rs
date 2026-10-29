//! The search space (P16-C step 8): every tunable number of a `Ruleset` as
//! a named knob with a step and bounds. `all` destructures `Ruleset`,
//! `UnitDef`, and `FormDef` without `..`, so a new field fails to compile
//! until it is registered here or marked untunable with a reason (C5).
//! A zero stays zero: the search never adds an ability a unit lacks.

use tbg_core::content::{FormDef, UnitDef, ENERGY_SCALE, MIN_INTERVAL};
use tbg_core::{Archetype, Ruleset, FACTIONS, UNIT_KINDS};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum UnitField {
    Cost,
    Hp,
    Damage,
    Interval,
    Range,
    Speed,
    Splash,
    Heal,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FormField {
    HpBp,
    DmgBp,
    AtkBp,
    SpeedBp,
    RangeAdd,
    SplashAdd,
    HealBp,
    DmgAdd,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Knob {
    EnergyStart,
    RegenRunning,
    InterestBand,
    InterestPerBand,
    InterestMaxBands,
    AugmentRoundTicks,
    AugmentWindow,
    AugmentForgo,
    AugmentCost(usize),
    EvolveCost(usize),
    StrongholdHp,
    StrongholdDamage,
    StrongholdInterval,
    StrongholdRange,
    /// Faction index, level index.
    Synergy(usize, usize),
    HatchEvery,
    HatchEveryHive,
    BeetleLifetime,
    /// `ALL_KINDS` index.
    Unit(usize, UnitField),
    /// `ALL_KINDS` index, form index `[A1, A2, B1, B2]`.
    Form(usize, usize, FormField),
    /// Arcana node index (bp per rank, GAME-DESIGN §9).
    Arcana(usize),
}

/// Arcana node names in serialization order (GAME-DESIGN §9).
const ARCANA_NAMES: [&str; 6] = [
    "guard",
    "assault",
    "piercing",
    "endurance",
    "resilience",
    "renewal",
];

/// Every knob whose canonical value is non-zero.
pub fn all(r: &Ruleset) -> Vec<Knob> {
    let Ruleset {
        energy_start,
        // Untunable: no gameplay cap. The lab builds `Some(cap)` directly.
        energy_cap: _,
        regen_running,
        interest,
        augment,
        evolve_cost,
        stronghold_hp,
        stronghold_damage,
        stronghold_interval,
        stronghold_range,
        // Untunable: P16-B asks for evidence before touching thresholds.
        synergy_thresholds: _,
        synergies,
        hatch_every,
        hatch_every_hive,
        // Untunable: the Hatch ability's own limits (BR-U08), not balance.
        beetles_per_nester: _,
        beetle_lifetime,
        units,
        arcana_bp,
    } = r;
    let mut v = Vec::new();
    let mut add = |k: Knob, value: i64| {
        if value != 0 {
            v.push(k);
        }
    };
    add(Knob::EnergyStart, i64::from(*energy_start));
    add(Knob::RegenRunning, i64::from(*regen_running));
    add(Knob::InterestBand, i64::from(interest.band));
    add(Knob::InterestPerBand, i64::from(interest.per_band));
    add(Knob::InterestMaxBands, i64::from(interest.max_bands));
    add(Knob::AugmentRoundTicks, i64::from(augment.round_ticks));
    add(Knob::AugmentWindow, i64::from(augment.window));
    add(Knob::AugmentForgo, i64::from(augment.forgo_energy));
    for (i, cost) in augment.costs.iter().enumerate() {
        add(Knob::AugmentCost(i), i64::from(*cost));
    }
    for (i, c) in evolve_cost.iter().enumerate() {
        add(Knob::EvolveCost(i), i64::from(*c));
    }
    add(Knob::StrongholdHp, i64::from(*stronghold_hp));
    add(Knob::StrongholdDamage, i64::from(*stronghold_damage));
    add(Knob::StrongholdInterval, i64::from(*stronghold_interval));
    add(Knob::StrongholdRange, i64::from(*stronghold_range));
    for (f, (_, bps)) in synergies.iter().enumerate() {
        for (l, bp) in bps.iter().enumerate() {
            add(Knob::Synergy(f, l), i64::from(*bp));
        }
    }
    add(Knob::HatchEvery, i64::from(*hatch_every));
    add(Knob::HatchEveryHive, i64::from(*hatch_every_hive));
    add(Knob::BeetleLifetime, i64::from(*beetle_lifetime));
    for (i, bp) in arcana_bp.iter().enumerate() {
        add(Knob::Arcana(i), i64::from(*bp));
    }
    for (k, u) in units.iter().enumerate() {
        let UnitDef {
            // Identity, never tuned.
            kind: _,
            name: _,
            faction: _,
            archetype: _,
            cost,
            hp,
            damage,
            interval,
            range,
            speed,
            splash,
            heal,
            // Untunable: slow/root rules are under review (P32).
            on_hit: _,
            forms,
        } = u;
        // Summons are tuned through their Nester's forms only.
        let shop = UNIT_KINDS.iter().any(|s| *s as usize == k);
        for (field, value) in [
            (UnitField::Cost, i64::from(*cost)),
            (UnitField::Hp, i64::from(*hp)),
            (UnitField::Damage, i64::from(*damage)),
            (UnitField::Interval, i64::from(*interval)),
            (UnitField::Range, i64::from(*range)),
            (UnitField::Speed, i64::from(*speed)),
            (UnitField::Splash, i64::from(*splash)),
            (UnitField::Heal, i64::from(*heal)),
        ] {
            if shop || field != UnitField::Cost {
                add(Knob::Unit(k, field), value);
            }
        }
        for (i, f) in forms.iter().enumerate() {
            let FormDef {
                name: _,
                hp_bp,
                dmg_bp,
                atk_bp,
                speed_bp,
                range_add,
                splash_add,
                heal_bp,
                dmg_add,
                on_hit: _,
            } = f;
            for (field, value) in [
                (FormField::HpBp, hp_bp),
                (FormField::DmgBp, dmg_bp),
                (FormField::AtkBp, atk_bp),
                (FormField::SpeedBp, speed_bp),
                (FormField::RangeAdd, range_add),
                (FormField::SplashAdd, splash_add),
                (FormField::HealBp, heal_bp),
                (FormField::DmgAdd, dmg_add),
            ] {
                add(Knob::Form(k, i, field), i64::from(*value));
            }
        }
    }
    v
}

fn unit_field(u: &mut UnitDef, f: UnitField) -> &mut dyn Field {
    match f {
        UnitField::Cost => &mut u.cost,
        UnitField::Hp => &mut u.hp,
        UnitField::Damage => &mut u.damage,
        UnitField::Interval => &mut u.interval,
        UnitField::Range => &mut u.range,
        UnitField::Speed => &mut u.speed,
        UnitField::Splash => &mut u.splash,
        UnitField::Heal => &mut u.heal,
    }
}

fn form_field(d: &mut FormDef, f: FormField) -> &mut dyn Field {
    match f {
        FormField::HpBp => &mut d.hp_bp,
        FormField::DmgBp => &mut d.dmg_bp,
        FormField::AtkBp => &mut d.atk_bp,
        FormField::SpeedBp => &mut d.speed_bp,
        FormField::RangeAdd => &mut d.range_add,
        FormField::SplashAdd => &mut d.splash_add,
        FormField::HealBp => &mut d.heal_bp,
        FormField::DmgAdd => &mut d.dmg_add,
    }
}

/// An integer field of any width, read and written as `i64`.
trait Field {
    fn get(&self) -> i64;
    fn set(&mut self, v: i64);
}

macro_rules! field {
    ($($t:ty),*) => {$(
        impl Field for $t {
            fn get(&self) -> i64 { i64::from(*self) }
            fn set(&mut self, v: i64) { *self = v.clamp(<$t>::MIN as i64, <$t>::MAX as i64) as $t; }
        }
    )*};
}
field!(u16, u32, i32);

fn slot(r: &mut Ruleset, k: Knob) -> &mut dyn Field {
    match k {
        Knob::EnergyStart => &mut r.energy_start,
        Knob::RegenRunning => &mut r.regen_running,
        Knob::InterestBand => &mut r.interest.band,
        Knob::InterestPerBand => &mut r.interest.per_band,
        Knob::InterestMaxBands => &mut r.interest.max_bands,
        Knob::AugmentRoundTicks => &mut r.augment.round_ticks,
        Knob::AugmentWindow => &mut r.augment.window,
        Knob::AugmentForgo => &mut r.augment.forgo_energy,
        Knob::AugmentCost(i) => &mut r.augment.costs[i],
        Knob::EvolveCost(i) => &mut r.evolve_cost[i],
        Knob::StrongholdHp => &mut r.stronghold_hp,
        Knob::StrongholdDamage => &mut r.stronghold_damage,
        Knob::StrongholdInterval => &mut r.stronghold_interval,
        Knob::StrongholdRange => &mut r.stronghold_range,
        Knob::Synergy(f, l) => &mut r.synergies[f].1[l],
        Knob::HatchEvery => &mut r.hatch_every,
        Knob::HatchEveryHive => &mut r.hatch_every_hive,
        Knob::BeetleLifetime => &mut r.beetle_lifetime,
        Knob::Unit(k, f) => unit_field(&mut r.units[k], f),
        Knob::Form(k, i, f) => form_field(&mut r.units[k].forms[i], f),
        Knob::Arcana(i) => &mut r.arcana_bp[i],
    }
}

pub fn get(r: &Ruleset, k: Knob) -> i64 {
    let mut copy = r.clone();
    slot(&mut copy, k).get()
}

pub fn set(r: &mut Ruleset, k: Knob, v: i64) {
    slot(r, k).set(v);
}

/// Step size and how many steps away from canon the search may go.
pub fn step(k: Knob, canon: i64) -> (i64, i64) {
    let five_percent = (canon.abs() / 20).max(1);
    match k {
        Knob::EnergyStart
        | Knob::EvolveCost(_)
        | Knob::AugmentCost(_)
        | Knob::AugmentForgo
        // Whole Energy, at most ±25 % (one step for every current cost).
        | Knob::Unit(_, UnitField::Cost) => {
            let e = i64::from(ENERGY_SCALE);
            (e, (canon / 4 / e).max(1))
        }
        Knob::Form(_, _, FormField::RangeAdd | FormField::SplashAdd | FormField::DmgAdd) => {
            (five_percent, 5)
        }
        Knob::Form(..) => (500, 4),
        // One percentage point per rank, at most ±2 (P34 sensitivity).
        Knob::Arcana(_) => (100, 2),
        _ => (five_percent, 5),
    }
}

/// Smallest value a knob may take (a positive value stays positive).
fn floor(k: Knob) -> i64 {
    match k {
        Knob::EnergyStart
        | Knob::EvolveCost(_)
        | Knob::AugmentCost(_)
        | Knob::AugmentForgo
        | Knob::Unit(_, UnitField::Cost) => i64::from(ENERGY_SCALE),
        Knob::Unit(_, UnitField::Interval) | Knob::StrongholdInterval => i64::from(MIN_INTERVAL),
        _ => 1,
    }
}

/// `canon` moved by `steps` steps, clamped to the knob's bounds.
pub fn moved(k: Knob, canon: i64, steps: i64) -> i64 {
    let (size, max) = step(k, canon);
    (canon + size * steps.clamp(-max, max)).max(floor(k))
}

/// Display and `--apply` name: `Rampart.cost`, `Archer.Longbow.dmg_bp`.
pub fn name(r: &Ruleset, k: Knob) -> String {
    let unit = |i: usize| format!("{:?}", r.units[i].kind);
    match k {
        Knob::EvolveCost(i) => format!("evolve_cost.{i}"),
        Knob::AugmentCost(i) => format!("augment_cost.{i}"),
        Knob::InterestBand => "interest.band".into(),
        Knob::InterestPerBand => "interest.per_band".into(),
        Knob::InterestMaxBands => "interest.max_bands".into(),
        Knob::AugmentRoundTicks => "augment.round_ticks".into(),
        Knob::AugmentWindow => "augment.window".into(),
        Knob::AugmentForgo => "augment.forgo_energy".into(),
        Knob::Arcana(i) => format!("arcana.{}", ARCANA_NAMES[i]),
        Knob::Synergy(f, l) => format!("synergy.{:?}.{}", FACTIONS[f], l + 1).to_lowercase(),
        Knob::Unit(i, f) => format!("{}.{}", unit(i), snake(&format!("{f:?}"))),
        Knob::Form(i, j, f) => format!(
            "{}.{}.{}",
            unit(i),
            r.units[i].forms[j].name.replace(' ', ""),
            snake(&format!("{f:?}"))
        ),
        other => snake(&format!("{other:?}")),
    }
}

fn snake(s: &str) -> String {
    let mut out = String::new();
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() && i > 0 {
            out.push('_');
        }
        out.extend(c.to_lowercase());
    }
    out
}

/// Parse `name=value,name=value` against the knobs of `r` and apply it.
pub fn apply(r: &mut Ruleset, spec: &str) -> Result<(), String> {
    let knobs = all(r);
    for part in spec.split(',').map(str::trim).filter(|p| !p.is_empty()) {
        let (n, v) = part
            .split_once('=')
            .ok_or(format!("`{part}`: expected name=value"))?;
        let k = knobs
            .iter()
            .copied()
            .find(|&k| name(r, k).eq_ignore_ascii_case(n.trim()))
            .ok_or(format!("unknown knob `{n}`"))?;
        let v: i64 = v
            .trim()
            .parse()
            .map_err(|_| format!("`{part}`: value is not an integer"))?;
        set(r, k, v);
    }
    valid(r)
}

/// Role invariants (Proposed, GAME-DESIGN §5 note). A candidate breaking
/// one is discarded before any match is played.
pub fn valid(r: &Ruleset) -> Result<(), String> {
    for k in UNIT_KINDS {
        let d = r.def(k);
        if d.range > r.stronghold_range {
            return Err(format!("{:?}: base range above the Stronghold's", k));
        }
        let cap = r.energy_cap.unwrap_or(u32::MAX);
        if d.cost < ENERGY_SCALE || d.cost > cap {
            return Err(format!("{:?}: cost outside 1..=cap Energy", k));
        }
        if d.interval < MIN_INTERVAL {
            return Err(format!("{:?}: interval below the minimum", k));
        }
    }
    if r.synergies.iter().any(|(_, bp)| bp[0] > bp[1]) {
        return Err("a level-1 Synergy bonus above its level-2 bonus".into());
    }
    let cap = r.energy_cap.unwrap_or(u32::MAX);
    if r.evolve_cost.iter().any(|&c| c > cap) || r.energy_start > cap {
        return Err("Evolution cost or start Energy above the cap".into());
    }
    for f in FACTIONS {
        let units: Vec<&UnitDef> = UNIT_KINDS
            .iter()
            .map(|&k| r.def(k))
            .filter(|d| d.faction == f)
            .collect();
        // The faction's best unit by `key` belongs to the Archetype, if any.
        let leads = |a: Archetype, key: &dyn Fn(&UnitDef) -> i64| {
            let top = units.iter().map(|d| key(d)).max().unwrap_or(0);
            !units.iter().any(|d| d.archetype == a)
                || units.iter().any(|d| d.archetype == a && key(d) == top)
        };
        if !leads(Archetype::Tank, &|d| {
            i64::from(d.hp) * 1_000 / i64::from(d.cost.max(1))
        }) {
            return Err(format!(
                "{f:?}: a Tank no longer has the most HP per Energy"
            ));
        }
        if !leads(Archetype::Ranged, &|d| i64::from(d.range)) {
            return Err(format!(
                "{f:?}: a Ranged unit no longer has the longest range"
            ));
        }
    }
    Ok(())
}
