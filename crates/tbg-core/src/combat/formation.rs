//! Formation queries over live positions. Only static ability parameters
//! are indexed for this combat pass; movement and death are never cached.

use crate::content::Ability;
use crate::state::{MatchState, Unit};

struct Anchor {
    index: usize,
    engage: i32,
    gap: i32,
}

pub(super) struct FormationIndex(Vec<Anchor>);

impl FormationIndex {
    pub(super) fn new(state: &MatchState) -> Self {
        Self(
            state
                .units
                .iter()
                .enumerate()
                .filter_map(|(index, u)| {
                    state
                        .rules()
                        .def(u.kind)
                        .ability(u.tier, u.branch)
                        .into_iter()
                        .find_map(|ability| match ability {
                            Ability::Formation { engage, gap } => {
                                Some(Anchor { index, engage, gap })
                            }
                            _ => None,
                        })
                })
                .collect(),
        )
    }

    pub(super) fn clamp(&self, state: &MatchState, u: &Unit, new_x: i32) -> i32 {
        let ahead = |a: &&Anchor| {
            let oil = &state.units[a.index];
            oil.seat == u.seat && oil.hp > 0 && oil.id != u.id && (oil.x - u.x) * u.seat.dir() > 0
        };
        if !self.0.iter().any(|a| ahead(&a)) {
            return new_x;
        }
        // Rebuild from the CURRENT state for this moving unit. A start-of-
        // tick snapshot would miss earlier movement and deaths in id order.
        let mut enemy_x: Vec<i32> = state
            .units
            .iter()
            .filter(|e| e.seat != u.seat && e.hp > 0)
            .map(|e| e.x)
            .collect();
        enemy_x.sort_unstable();
        let anchor = self
            .0
            .iter()
            .filter(ahead)
            .filter(|a| {
                let x = state.units[a.index].x;
                engaged(&enemy_x, x, u.seat.dir(), a.engage)
            })
            .min_by_key(|a| ((state.units[a.index].x - u.x).abs(), a.index));
        let Some(a) = anchor else {
            return new_x;
        };
        let x = state.units[a.index].x;
        let stop = x - u.seat.dir() * a.gap;
        if u.seat.dir() > 0 && u.x <= stop && new_x > stop
            || u.seat.dir() < 0 && u.x >= stop && new_x < stop
        {
            stop
        } else {
            new_x
        }
    }
}

fn engaged(enemies: &[i32], x: i32, dir: i32, range: i32) -> bool {
    let at = enemies.partition_point(|&enemy| {
        #[cfg(feature = "bench-profile")]
        crate::profiling::FORMATION_CANDIDATES.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        if dir > 0 {
            enemy <= x
        } else {
            enemy < x
        }
    });
    let nearest = if dir > 0 {
        enemies.get(at)
    } else {
        at.checked_sub(1).and_then(|i| enemies.get(i))
    };
    nearest.is_some_and(|&enemy| (enemy - x).abs() <= range)
}
