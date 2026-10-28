//! Keyframe tables for the shared animation library (data, like
//! `tbg-core`'s content tables). Times are normalized 0..1 per clip.

use super::legacy::Joint;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Clip {
    Idle,
    Walk,
    AttackMelee,
    AttackRanged,
    Cast,
    Hit,
    Death,
    Spawn,
    Evolve,
    Victory,
}

pub const CLIPS: [Clip; 10] = [
    Clip::Idle,
    Clip::Walk,
    Clip::AttackMelee,
    Clip::AttackRanged,
    Clip::Cast,
    Clip::Hit,
    Clip::Death,
    Clip::Spawn,
    Clip::Evolve,
    Clip::Victory,
];

/// (normalized time, rotation degrees XYZ, offset).
pub(super) type Key = (f32, [f32; 3], [f32; 3]);
type Track = (Joint, &'static [Key]);
const Z: [f32; 3] = [0.0; 3];

use Joint::*;

#[rustfmt::skip]
const IDLE: &[Track] = &[
    (Torso, &[(0.0, Z, Z), (0.5, [2.0, 0.0, 0.0], [0.0, 0.012, 0.0]),
            (1.0, Z, Z),
        ],
    ),
    (Head, &[(0.0, Z, Z), (0.5, [4.0, 6.0, 0.0], Z), (1.0, Z, Z)]),
    (ArmL, &[(0.0, [0.0, 0.0, 6.0], Z), (0.5, [0.0, 0.0, 10.0], Z), (1.0, [0.0, 0.0, 6.0], Z)],),
    (ArmR, &[(0.0, [0.0, 0.0, -6.0], Z), (0.5, [0.0, 0.0, -10.0], Z), (1.0, [0.0, 0.0, -6.0], Z)],),
];
#[rustfmt::skip]
const WALK: &[Track] = &[
    (LegL, &[(0.0, [32.0, 0.0, 0.0], Z), (0.5, [-32.0, 0.0, 0.0], Z), (1.0, [32.0, 0.0, 0.0], Z)],),
    (LegR, &[(0.0, [-32.0, 0.0, 0.0], Z), (0.5, [32.0, 0.0, 0.0], Z), (1.0, [-32.0, 0.0, 0.0], Z)],),
    (ArmL, &[(0.0, [-28.0, 0.0, 6.0], Z), (0.5, [28.0, 0.0, 6.0], Z), (1.0, [-28.0, 0.0, 6.0], Z)],),
    (ArmR, &[(0.0, [28.0, 0.0, -6.0], Z), (0.5, [-28.0, 0.0, -6.0], Z), (1.0, [28.0, 0.0, -6.0], Z)],),
    (Hips, &[(0.0, Z, [0.0, 0.035, 0.0]),
            (0.25, Z, Z),
            (0.5, Z, [0.0, 0.035, 0.0]),
            (0.75, Z, Z),
            (1.0, Z, [0.0, 0.035, 0.0]),
        ],
    ),
    (Torso, &[(0.0, [4.0, 6.0, 0.0], Z), (0.5, [4.0, -6.0, 0.0], Z), (1.0, [4.0, 6.0, 0.0], Z)],),
];
#[rustfmt::skip]
const MELEE: &[Track] = &[
    (ArmR, &[(0.0, Z, Z), (0.45, [-135.0, 0.0, -10.0], Z), (0.65, [55.0, 0.0, 0.0], Z), (1.0, Z, Z)],),
    (ForearmR, &[(0.0, Z, Z), (0.45, [-45.0, 0.0, 0.0], Z), (0.65, Z, Z)],),
    (Torso, &[(0.0, Z, Z), (0.45, [-6.0, -22.0, 0.0], Z), (0.65, [10.0, 26.0, 0.0], Z), (1.0, Z, Z)],),
    (Hips, &[(0.0, Z, Z), (0.65, Z, [0.0, -0.03, 0.06]), (1.0, Z, Z)],
    ),
];
#[rustfmt::skip]
const RANGED: &[Track] = &[
    (ArmL, &[(0.0, Z, Z), (0.25, [-88.0, 0.0, 0.0], Z), (0.8, [-88.0, 0.0, 0.0], Z), (1.0, Z, Z)],),
    (ArmR, &[(0.0, Z, Z), (0.25, [-82.0, 0.0, 0.0], Z), (0.55, [-82.0, 25.0, 0.0], Z), (0.62, [-70.0, 0.0, 0.0], Z), (1.0, Z, Z)],),
    (ForearmR, &[(0.0, Z, Z), (0.25, [-20.0, 0.0, 0.0], Z), (0.55, [-75.0, 0.0, 0.0], Z), (0.62, Z, Z)],),
    (Torso, &[(0.0, Z, Z), (0.55, [0.0, 0.0, 0.0], Z), (0.66, [-9.0, 0.0, 0.0], Z), (1.0, Z, Z)],),
];
#[rustfmt::skip]
const CAST: &[Track] = &[
    (ArmL, &[(0.0, Z, Z), (0.4, [-155.0, 0.0, 22.0], Z), (0.6, [-105.0, 0.0, 10.0], Z), (1.0, Z, Z)],),
    (ArmR, &[(0.0, Z, Z), (0.4, [-155.0, 0.0, -22.0], Z), (0.6, [-105.0, 0.0, -10.0], Z), (1.0, Z, Z)],),
    (Torso, &[(0.0, Z, Z), (0.4, [-9.0, 0.0, 0.0], Z), (0.6, [12.0, 0.0, 0.0], Z), (1.0, Z, Z)],),
    (Hips, &[(0.0, Z, Z), (0.4, Z, [0.0, 0.06, 0.0]), (0.7, Z, Z)],
    ),
];
#[rustfmt::skip]
const HIT: &[Track] = &[
    (Torso, &[(0.0, Z, Z), (0.3, [-20.0, 0.0, 7.0], Z), (1.0, Z, Z)],),
    (Head, &[(0.0, Z, Z), (0.3, [-16.0, 0.0, 0.0], Z), (1.0, Z, Z)],),
    (Hips, &[(0.0, Z, Z), (0.3, Z, [0.0, 0.0, -0.07]), (1.0, Z, Z)],
    ),
];
#[rustfmt::skip]
const DEATH: &[Track] = &[
    (Root, &[(0.0, Z, Z), (0.75, [0.0, 0.0, 88.0], [0.0, 0.04, 0.0]),
            (1.0, [0.0, 0.0, 90.0], [0.0, -0.05, 0.0]),
        ],
    ),
    (ArmL, &[(0.0, Z, Z), (0.75, [-60.0, 0.0, 40.0], Z)]),
    (ArmR, &[(0.0, Z, Z), (0.75, [-30.0, 0.0, -50.0], Z)]),
    (Head, &[(0.0, Z, Z), (0.75, [20.0, 0.0, 0.0], Z)]),
];
#[rustfmt::skip]
const SPAWN: &[Track] = &[
    (Root, &[(0.0, Z, [0.0, 0.7, 0.0]),
            (0.65, Z, [0.0, -0.04, 0.0]),
            (1.0, Z, Z),
        ],
    ),
    (ArmL, &[(0.0, [0.0, 0.0, 70.0], Z), (0.65, [0.0, 0.0, 20.0], Z), (1.0, Z, Z)],),
    (ArmR, &[(0.0, [0.0, 0.0, -70.0], Z), (0.65, [0.0, 0.0, -20.0], Z), (1.0, Z, Z)],),
];
#[rustfmt::skip]
const EVOLVE: &[Track] = &[
    (Root, &[(0.0, Z, Z), (0.5, [0.0, 180.0, 0.0], [0.0, 0.4, 0.0]),
            (1.0, [0.0, 360.0, 0.0], Z),
        ],
    ),
    (ArmL, &[(0.0, Z, Z), (0.5, [-160.0, 0.0, 25.0], Z), (1.0, Z, Z)],),
    (ArmR, &[(0.0, Z, Z), (0.5, [-160.0, 0.0, -25.0], Z), (1.0, Z, Z)],),
];
#[rustfmt::skip]
const VICTORY: &[Track] = &[
    (Root, &[(0.0, Z, Z), (0.5, Z, [0.0, 0.16, 0.0]), (1.0, Z, Z)],
    ),
    (ArmL, &[(0.0, [-165.0, 0.0, 25.0], Z), (1.0, [-165.0, 0.0, 25.0], Z)],),
    (ArmR, &[(0.0, [-165.0, 0.0, -25.0], Z), (1.0, [-165.0, 0.0, -25.0], Z)],),
];

impl Clip {
    pub fn duration(self) -> f32 {
        match self {
            Clip::Idle => 1.6,
            Clip::Walk => 0.6,
            Clip::AttackMelee | Clip::AttackRanged => 0.5,
            Clip::Cast => 0.6,
            Clip::Hit => 0.25,
            Clip::Death => 0.6,
            Clip::Spawn => 0.35,
            Clip::Evolve => 0.5,
            Clip::Victory => 0.8,
        }
    }

    pub fn looping(self) -> bool {
        matches!(self, Clip::Idle | Clip::Walk | Clip::Victory)
    }

    pub(super) fn tracks(self) -> &'static [Track] {
        match self {
            Clip::Idle => IDLE,
            Clip::Walk => WALK,
            Clip::AttackMelee => MELEE,
            Clip::AttackRanged => RANGED,
            Clip::Cast => CAST,
            Clip::Hit => HIT,
            Clip::Death => DEATH,
            Clip::Spawn => SPAWN,
            Clip::Evolve => EVOLVE,
            Clip::Victory => VICTORY,
        }
    }
}
