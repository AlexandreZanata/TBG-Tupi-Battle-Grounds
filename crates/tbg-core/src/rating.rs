//! Elo rating (BR-R03, BR-R05; GAME-DESIGN §5 "Ranked rating"). Pure math
//! on copies — never simulation state, so `f64` is fine here. Each
//! player's delta uses their own K (placement players move without
//! inflating settled ones).

pub const START_RATING: u16 = 1000;
pub const RATING_FLOOR: u16 = 100;
pub const PLACEMENT_GAMES: u32 = 20;
pub const K_PLACEMENT: f64 = 40.0;
pub const K_STANDARD: f64 = 24.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Score {
    Win,
    Draw,
    Loss,
}

impl Score {
    fn value(self) -> f64 {
        match self {
            Score::Win => 1.0,
            Score::Draw => 0.5,
            Score::Loss => 0.0,
        }
    }
}

/// Expected score of `own` against `opp`.
pub fn expected(own: u16, opp: u16) -> f64 {
    1.0 / (1.0 + 10f64.powf((f64::from(opp) - f64::from(own)) / 400.0))
}

/// Rating change for `own` against `opp`, given `own`'s game count before
/// this match. A win is at least +1, a loss at most −1 (BR-R05).
pub fn delta(own: u16, opp: u16, own_games_before: u32, score: Score) -> i32 {
    let k = if own_games_before < PLACEMENT_GAMES {
        K_PLACEMENT
    } else {
        K_STANDARD
    };
    let raw = (k * (score.value() - expected(own, opp))).round() as i32;
    match score {
        Score::Win => raw.max(1),
        Score::Loss => raw.min(-1),
        Score::Draw => raw,
    }
}

/// Apply a delta, clamped to the floor (BR-R05) and `u16` range.
pub fn apply(rating: u16, delta: i32) -> u16 {
    (i32::from(rating) + delta)
        .max(i32::from(RATING_FLOOR))
        .min(i32::from(u16::MAX)) as u16
}
