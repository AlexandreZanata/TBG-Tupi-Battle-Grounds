//! Deterministic team visibility and last-observed structure health (M-50–M-55).

use crate::geom::Pos;
use crate::grid::Grid;
use crate::state::MobaMatch;
use crate::types::Team;

pub struct Vision<'a> {
    game: &'a MobaMatch,
    team: Team,
    pub visible: Vec<bool>,
}
impl<'a> Vision<'a> {
    pub fn new(game: &'a MobaMatch, team: Team) -> Self {
        // Post-movement positions and post-death slots differ from the combat grid.
        let mut grid = Grid::default();
        grid.rebuild(game.units.iter().map(|u| u.pos));
        let mut visible = vec![false; game.units.len()];
        for ally in game
            .units
            .iter()
            .filter(|u| u.alive() && u.team() == Some(team))
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
            visible,
        }
    }
    pub fn point(&self, point: Pos) -> bool {
        self.game
            .units
            .iter()
            .any(|u| u.alive() && u.team() == Some(self.team) && u.pos.within(point, u.stats.sight))
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
