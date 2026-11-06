//! Gust Drummer's finite, swept rolling heal. Server snapshots supply its exact position.
use super::{abilities, ally, cue, effects, enemy};
use crate::{
    geom::Pos,
    state::{MobaMatch, Unit},
};
use tbg_core::{
    barrel::CONTACT_RADIUS,
    content::{completion::Ability as C, Ability},
};

#[derive(Clone, Debug)]
pub struct Barrel {
    pub source: Unit,
    pub at: Pos,
    pub origin: Pos,
    pub end: Pos,
    pub remaining: i32,
    pub speed: i32,
    pub radius: i32,
    pub damage: i32,
}
pub fn launch(g: &mut MobaMatch, u: &Unit, target: &Unit) -> bool {
    let Some((speed, radius, damage)) = abilities(u).into_iter().find_map(|a| match a {
        Ability::Completion(C::Barrel {
            speed,
            radius,
            damage,
        }) => Some((speed, radius, damage)),
        _ => None,
    }) else {
        return false;
    };
    if target.id == u.id || !target.alive() || g.barrels.iter().any(|b| b.source.id == u.id) {
        return true;
    }
    let distance = u.pos.dist(target.pos).max(1);
    let length = super::reach(u, target);
    let end = Pos {
        x: u.pos.x + (i64::from(target.pos.x - u.pos.x) * i64::from(length) / distance) as i32,
        y: u.pos.y + (i64::from(target.pos.y - u.pos.y) * i64::from(length) / distance) as i32,
    }
    .clamp_to_map();
    g.barrels.push(Barrel {
        source: u.clone(),
        at: u.pos,
        origin: u.pos,
        end,
        remaining: length,
        speed,
        radius,
        damage: i32::from(damage),
    });
    true
}
pub fn advance(g: &mut MobaMatch) {
    let mut barrels = std::mem::take(&mut g.barrels);
    barrels.retain_mut(|b| {
        let step = b.speed.min(b.remaining);
        let mut next = b.at;
        let arrived = next.step_toward(b.end, step);
        let dx = i64::from(next.x - b.at.x);
        let dy = i64::from(next.y - b.at.y);
        let length = dx * dx + dy * dy;
        let contact = g
            .units
            .iter()
            .filter(|u| {
                u.id != b.source.id
                    && u.fx.carrier == 0
                    && (ally(&b.source, u) || enemy(&b.source, u))
            })
            .filter_map(|u| {
                let along = (i64::from(u.pos.x - b.at.x) * dx + i64::from(u.pos.y - b.at.y) * dy)
                    .clamp(0, length);
                let at = Pos {
                    x: b.at.x + (dx * along / length.max(1)) as i32,
                    y: b.at.y + (dy * along / length.max(1)) as i32,
                };
                at.within(u.pos, CONTACT_RADIUS + u.stats.radius)
                    .then_some((along, u.id, at))
            })
            .min_by_key(|&(dist, id, _)| (dist, id));
        if let Some((_, id, at)) = contact {
            burst(g, b, id, at);
            return false;
        }
        b.at = next;
        b.remaining -= step;
        !arrived && b.remaining > 0
    });
    g.barrels = barrels;
}
fn burst(g: &mut MobaMatch, b: &Barrel, id: u32, at: Pos) {
    if g.unit(id).is_some_and(|u| enemy(&b.source, u)) {
        effects::hurt(g, &b.source, id, b.damage);
    }
    let allies: Vec<_> = g
        .units
        .iter()
        .filter(|u| ally(&b.source, u) && u.pos.within(at, b.radius + u.stats.radius))
        .map(|u| u.id)
        .collect();
    for id in allies {
        effects::heal(g, &b.source, id, b.source.stats.heal);
    }
    cue(g, &b.source, 0, at, b.radius, false);
}
