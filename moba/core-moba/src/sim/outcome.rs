//! Time-limit resolution (M-30); Stronghold wins are set by `rewards`.

use crate::content::MATCH_TICKS;
use crate::state::{EndReason, Event, MobaMatch, Outcome};
use crate::types::{Team, UnitKind};

pub(super) fn check(m: &mut MobaMatch) {
    if m.outcome.is_some() || m.tick < MATCH_TICKS {
        return;
    }
    let key = |t: Team| {
        let stronghold = m
            .units
            .iter()
            .find(|u| u.team() == Some(t) && u.kind == UnitKind::Stronghold)
            .map_or(0, |u| u.hp as i64 * 1000 / u.max_hp.max(1) as i64);
        (m.towers_left(t), stronghold, m.score[t.index()])
    };
    let (blue, red) = (key(Team::Blue), key(Team::Red));
    let winner = match blue.cmp(&red) {
        std::cmp::Ordering::Greater => Some(Team::Blue),
        std::cmp::Ordering::Less => Some(Team::Red),
        std::cmp::Ordering::Equal => None,
    };
    let outcome = Outcome {
        winner,
        reason: EndReason::TimeLimit,
    };
    m.outcome = Some(outcome);
    m.events.push(Event::Ended(outcome));
}
