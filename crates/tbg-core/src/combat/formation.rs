//! Formation queries over live positions. Only static ability parameters
//! are indexed for this combat pass; movement and death are never cached.

use crate::content::Ability;
use crate::state::{MatchState, Unit};

struct Anchor {
    index: usize,
    engage: i32,
    gap: i32,
}

pub(super) struct FormationIndex {
    /// Discovered on the first movement of the pass: a tick where nobody
    /// moves (engaged armies) skips the per-unit ability scan entirely.
    anchors: Option<Vec<Anchor>>,
    enemies: Vec<i32>,
}

impl FormationIndex {
    /// Unit indices and forms stay fixed during the combat pass (spawns
    /// happen before it, removals after it), so discovering the anchors
    /// lazily finds the same set as discovering them up front.
    pub(super) fn new() -> Self {
        Self {
            anchors: None,
            enemies: Vec::new(),
        }
    }

    fn discover(state: &MatchState) -> Vec<Anchor> {
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
                        Ability::Formation { engage, gap } => Some(Anchor { index, engage, gap }),
                        _ => None,
                    })
            })
            .collect()
    }

    pub(super) fn clamp(&mut self, state: &MatchState, u: &Unit, new_x: i32) -> i32 {
        if self.anchors.is_none() {
            let found = Self::discover(state);
            if !found.is_empty() {
                self.enemies.reserve(state.units.len());
            }
            self.anchors = Some(found);
        }
        let anchors = self.anchors.as_deref().unwrap_or(&[]);
        let ahead = |a: &&Anchor| {
            let oil = &state.units[a.index];
            oil.seat == u.seat && oil.hp > 0 && oil.id != u.id && (oil.x - u.x) * u.seat.dir() > 0
        };
        if !anchors.iter().any(|a| ahead(&a)) {
            return new_x;
        }
        // Rebuild from the CURRENT state for this moving unit. A start-of-
        // tick snapshot would miss earlier movement and deaths in id order.
        self.enemies.clear();
        self.enemies.extend(
            state
                .units
                .iter()
                .filter(|e| e.seat != u.seat && e.hp > 0)
                .map(|e| e.x),
        );
        self.enemies.sort_unstable();
        let anchor = anchors
            .iter()
            .filter(ahead)
            .filter(|a| {
                let x = state.units[a.index].x;
                engaged(&self.enemies, x, u.seat.dir(), a.engage)
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
