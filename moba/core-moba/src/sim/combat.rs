//! Executes locked basics and resolves their completed batch with 1v1 windups.
use super::{
    decision::{self, Order, Phase},
    targeting::Vulnerability,
};
use crate::{
    champions::{self, state::Cast},
    content,
    state::MobaMatch,
    types::UnitKind,
};

pub(super) fn fight(m: &mut MobaMatch) -> Vec<bool> {
    let vul = Vulnerability::of(m);
    let mut hits = Vec::new();
    let mut engaged = Vec::with_capacity(m.units.len());
    for i in 0..m.units.len() {
        m.units[i].cooldown = m.units[i].cooldown.saturating_sub(1);
        let order = decision::order(m, &vul, i);
        match order {
            Order::Windup => finish_windup(m, i, &mut hits),
            Order::Strike(choice) => start(m, i, choice.slot, choice.support, &mut hits),
            _ => {}
        }
        engaged.push(!matches!(order, Order::Move));
    }
    for (i, j) in hits {
        land(m, i, j);
    }
    engaged
}

fn finish_windup(m: &mut MobaMatch, i: usize, hits: &mut Vec<(usize, usize)>) {
    let Some(mut cast) = m.units[i].fx.basic else {
        return;
    };
    cast.left = cast.left.saturating_sub(1);
    m.units[i].fx.basic = (cast.left > 0).then_some(cast);
    if cast.left == 0 {
        if let Some(j) = m.slot(cast.target).filter(|&j| m.units[j].alive()) {
            hits.push((i, j));
        }
        decision::phase(&mut m.units[i], Phase::Recover);
    }
}

fn start(m: &mut MobaMatch, i: usize, j: usize, support: bool, hits: &mut Vec<(usize, usize)>) {
    let a = &m.units[i];
    let betrayal = (!support)
        .then(|| champions::control::betrayal(m, a))
        .flatten();
    let id = betrayal.unwrap_or(m.units[j].id);
    let centre = m.unit(id).map_or(m.units[j].pos, |u| u.pos);
    let a = &mut m.units[i];
    a.cooldown = champions::interval(a, m.tick);
    a.last_attack = m.tick;
    a.fx.betrayed = betrayal.is_some();
    if matches!(a.kind, UnitKind::Troop(_)) {
        let windup = tbg_core::content::WINDUP;
        a.fx.basic = Some(Cast {
            slot: 0,
            target: id,
            centre,
            left: windup,
            windup,
        });
        decision::phase(a, Phase::Windup);
    } else {
        hits.push((i, j));
    }
}

fn land(m: &mut MobaMatch, i: usize, j: usize) {
    let a = m.units[i].clone();
    let b = m.units[j].clone();
    if !b.alive() {
        return;
    }
    if champions::control::resolve(m, &a, &b) {
        return;
    }
    if a.fx.betrayed {
        let base = a.stats.damage * (100 + a.power) / 100;
        let dmg = champions::before_basic(m, i, j, base);
        champions::control::betray_hit(m, &a, &b, dmg);
        return;
    }
    if a.side == b.side {
        champions::healing(m, &a, b.id);
        return;
    }
    let base = a.stats.damage * (100 + a.power) / 100;
    let dmg = champions::before_basic(m, i, j, base);
    if champions::flame(m, &a, &b, dmg) {
        return;
    }
    let removed = champions::hurt(m, &a, b.id, dmg);
    if !b.kind.structure() && b.fx.summon == 0 {
        champions::feed(m, a.id, &b, removed);
    }
    champions::after_basic(m, a.id, b.id);
    if a.stats.splash <= 0 {
        return;
    }
    let friendly =
        champions::abilities(&a).contains(&tbg_core::content::Ability::Blast { friendly: true });
    let mut ids = Vec::new();
    m.grid.near(b.pos, a.stats.splash, |k| {
        let v = &m.units[k];
        if k != j
            && v.alive()
            && !v.kind.structure()
            && v.pos.within(b.pos, a.stats.splash)
            && (champions::enemy(&a, v) || friendly && v.side == a.side)
        {
            ids.push(v.id);
        }
    });
    ids.sort_unstable();
    for id in ids {
        champions::hurt(m, &a, id, base * content::SPLASH_PERCENT / 100);
    }
}
