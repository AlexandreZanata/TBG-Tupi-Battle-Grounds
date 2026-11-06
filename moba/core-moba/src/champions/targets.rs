//! Stable 2D target sets: distance then id, with lane/jungle neutrality and allied ownership preserved.
use crate::{
    geom::Pos,
    state::{Brain, MobaMatch, Unit},
    types::Side,
};
pub fn enemy(source: &Unit, target: &Unit) -> bool {
    target.alive()
        && target.fx.carrier == 0
        && !target.kind.structure()
        && source.side.hostile(target.side)
        && !(target.side == Side::Neutral
            && matches!(source.brain, Brain::Lane { .. } | Brain::Static))
}
pub fn ally(source: &Unit, target: &Unit) -> bool {
    target.alive() && !target.kind.structure() && source.side == target.side
}
pub fn nearby(
    game: &MobaMatch,
    source: &Unit,
    at: Pos,
    radius: i32,
    cap: u8,
    exclude: Option<u32>,
    allied: bool,
) -> Vec<u32> {
    let mut ids: Vec<_> = game
        .units
        .iter()
        .filter(|u| {
            Some(u.id) != exclude
                && if allied {
                    ally(source, u)
                } else {
                    enemy(source, u)
                }
        })
        .filter(|u| u.pos.within(at, radius + u.stats.radius))
        .map(|u| (u.pos.dist2(at), u.id))
        .collect();
    ids.sort_unstable();
    ids.into_iter()
        .take(usize::from(cap))
        .map(|(_, id)| id)
        .collect()
}
