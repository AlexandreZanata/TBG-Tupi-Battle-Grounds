//! Safe ground for a larger summoned body or a released passenger at its parent's position.
use crate::{geom::Pos, map, state::MobaMatch};

pub(crate) fn restore(m: &mut MobaMatch, id: u32) {
    let Some(i) = m.slot(id) else { return };
    let u = &m.units[i];
    if map::surface::walkable(u.pos, u.stats.radius) {
        return;
    }
    let origin = u.pos;
    // The map's widest wet strip plus its largest body fits within 20 units.
    // Search outward in stable half-unit rings, preserving the original bank.
    let bank = (origin.x + origin.y - map::SIZE * 1000).signum();
    for radius in (500..=20_000).step_by(500) {
        let at = [
            (-1, 0),
            (0, -1),
            (1, 0),
            (0, 1),
            (-1, -1),
            (1, -1),
            (-1, 1),
            (1, 1),
        ]
        .into_iter()
        .map(|(x, y)| Pos {
            x: origin.x + x * radius,
            y: origin.y + y * radius,
        })
        .filter(|p| bank == 0 || (p.x + p.y - map::SIZE * 1000).signum() == bank)
        .filter(|p| map::surface::walkable(*p, u.stats.radius))
        .min_by_key(|p| (origin.dist2(*p), p.x, p.y));
        if let Some(at) = at {
            m.units[i].pos = at;
            return;
        }
    }
}
