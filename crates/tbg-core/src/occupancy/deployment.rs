//! Nearest free birth position in the defended backfield, without crossing a hostile front.
use super::*;
pub(super) fn admit(u: &mut Unit, placed: &[Unit]) -> bool {
    let dir = u.seat.dir();
    let origin = if dir > 0 {
        content::SPAWN_OFFSET
    } else {
        content::LANE_LEN - content::SPAWN_OFFSET
    };
    if u.x != origin {
        return false;
    }
    let start = origin * dir;
    let mut end = start + content::occupancy::DEPLOY_END - content::SPAWN_OFFSET;
    let body = footprint(u.kind, u.tier).body;
    for enemy in placed
        .iter()
        .filter(|v| v.seat != u.seat && v.hp > 0 && v.fx.carrier.is_none())
    {
        end = end.min(enemy.x * dir - body - footprint(enemy.kind, enemy.tier).body - GAP);
    }
    let mut best = None;
    let mut intervals = Vec::new();
    for slot in 0..=COLUMNS - cells(u) {
        intervals.clear();
        for other in placed
            .iter()
            .filter(|v| v.seat == u.seat && v.hp > 0 && v.fx.carrier.is_none())
        {
            if overlaps(u, slot, other) {
                intervals.push((
                    other.x * dir - spacing(u, other),
                    other.x * dir + spacing(other, u),
                ));
            }
        }
        intervals.sort_unstable();
        let mut progress = start;
        for &(lo, hi) in &intervals {
            if lo < progress && progress < hi {
                progress = hi;
            }
        }
        if progress > end {
            continue;
        }
        let score = (-progress, -center(u, slot).abs(), slot);
        if best.is_none_or(|(old, _)| score > old) {
            best = Some((score, slot));
        }
    }
    let Some(((negative, _, _), slot)) = best else {
        return false;
    };
    u.x = -negative * dir;
    u.fx.lane_slot = (slot + 1) as u8;
    u.fx.lane_goal = u.fx.lane_slot;
    true
}
