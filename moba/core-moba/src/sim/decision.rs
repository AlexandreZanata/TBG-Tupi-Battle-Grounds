//! Decision priorities shared by combat and navigation, without replacing a unit's mission.
use crate::{
    geom::Pos,
    state::{Brain, Unit},
    types::Side,
};

pub fn in_position_at(a: &Unit, b: &Unit, at: Pos) -> bool {
    at.within(b.pos, crate::champions::reach(a, b))
        && !(matches!(a.brain, Brain::Jungle { .. })
            && b.side == Side::Neutral
            && matches!(b.brain, Brain::Neutral { camp } if !at.within(crate::map::camp(usize::from(camp)).0,crate::content::CAMP_LEASH)))
}
pub fn in_position(a: &Unit, b: &Unit) -> bool {
    in_position_at(a, b, a.pos)
}
