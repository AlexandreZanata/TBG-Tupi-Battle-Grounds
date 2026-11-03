//! Shared home-half, invading-army and conquered-tower visibility rules (M-67–M-69).

use crate::content;
use crate::geom::{Pos, MM};
use crate::map;
use crate::types::{Side, Team, UnitKind};

/// The river is x+y=SIZE; its centre line belongs to both home halves (M-67).
pub fn home_half(team: Team, point: Pos) -> bool {
    let sum = i64::from(point.x) + i64::from(point.y);
    let river = i64::from(map::SIZE) * i64::from(MM);
    match team {
        Team::Blue => sum <= river,
        Team::Red => sum >= river,
    }
}

/// Home-side units cannot scout across the river until their centre crosses it (M-68).
pub fn invading_scout(team: Team, side: Side, kind: UnitKind, point: Pos) -> bool {
    side == Side::Team(team)
        && matches!(kind, UnitKind::Troop(_) | UnitKind::Minion(_))
        && !home_half(team, point)
}

#[derive(Clone, Copy, Debug)]
pub struct SightCircle {
    pub centre: Pos,
    pub radius: i32,
}

/// All surviving Tower sites are public map knowledge, including those outside sight.
/// An absent enemy Tower therefore reveals its original sight circle until a new match.
#[derive(Clone, Debug)]
pub struct Territory {
    team: Team,
    pub conquered: Vec<SightCircle>,
}

impl Territory {
    pub fn new(team: Team, standing_enemy_towers: &[Pos]) -> Self {
        let conquered = map::towers()
            .filter(|(owner, _, _, pos)| {
                *owner == team.enemy() && !standing_enemy_towers.contains(pos)
            })
            .map(|(_, tier, lane, centre)| SightCircle {
                centre,
                radius: content::sight(UnitKind::Tower(tier, lane)),
            })
            .collect();
        Self { team, conquered }
    }

    pub fn contains(&self, point: Pos) -> bool {
        home_half(self.team, point)
            || self
                .conquered
                .iter()
                .any(|area| area.centre.within(point, area.radius))
    }
}
