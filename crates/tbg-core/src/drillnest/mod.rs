//! R21's bounded production, finite snapshot profiles and non-recursive deterministic drill fusion.
mod effects;
use crate::{
    content::{Ability, WINDUP},
    state::{Event, MatchState, PendingCast, Target, Unit, UnitFx},
    targeting,
    types::{Seat, Status, Tick, UnitKind},
    Ruleset,
};
const CAP: usize = 8;
fn rank(k: UnitKind) -> u8 {
    match k {
        UnitKind::Microdrill => 1,
        UnitKind::Duodrill => 2,
        UnitKind::Megadrill => 3,
        _ => 0,
    }
}
fn next(k: UnitKind) -> Option<UnitKind> {
    match k {
        UnitKind::Microdrill => Some(UnitKind::Duodrill),
        UnitKind::Duodrill => Some(UnitKind::Megadrill),
        _ => None,
    }
}
fn lifetime(k: UnitKind) -> Tick {
    match k {
        UnitKind::Microdrill => 240,
        UnitKind::Duodrill => 280,
        UnitKind::Megadrill => 320,
        _ => 0,
    }
}
pub(crate) fn period(r: &Ruleset, k: UnitKind, t: u8, b: u8) -> Option<u16> {
    r.def(k).ability(t, b).into_iter().find_map(|a| match a {
        Ability::DrillNest { period, .. } => Some(period),
        _ => None,
    })
}
fn living(s: &MatchState, u: &Unit) -> bool {
    u.hp > 0 && rank(u.kind) > 0 && u.expires_at.is_some_and(|t| s.tick < t)
}
fn count(s: &MatchState, seat: Seat) -> usize {
    s.units
        .iter()
        .filter(|u| u.seat == seat && living(s, u))
        .count()
}
fn spawn(s: &mut MatchState, source: &Unit, kind: UnitKind, ev: &mut Vec<Event>) -> Unit {
    let st = s
        .rules()
        .effective_stats(kind, source.tier, source.branch, 0);
    let id = s.next_unit_id;
    s.next_unit_id = s.next_unit_id.saturating_add(1);
    let u = Unit {
        id,
        seat: source.seat,
        kind,
        tier: source.tier,
        branch: source.branch,
        x: source.x,
        hp: st.max_hp,
        max_hp: st.max_hp,
        cooldown: 0,
        windup: None,
        status: Status::None,
        summoner: Some(source.summoner.unwrap_or(source.id)),
        expires_at: Some(s.tick.saturating_add(lifetime(kind))),
        action_cd: 0,
        haste_until: 0,
        heal_cut_until: 0,
        heal_cut_bp: 0,
        second_wind_used: 0,
        fx: UnitFx::default(),
    };
    ev.push(Event::Spawn {
        unit: id,
        seat: u.seat,
        kind,
    });
    u
}
/// Called after movement/hits, before production. Outputs cannot enter this pass.
pub(crate) fn fuse(s: &mut MatchState, ev: &mut Vec<Event>) {
    let candidates: Vec<_> = s
        .units
        .iter()
        .filter(|u| living(s, u) && rank(u.kind) < 3)
        .map(|u| u.id)
        .collect();
    let mut used = Vec::new();
    let mut outputs = Vec::new();
    for (n, &id) in candidates.iter().enumerate() {
        if used.contains(&id) {
            continue;
        }
        let Some(i) = s.unit_index(id) else {
            continue;
        };
        let a = s.units[i];
        let partner = candidates[n + 1..]
            .iter()
            .filter(|id| !used.contains(id))
            .filter_map(|id| s.unit_index(*id))
            .find(|&j| {
                let b = s.units[j];
                a.seat == b.seat && a.kind == b.kind && (a.x - b.x).abs() <= 200
            });
        let Some(j) = partner else {
            continue;
        };
        let b = s.units[j];
        let Some(kind) = next(a.kind) else {
            continue;
        };
        used.extend([a.id, b.id]);
        let mut out = spawn(s, &a, kind, ev);
        out.x = ((i64::from(a.x) + i64::from(b.x)).div_euclid(2)) as i32;
        out.hp = ((i64::from(a.hp) + i64::from(b.hp)) * i64::from(out.max_hp)
            / (i64::from(a.max_hp) + i64::from(b.max_hp)))
        .clamp(1, i64::from(out.max_hp)) as i32;
        out.cooldown = a.cooldown.max(b.cooldown).min(
            s.rules()
                .effective_stats(kind, out.tier, out.branch, 0)
                .interval,
        );
        out.status = effects::status(a.status, b.status);
        out.fx = effects::inherit(&a, &b, s.tick);
        (out.heal_cut_bp, out.heal_cut_until) = effects::pair_cut(&a, &b, s.tick);
        if out.tier == 2 && out.branch == 0 {
            crate::roster_slots::apply_shield(&mut out.fx, 25, 40, s.tick);
        }
        ev.push(Event::DrillFused {
            first: a.id,
            second: b.id,
            unit: out.id,
            x: out.x,
        });
        preserve_marks(s, a.id, b.id, out.id);
        outputs.push(out);
    }
    s.units.retain(|u| !used.contains(&u.id));
    s.units.extend(outputs);
}
/// Hostile marks follow the replacement; caster focus sequences still see a new target id.
fn preserve_marks(s: &mut MatchState, first: u32, second: u32, out: u32) {
    for u in &mut s.units {
        if u.kind == UnitKind::Crowcaller
            && u.fx
                .mark_target
                .is_some_and(|id| id == first || id == second)
        {
            u.fx.mark_target = Some(out);
        }
    }
}
/// Producers engage with no basic attack. Full cap keeps one readiness, never a burst.
pub(crate) fn produce(s: &mut MatchState, ev: &mut Vec<Event>) {
    let n = s.units.len();
    for i in 0..n {
        advance(s, i, ev);
    }
}
fn advance(s: &mut MatchState, i: usize, ev: &mut Vec<Event>) {
    let u = s.units[i];
    let Some(every) = period(s.rules(), u.kind, u.tier, u.branch) else {
        return;
    };
    if u.hp <= 0 || u.is_summon() {
        return;
    }
    if matches!(u.status, Status::Rooted { .. }) {
        s.units[i].fx.special = None;
        return;
    }
    if let Some(cast) = u.fx.special {
        advance_cast(s, i, &u, cast, ev);
    } else {
        start(s, i, &u, every, ev);
    }
}
fn advance_cast(
    s: &mut MatchState,
    i: usize,
    u: &Unit,
    mut cast: PendingCast,
    ev: &mut Vec<Event>,
) {
    let alive = cast
        .target
        .map_or(s.player(u.seat.other()).stronghold_hp > 0, |id| {
            s.unit_index(id).is_some_and(|j| s.units[j].hp > 0)
        });
    if !alive {
        s.units[i].fx.special = None;
        return;
    }
    if cast.left > 1 {
        cast.left -= 1;
        s.units[i].fx.special = Some(cast);
        return;
    }
    s.units[i].fx.special = None;
    if count(s, u.seat) >= CAP {
        s.units[i].action_cd = 0;
        return;
    }
    let drill = spawn(s, u, UnitKind::Microdrill, ev);
    s.units.push(drill);
    ev.push(Event::AbilityResolved {
        unit: u.id,
        kind: u.kind,
        primary: cast.target,
        center: u.x,
        tick: s.tick,
    });
}
fn start(s: &mut MatchState, i: usize, u: &Unit, every: u16, ev: &mut Vec<Event>) {
    if u.action_cd > 0 || count(s, u.seat) >= CAP {
        return;
    }
    let Some(target) = targeting::enemy_in_range(s, u, s.stats_of(u).range) else {
        return;
    };
    let primary = match target {
        Target::Unit(id) => Some(id),
        Target::Stronghold(_) => None,
    };
    s.units[i].fx.special = Some(PendingCast {
        target: primary,
        left: WINDUP,
    });
    s.units[i].action_cd = every;
    ev.push(Event::AbilityStarted {
        unit: u.id,
        kind: u.kind,
        primary,
        center: u.x,
        tick: s.tick,
    });
}
/// B2's printed-HP condition applies to rank three and never to Strongholds.
pub(crate) fn bonus(s: &MatchState, u: &Unit, target: Target, amount: i32) -> i32 {
    if u.kind != UnitKind::Megadrill || u.tier != 2 || u.branch != 1 {
        return amount;
    }
    let Target::Unit(id) = target else {
        return amount;
    };
    if s.unit_index(id)
        .is_some_and(|i| s.rules().def(s.units[i].kind).hp >= 1000)
    {
        crate::hits::scale(amount, 2500)
    } else {
        amount
    }
}

#[cfg(all(test, feature = "lab"))]
mod tests;
