//! GD-04 Volthar's Storm Judgment: a counted fourth hit chains to distinct nearby enemies.
use crate::{
    content::{Ability, Stats},
    hits,
    state::{Event, MatchState, Target, Unit},
    targeting,
    types::{OnHit, UnitId, UnitKind},
};

/// One primary, at most four jumps and two B2 extras bound a single discharge.
const MAX_STRUCK: usize = 7;

#[derive(Clone, Copy)]
struct Storm {
    every: u8,
    jumps: u8,
    radius: i32,
    ratios: [u16; 4],
    primary_bp: u16,
    root: u16,
    extra_ratio: u16,
    extra_targets: u8,
}

fn parameters(s: &MatchState, u: &Unit) -> Option<Storm> {
    if u.kind != UnitKind::Volthar || u.is_summon() {
        return None;
    }
    s.rules()
        .def(u.kind)
        .ability(u.tier, u.branch)
        .into_iter()
        .find_map(|a| match a {
            Ability::StormJudgment {
                every,
                jumps,
                radius,
                ratios,
                primary_bp,
                root,
                extra_ratio,
                extra_targets,
            } => Some(Storm {
                every: every.max(1),
                jumps: jumps.min(4),
                radius,
                ratios,
                primary_bp,
                root,
                extra_ratio,
                extra_targets: extra_targets.min(2),
            }),
            _ => None,
        })
}

pub(crate) fn counter(u: &Unit) -> u8 {
    if u.kind == UnitKind::Volthar && !u.is_summon() {
        u.fx.attacks.min(3)
    } else {
        0
    }
}

fn hostile_unit(s: &MatchState, u: &Unit, target: Target) -> Option<usize> {
    let Target::Unit(id) = target else {
        return None;
    };
    s.unit_index(id).filter(|&t| s.units[t].seat != u.seat)
}

/// B1/B2: the triggering primary gains a share of static basic damage before
/// situational percentages (BR-X01), like other counted basic bonuses.
pub(crate) fn bonus(s: &MatchState, u: &Unit, target: Target, st: &Stats) -> i32 {
    let Some(storm) = parameters(s, u) else {
        return 0;
    };
    let ready = counter(u).saturating_add(1) >= storm.every;
    match hostile_unit(s, u, target) {
        Some(t) if ready && storm.primary_bp > 0 && s.units[t].hp > 0 => {
            (i64::from(st.damage) * i64::from(storm.primary_bp) / 10_000)
                .clamp(0, i64::from(i32::MAX)) as i32
        }
        _ => 0,
    }
}

/// The nearest living, targetable enemy unit to `center` within `radius` that
/// was not struck yet; ties break by lower id. Strongholds never take part.
fn next_link(
    s: &MatchState,
    a: &Unit,
    struck: &[UnitId],
    center: i32,
    radius: i32,
) -> Option<UnitId> {
    s.units
        .iter()
        .filter(|u| u.seat != a.seat && u.hp > 0 && targeting::targetable(u))
        .filter(|u| !struck.contains(&u.id))
        .map(|u| ((u.x - center).abs(), u.id))
        .filter(|&(d, _)| d <= radius)
        .min()
        .map(|(_, id)| id)
}

fn share(st: &Stats, ratio: u16) -> i32 {
    (i64::from(st.damage) * i64::from(ratio) / 10_000).clamp(0, i64::from(i32::MAX)) as i32
}

pub(crate) fn completed(
    s: &mut MatchState,
    a: &Unit,
    target: Target,
    st: &Stats,
    ev: &mut Vec<Event>,
) {
    let Some(storm) = parameters(s, a) else {
        return;
    };
    let Some(pi) = hostile_unit(s, a, target) else {
        return;
    };
    let Some(i) = s.unit_index(a.id) else {
        return;
    };
    let n = s.units[i].fx.attacks.saturating_add(1);
    if n < storm.every {
        s.units[i].fx.attacks = n;
        return;
    }
    s.units[i].fx.attacks = 0;
    let primary = s.units[pi].id;
    let origin = s.units[pi].x;
    if storm.root > 0 {
        hits::apply_on_hit(s, primary, OnHit::Root { ticks: storm.root }, a.seat, ev);
    }
    let mut struck = [UnitId::MAX; MAX_STRUCK];
    struck[0] = primary;
    let mut count = 1;
    let (mut link, mut center) = (primary, origin);
    for jump in 0..usize::from(storm.jumps) {
        let Some(to) = next_link(s, a, &struck[..count], center, storm.radius) else {
            break;
        };
        // Damage reads the jump's own share; a link that dies still anchors the next.
        let Some(t) = s.unit_index(to) else { break };
        let at = s.units[t].x;
        hits::auxiliary_hit(s, a, to, share(st, storm.ratios[jump]), ev);
        ev.push(Event::StormArc {
            unit: a.id,
            from: link,
            to,
            jump: jump as u8 + 1,
        });
        struck[count] = to;
        count += 1;
        (link, center) = (to, at);
    }
    // B2: a separate discharge from the primary to enemies the chain missed.
    for _ in 0..storm.extra_targets {
        let Some(to) = next_link(s, a, &struck[..count], origin, storm.radius) else {
            break;
        };
        hits::auxiliary_hit(s, a, to, share(st, storm.extra_ratio), ev);
        ev.push(Event::StormArc {
            unit: a.id,
            from: primary,
            to,
            jump: 0,
        });
        struck[count] = to;
        count += 1;
    }
    ev.push(Event::AbilityResolved {
        unit: a.id,
        kind: a.kind,
        primary: Some(primary),
        center: origin,
        tick: s.tick,
    });
}

#[cfg(all(test, feature = "lab"))]
mod tests;
