//! Shared-value damage, shields, healing, control and bounded periodic effects with seat credit.
use super::{
    abilities, credit, scaled,
    state::{Pulse, PulseKind},
};
use crate::state::{Credit, Event, MobaMatch, Unit};
use tbg_core::content::Ability;

pub fn hurt(game: &mut MobaMatch, source: &Unit, target: u32, amount: i32) -> i32 {
    if let Some(i) = game.slot(target) {
        if matches!(game.units[i].brain, crate::state::Brain::Neutral { .. })
            && game.units[i].target == 0
        {
            game.units[i].target = source.id;
        }
    }
    let carrier = game.unit(target).map_or(0, |u| u.fx.carrier);
    let target = if carrier != 0 { carrier } else { target };
    let removed = damage(game, credit(source), target, amount);
    if removed > 0 {
        if let Some(at) = game.unit(target).map(|u| u.pos) {
            game.events.push(Event::UnitHit {
                unit: target,
                from: source.pos,
                at,
            });
        }
    }
    removed
}
pub fn damage(game: &mut MobaMatch, credit: Credit, target: u32, amount: i32) -> i32 {
    let Some(i) = game.slot(target) else { return 0 };
    if !game.units[i].alive() || amount <= 0 {
        return 0;
    }
    let now = game.tick;
    let u = &mut game.units[i];
    let reduction = if u.fx.reduction_until > now {
        u.fx.reduction
    } else {
        0
    };
    let bp = (i32::from(reduction) + i32::from(u.fx.aura_dr)).min(2_000);
    let mut damage = scaled(scaled(amount, 10_000 - bp), 10_000 - u.stats.armor * 100).max(1);
    let blocked = damage.min(u.fx.shield.max(0));
    u.fx.shield -= blocked;
    damage -= blocked;
    // Plates absorb the remaining receipt before HP, as in the shared 1v1 hit path.
    for ability in abilities(u) {
        if let Ability::Plates { hp, .. } = ability {
            while damage > 0 && u.fx.plates > 0 {
                let take = damage.min(u.fx.plate_hp);
                damage -= take;
                u.fx.plate_hp -= take;
                if u.fx.plate_hp <= 0 {
                    u.fx.plates -= 1;
                    u.fx.plate_hp = if u.fx.plates > 0 { i32::from(hp) } else { 0 };
                }
            }
        }
    }
    let actual = damage.min(u.hp.max(0));
    u.hp -= damage;
    u.last_hit = Some(credit);

    let echo = (u.fx.echo_until > now && actual > 0).then_some((u.fx.echo, u.fx.echo_credit));
    if actual > 0 {
        u.fx.echo_until = 0;
        u.fx.pulses.retain(|p| p.kind != PulseKind::Regrowth);
    }
    received(u, now, actual);
    if let Some((amount, Some(c))) = echo {
        heal_credit(game, c, target, amount);
    }
    actual
}
fn received(unit: &mut Unit, now: u32, damage: i32) {
    for ability in abilities(unit) {
        match ability {
            Ability::PrimordialCarapace {
                step_bp,
                cap_bp,
                gap,
                ..
            } if damage > 0 => {
                if unit.fx.reduction_until <= now {
                    unit.fx.reduction = 0;
                }
                unit.fx.reduction = unit.fx.reduction.saturating_add(step_bp).min(cap_bp);
                unit.fx.reduction_until = now + u32::from(gap);
            }
            _ => {}
        }
    }
}
pub fn heal(game: &mut MobaMatch, source: &Unit, target: u32, amount: i32) -> i32 {
    heal_credit(game, credit(source), target, amount)
}
pub fn heal_credit(game: &mut MobaMatch, _credit: Credit, target: u32, amount: i32) -> i32 {
    let Some(i) = game.slot(target) else { return 0 };
    let now = game.tick;
    let u = &mut game.units[i];
    if !u.alive() {
        return 0;
    }
    let cut = if u.fx.cut_until > now { u.fx.cut } else { 0 };
    let actual = scaled(amount, 10_000 - i32::from(cut))
        .min(u.max_hp - u.hp)
        .max(0);
    u.hp += actual;
    actual
}
pub fn slow(game: &mut MobaMatch, target: u32, bp: u16, ticks: u16) {
    let now = game.tick;
    if let Some(i) = game.slot(target) {
        let u = &mut game.units[i];
        if bp == 0 || !u.alive() || u.fx.ward_until > now {
            return;
        }
        u.fx.slow = u.fx.slow.max(bp);
        u.fx.slow_until = u.fx.slow_until.max(now + u32::from(ticks));
    }
}
pub fn root(game: &mut MobaMatch, target: u32, ticks: u16) {
    let now = game.tick;
    if let Some(i) = game.slot(target) {
        let u = &mut game.units[i];
        if u.alive() && u.fx.ward_until <= now {
            u.fx.root_until = u.fx.root_until.max(now + u32::from(ticks));
            u.fx.casts = [None; 2];
            u.fx.basic = None;
            u.fx.betrayed = false;
        }
    }
}
pub fn shield(game: &mut MobaMatch, target: u32, amount: i32, ticks: u16) {
    if let Some(i) = game.slot(target) {
        let until = game.tick + u32::from(ticks);
        let u = &mut game.units[i];
        u.fx.shield = u.fx.shield.max(amount);
        u.fx.shield_until = u.fx.shield_until.max(until);
    }
}
pub fn reduction(game: &mut MobaMatch, target: u32, bp: u16, ticks: u16) {
    if let Some(i) = game.slot(target) {
        let until = game.tick + u32::from(ticks);
        let u = &mut game.units[i];
        u.fx.reduction = u.fx.reduction.max(bp);
        u.fx.reduction_until = u.fx.reduction_until.max(until);
    }
}
pub fn pulse(
    game: &mut MobaMatch,
    source: &Unit,
    target: u32,
    kind: PulseKind,
    amount: i32,
    left: u8,
    gap: u16,
    cap: usize,
) {
    let Some(i) = game.slot(target) else { return };
    let u = &mut game.units[i];
    if !u.alive()
        || left == 0
        || u.fx.ward_until > game.tick
            && matches!(
                kind,
                PulseKind::Poison | PulseKind::Bleed | PulseKind::Venom
            )
    {
        return;
    }
    let new = Pulse {
        kind,
        amount,
        left,
        next: game.tick + u32::from(gap),
        gap,
        credit: credit(source),
    };
    let count = u.fx.pulses.iter().filter(|p| p.kind == kind).count();
    if count < cap.max(1) {
        u.fx.pulses.push(new);
    } else if let Some(p) =
        u.fx.pulses
            .iter_mut()
            .filter(|p| p.kind == kind)
            .min_by_key(|p| p.left)
    {
        if new.amount >= p.amount || kind == PulseKind::Poison {
            *p = Pulse {
                next: p.next,
                ..new
            };
        }
    }
}
pub fn tick(game: &mut MobaMatch) {
    let mut due = Vec::new();
    for unit in &mut game.units {
        if unit.fx.shield_until <= game.tick {
            unit.fx.shield = 0;
        }
        if unit.fx.slow_until <= game.tick {
            unit.fx.slow = 0;
        }
        unit.fx.fish.retain(|(_, until)| *until > game.tick);
        for ready in &mut unit.fx.ready {
            *ready = ready.saturating_sub(1);
        }
        for pulse in &mut unit.fx.pulses {
            if pulse.left > 0 && pulse.next <= game.tick {
                due.push((unit.id, *pulse));
                pulse.left -= 1;
                pulse.next += u32::from(pulse.gap);
            }
        }
        unit.fx.pulses.retain(|p| p.left > 0);
    }
    for (target, pulse) in due {
        if matches!(pulse.kind, PulseKind::Regrowth) {
            heal_credit(game, pulse.credit, target, pulse.amount);
        } else {
            damage(game, pulse.credit, target, pulse.amount);
        }
    }
}
