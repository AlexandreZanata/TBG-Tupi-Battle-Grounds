//! Team territory/army visibility and last-observed structure health (M-50–M-55, M-67–M-69).

use crate::geom::Pos;
use crate::grid::Grid;
use crate::state::MobaMatch;
use crate::types::{Team, UnitKind};

mod territory;
pub use territory::{home_half, invading_scout, SightCircle, Territory};

pub struct Vision<'a> {
    game: &'a MobaMatch,
    team: Team,
    territory: Territory,
    pub visible: Vec<bool>,
}
impl<'a> Vision<'a> {
    pub fn new(game: &'a MobaMatch, team: Team) -> Self {
        // Post-movement positions and post-death slots differ from the combat grid.
        let mut grid = Grid::default();
        grid.rebuild(game.units.iter().map(|u| u.pos));
        let standing: Vec<_> = game
            .units
            .iter()
            .filter(|u| {
                u.alive() && u.team() == Some(team.enemy()) && matches!(u.kind, UnitKind::Tower(..))
            })
            .map(|u| u.pos)
            .collect();
        let territory = Territory::new(team, &standing);
        let mut visible = game
            .units
            .iter()
            .map(|u| u.team() == Some(team) || territory.contains(u.pos))
            .collect::<Vec<_>>();
        for ally in game
            .units
            .iter()
            .filter(|u| u.alive() && invading_scout(team, u.side, u.kind, u.pos))
        {
            grid.near(ally.pos, ally.stats.sight, |i| {
                if ally.pos.within(game.units[i].pos, ally.stats.sight) {
                    visible[i] = true;
                }
            });
        }
        Self {
            game,
            team,
            territory,
            visible,
        }
    }
    pub fn point(&self, point: Pos) -> bool {
        self.territory.contains(point)
            || self.game.units.iter().any(|u| {
                u.alive()
                    && invading_scout(self.team, u.side, u.kind, u.pos)
                    && u.pos.within(point, u.stats.sight)
            })
    }
    pub fn unit(&self, id: u32) -> bool {
        self.game
            .slot(id)
            .is_some_and(|i| self.visible[i] || self.game.units[i].kind.structure())
    }
}

/// Structures remain map knowledge; unseen health never updates through the wire.
pub(crate) fn remember_structures(game: &mut MobaMatch) {
    for team in Team::ALL {
        let vision = Vision::new(game, team);
        let known: Vec<_> = game
            .units
            .iter()
            .enumerate()
            .filter(|(_, u)| u.kind.structure())
            .map(|(i, u)| {
                let hp = if vision.visible[i] {
                    u.hp
                } else {
                    game.structure_hp[team.index()]
                        .iter()
                        .find(|(id, _)| *id == u.id)
                        .map_or(u.max_hp, |(_, hp)| *hp)
                };
                (u.id, hp)
            })
            .collect();
        game.structure_hp[team.index()] = known;
    }
}
