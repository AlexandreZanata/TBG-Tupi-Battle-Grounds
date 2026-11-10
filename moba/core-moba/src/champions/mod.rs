//! Automatic 2D champion abilities driven only by authoritative ticks and the read-only 1v1 catalog.
pub mod barrels;
mod basic;
pub(crate) mod control;
mod effects;
mod periodic;
pub mod state;
mod summons;
mod support;
mod targets;
use crate::{
    state::{Credit, Event, MobaMatch, Unit},
    types::UnitKind,
};
pub use effects::{heal, hurt, root, slow};
pub(crate) use targets::{ally, enemy, nearby};
use tbg_core::content::{Ability, BP};

pub fn abilities(unit: &Unit) -> [Ability; 2] {
    match unit.kind {
        UnitKind::Troop(k) if unit.fx.summon == 0 => tbg_core::content::def(k).ability(0, 0),
        _ => [Ability::None; 2],
    }
}
pub fn scaled(amount: i32, bp: i32) -> i32 {
    (i64::from(amount) * i64::from(bp) / BP).clamp(0, i64::from(i32::MAX)) as i32
}
pub fn credit(unit: &Unit) -> Credit {
    Credit {
        side: unit.side,
        seat: unit.owner,
    }
}
pub fn initialise(unit: &mut Unit) {
    for a in abilities(unit) {
        match a {
            Ability::Hatch => unit.fx.summon_cd = tbg_core::content::HATCH_EVERY,
            Ability::Cargo { every, .. } => unit.fx.summon_cd = every,
            _ => {}
        }
    }

    for ability in abilities(unit) {
        if let Ability::Plates { count, hp, .. } = ability {
            unit.fx.plates = count;
            unit.fx.plate_hp = i32::from(hp);
        }
    }
}
pub fn tick(game: &mut MobaMatch) {
    barrels::advance(game);
    effects::tick(game);
    support::channels(game);
    support::auras(game);
    summons::tick(game);
    periodic::tick(game);
}
pub use basic::flame;
pub fn before_basic(game: &mut MobaMatch, i: usize, target: usize, damage: i32) -> i32 {
    basic::bonus(game, i, target, damage)
}
pub fn after_basic(game: &mut MobaMatch, source: u32, target: u32) {
    basic::completed(game, source, target);
}
pub fn feed(game: &mut MobaMatch, source: u32, victim: &Unit, removed: i32) {
    basic::feed(game, source, victim, removed);
}
pub fn healing(game: &mut MobaMatch, u: &Unit, target: u32) {
    support::healing(game, u, target);
}
pub fn died(game: &mut MobaMatch, unit: &Unit) {
    summons::died(game, unit);
    basic::death(game, unit);
}
pub fn can_attack_structure(unit: &Unit) -> bool {
    !abilities(unit).contains(&Ability::NoStronghold)
}
pub fn frozen(unit: &Unit, now: u32) -> bool {
    unit.fx.root_until > now || unit.fx.interrupt_until > now
}
/// PK-01 body contact prevents melee deadlocks after scaled collision reservations.
pub fn reach(a: &Unit, b: &Unit) -> i32 {
    a.stats
        .range
        .max(a.stats.radius + b.stats.radius + tbg_core::content::occupancy::GAP)
}
pub fn speed(unit: &Unit, now: u32) -> i32 {
    if unit.fx.carrier != 0
        || unit.fx.root_until > now
        || (unit.fx.reduction_until > now
            && abilities(unit)
                .into_iter()
                .any(|a| matches!(a, Ability::LivingWall { .. })))
    {
        return 0;
    }
    let fish: u16 = unit
        .fx
        .fish
        .iter()
        .map(|(bp, _)| *bp)
        .fold(0, u16::saturating_add);
    let slow = if unit.fx.ignore_slow {
        0
    } else if unit.fx.slow_until > now {
        unit.fx.slow.max(fish)
    } else {
        fish
    };
    scaled(
        unit.stats.speed,
        10_000 + i32::from(unit.fx.move_bp) - i32::from(slow.min(10_000)),
    )
}
pub fn interval(unit: &Unit, now: u32) -> u16 {
    let mut bonus = 0i32;
    for a in abilities(unit) {
        match a {
            Ability::Plates { count, atk_bp, .. } => {
                bonus += i32::from(count.saturating_sub(unit.fx.plates)) * i32::from(atk_bp)
            }
            Ability::Essence { atk_bp, .. } => {
                bonus += i32::from(unit.fx.essence) * i32::from(atk_bp)
            }
            _ => {}
        }
    }
    let interval = (u32::from(unit.stats.attack_ticks) * 10_000
        / (10_000 + bonus.clamp(0, 10_000) as u32))
        .max(u32::from(tbg_core::content::MIN_INTERVAL));
    let bp = if unit.fx.slow_until > now {
        u32::from(unit.fx.slow)
    } else {
        0
    };
    (interval * 10_000 / (10_000 - bp.min(9_999))).min(u32::from(u16::MAX)) as u16
}
pub fn cue(
    game: &mut MobaMatch,
    unit: &Unit,
    target: u32,
    at: crate::geom::Pos,
    radius: i32,
    started: bool,
) {
    if let UnitKind::Troop(kind) = unit.kind {
        game.events.push(Event::ChampionAbility {
            unit: unit.id,
            kind,
            side: unit.side,
            target,
            at,
            radius,
            started,
        });
    }
}
pub fn view(unit: &Unit, now: u32) -> crate::protocol::ChampionView {
    let f = &unit.fx;
    let left = |at: u32| at.saturating_sub(now).min(u32::from(u16::MAX)) as u16;
    let cast = f.casts.iter().flatten().next();
    let completion = tbg_core::completion::state::View {
        echo_ticks: left(f.echo_until),
        regrowth_pulses: f
            .pulses
            .iter()
            .filter(|p| p.kind == state::PulseKind::Regrowth)
            .map(|p| p.left)
            .max()
            .unwrap_or(0),
        spark_ticks: left(f.spark_until),
        interrupt_ticks: left(f.interrupt_until),
        charged: f.charged,
        channel_target: f.channel.map(|c| c.target),
        channel_pulses: f.channel.map_or(0, |c| c.left),
        beam_center: None,
        barrel_ready_in: unit.cooldown,
    };
    crate::protocol::ChampionView {
        counter: basic::counter(unit),
        reduction: if f.reduction_until > now {
            f.reduction
        } else {
            0
        },
        shield: f.shield.max(0).min(i32::from(u16::MAX)) as u16,
        plates: f.plates,
        rooted: f.root_until > now,
        slowed: f.slow_until > now,
        poison: f
            .pulses
            .iter()
            .filter(|p| p.kind == state::PulseKind::Poison)
            .count() as u8,
        carrier: f.carrier,
        bond: f.bond,
        cast_left: cast.map_or(0, |c| c.left),
        cast_windup: cast.map_or(0, |c| c.windup),
        ready_in: f.ready.into_iter().min().unwrap_or(0),
        completion: Some(completion),
    }
}

pub fn same_target_counter(u: &Unit) -> bool {
    abilities(u).into_iter().any(|a| {
        matches!(
            a,
            Ability::PreciseStrike { .. }
                | Ability::VerdictLance { .. }
                | Ability::PredatoryFrenzy { .. }
        )
    })
}
pub fn cannot_basic(u: &Unit, now: u32) -> bool {
    u.fx.root_until > now || u.fx.channel.is_some()
}
pub fn walked(u: &mut Unit) {
    for a in abilities(u) {
        if let Ability::Completion(tbg_core::content::completion::Ability::Pounce {
            walk_ticks,
            ..
        }) = a
        {
            u.fx.walked = u.fx.walked.saturating_add(1);
            u.fx.charged = u.fx.walked >= walk_ticks;
        }
    }
}
