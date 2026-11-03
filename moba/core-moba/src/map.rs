//! The Tupi Valley 5v5 map: one source of truth for rules and rendering (M-02).
//!
//! Blue owns the south-west corner, Red the north-east. Red's layout is the
//! reflection of Blue's across the anti-diagonal `x + y = SIZE`, so both
//! sides share every lane and the river splits the jungles.

use crate::geom::Pos;
use crate::types::{Lane, Team, TowerTier};

/// Map side in world units.
pub const SIZE: i32 = 200;
/// Units cannot walk closer than this to the map border.
pub const EDGE: i32 = 6;
/// Half-width of the shared road and deployment strip, in millimetres.
pub const LANE_HALF_WIDTH: i32 = 3_600;

/// Blue's view of each lane, from its Stronghold to Red's.
const TOP: [(i32, i32); 8] = [
    (24, 24),
    (20, 50),
    (20, 110),
    (25, 164),
    (36, 175),
    (90, 180),
    (150, 180),
    (176, 176),
];
const MID: [(i32, i32); 5] = [(24, 24), (60, 60), (100, 100), (140, 140), (176, 176)];
const BOT: [(i32, i32); 8] = [
    (24, 24),
    (50, 20),
    (110, 20),
    (164, 25),
    (175, 36),
    (180, 90),
    (180, 150),
    (176, 176),
];

/// Blue towers, standing beside (never on) their road; Red's are their [`mirror`].
const BLUE_TOWERS: [(TowerTier, Option<Lane>, (i32, i32)); 11] = [
    (TowerTier::Outer, Some(Lane::Top), (25, 112)),
    (TowerTier::Inner, Some(Lane::Top), (25, 78)),
    (TowerTier::Base, Some(Lane::Top), (26, 50)),
    (TowerTier::Outer, Some(Lane::Mid), (73, 79)),
    (TowerTier::Inner, Some(Lane::Mid), (56, 62)),
    (TowerTier::Base, Some(Lane::Mid), (40, 46)),
    (TowerTier::Outer, Some(Lane::Bot), (112, 25)),
    (TowerTier::Inner, Some(Lane::Bot), (78, 25)),
    (TowerTier::Base, Some(Lane::Bot), (50, 26)),
    (TowerTier::Nexus, None, (31, 38)),
    (TowerTier::Nexus, None, (38, 31)),
];

/// Blue-side camps (between Blue's lanes, south of the river); Red's are mirrored.
const BLUE_CAMPS: [(i32, i32); 4] = [(48, 86), (72, 112), (86, 48), (112, 72)];

/// River centre line, corner to corner (presentation only; it never slows
/// units). The top and bot lanes cross it on bridges near the corners.
pub const RIVER: [(i32, i32); 7] = [
    (8, 192),
    (40, 160),
    (70, 130),
    (100, 100),
    (130, 70),
    (160, 40),
    (192, 8),
];
pub const RIVER_HALF_WIDTH: i32 = 6;
/// The stone plaza where the mid lane crosses the river.
pub const PLAZA: (i32, i32) = (100, 100);
pub const PLAZA_RADIUS: i32 = 10;
/// Radius of each base plateau around a Stronghold.
pub const BASE_RADIUS: i32 = 26;
pub const CAMP_RADIUS: i32 = 7;

/// Reflection across the anti-diagonal: Blue layout → Red layout.
pub const fn mirror((x, y): (i32, i32)) -> (i32, i32) {
    (SIZE - y, SIZE - x)
}

pub fn stronghold(team: Team) -> Pos {
    match team {
        Team::Blue => Pos::units(24, 24),
        Team::Red => {
            let (x, y) = mirror((24, 24));
            Pos::units(x, y)
        }
    }
}

/// Lane polyline in whole units from Blue's Stronghold to Red's.
pub fn lane_units(lane: Lane) -> &'static [(i32, i32)] {
    match lane {
        Lane::Top => &TOP,
        Lane::Mid => &MID,
        Lane::Bot => &BOT,
    }
}

/// Waypoint `index` of `lane` walked by `team` (Red walks it backwards).
pub fn waypoint(lane: Lane, team: Team, index: usize) -> Pos {
    let path = lane_units(lane);
    let i = index.min(path.len() - 1);
    let (x, y) = match team {
        Team::Blue => path[i],
        Team::Red => path[path.len() - 1 - i],
    };
    Pos::units(x, y)
}

pub fn waypoint_count(lane: Lane) -> usize {
    lane_units(lane).len()
}

/// Index of the waypoint nearest to `p` on `lane` for `team`, used when a
/// unit joins a lane mid-way.
pub fn nearest_waypoint(lane: Lane, team: Team, p: Pos) -> usize {
    (0..waypoint_count(lane))
        .min_by_key(|&i| waypoint(lane, team, i).dist2(p))
        .unwrap_or(0)
}

/// Every tower of both teams in a fixed order (Blue first).
pub fn towers() -> impl Iterator<Item = (Team, TowerTier, Option<Lane>, Pos)> {
    Team::ALL.into_iter().flat_map(|team| {
        BLUE_TOWERS.iter().map(move |&(tier, lane, at)| {
            let (x, y) = match team {
                Team::Blue => at,
                Team::Red => mirror(at),
            };
            (team, tier, lane, Pos::units(x, y))
        })
    })
}

pub const CAMP_COUNT: usize = 8;

/// Camp centre and the team whose jungle it sits in; ids `0..4` Blue, `4..8` Red.
pub fn camp(id: usize) -> (Pos, Team) {
    let (team, at) = if id < 4 {
        (Team::Blue, BLUE_CAMPS[id])
    } else {
        (Team::Red, mirror(BLUE_CAMPS[id % 4]))
    };
    (Pos::units(at.0, at.1), team)
}

/// Where a team's purchased Troops appear.
pub fn spawn_point(team: Team) -> Pos {
    let (x, y) = match team {
        Team::Blue => (32, 32),
        Team::Red => mirror((32, 32)),
    };
    Pos::units(x, y)
}

/// Each side has the same four camp kinds in the same order (M-60).
pub fn camp_kind(id: usize) -> crate::types::CampKind {
    use crate::types::CampKind;
    [
        CampKind::Wolves,
        CampKind::Golems,
        CampKind::Jaguar,
        CampKind::Frogs,
    ][id % 4]
}
