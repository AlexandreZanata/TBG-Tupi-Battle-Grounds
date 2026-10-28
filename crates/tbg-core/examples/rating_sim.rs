//! Elo convergence check (P12, BR-R03): 60 players with hidden skill play
//! 6000 FIFO matches; the Spearman rank correlation between skill and
//! final rating must reach 0.85 with the mean staying near 1000.

use tbg_core::rating::{apply, delta, Score, START_RATING};
use tbg_core::Pcg32;

struct Player {
    skill: u32,
    rating: u16,
    games: u32,
}

fn win_probability(skill_a: u32, skill_b: u32) -> f64 {
    1.0 / (1.0 + 10f64.powf((skill_b as f64 - skill_a as f64) * 12.0 / 400.0))
}

/// Spearman rank correlation of two rank vectors (no ties in `xs`).
fn spearman(xs: &[u32], ys: &[u32]) -> f64 {
    let n = xs.len() as f64;
    let mut order: Vec<usize> = (0..xs.len()).collect();
    order.sort_by_key(|&i| ys[i]);
    let mut rank_of = vec![0.0; xs.len()];
    for (rank, &i) in order.iter().enumerate() {
        rank_of[i] = rank as f64;
    }
    let mean = (n - 1.0) / 2.0;
    let mut num = 0.0;
    let mut denom = 0.0;
    for i in 0..xs.len() {
        let dx = xs[i] as f64 - mean;
        let dy = rank_of[i] - mean;
        num += dx * dy;
        denom += dx * dx;
    }
    // Identical rank variance on both sides (both are permutations of ranks).
    num / denom
}

fn main() {
    let mut players: Vec<Player> = (0..60)
        .map(|skill| Player {
            skill,
            rating: START_RATING,
            games: 0,
        })
        .collect();
    let mut rng = Pcg32::new(0xe10, 77);
    for _ in 0..6000 {
        let a = rng.below(60) as usize;
        let mut b = rng.below(60) as usize;
        while b == a {
            b = rng.below(60) as usize;
        }
        let (sa, sb) = (players[a].skill, players[b].skill);
        let roll = f64::from(rng.below(1000)) / 1000.0;
        let (ca, cb) = if roll < 0.03 {
            (Score::Draw, Score::Draw)
        } else if roll < 0.03 + 0.97 * win_probability(sa, sb) {
            (Score::Win, Score::Loss)
        } else {
            (Score::Loss, Score::Win)
        };
        let (ra, rb, ga, gb) = (
            players[a].rating,
            players[b].rating,
            players[a].games,
            players[b].games,
        );
        players[a].rating = apply(ra, delta(ra, rb, ga, ca));
        players[b].rating = apply(rb, delta(rb, ra, gb, cb));
        players[a].games += 1;
        players[b].games += 1;
    }
    let skills: Vec<u32> = players.iter().map(|p| p.skill).collect();
    let ratings: Vec<u32> = players.iter().map(|p| u32::from(p.rating)).collect();
    let rho = spearman(&skills, &ratings);
    let mean = ratings.iter().sum::<u32>() as f64 / ratings.len() as f64;
    let games: Vec<u32> = players.iter().map(|p| p.games).collect();
    println!("mean rating {mean:.1}");
    println!(
        "min {} max {}",
        ratings.iter().min().unwrap(),
        ratings.iter().max().unwrap()
    );
    println!(
        "games per player min {} max {}",
        games.iter().min().unwrap(),
        games.iter().max().unwrap()
    );
    println!("spearman skill/rating {rho:.3}");
}
