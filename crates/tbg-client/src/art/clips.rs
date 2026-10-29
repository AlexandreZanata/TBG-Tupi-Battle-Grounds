//! Keyframe tables for the shared animation library (data, like
//! `tbg-core`'s content tables). Times are normalized 0..1 per clip.

use super::anim::{ClipEvent, Easing};
use super::rig::Joint;
use Easing::CubicInOut;

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
pub(super) type Key = (f32, [f32; 3], [f32; 3], Easing);
type Track = (Joint, &'static [Key]);
const Z: [f32; 3] = [0.0; 3];

use Joint::*;

#[rustfmt::skip]
const IDLE: &[Track] = &[
    (Chest, &[(0.0, Z, Z, CubicInOut), (0.5, [2.0, 0.0, 0.0], [0.0, 0.012, 0.0], CubicInOut),
            (1.0, Z, Z, CubicInOut),
        ],
    ),
    (Head, &[(0.0, Z, Z, CubicInOut), (0.5, [4.0, 6.0, 0.0], Z, CubicInOut), (1.0, Z, Z, CubicInOut)]),
    (UpperArmL, &[(0.0, [0.0, 0.0, 6.0], Z, CubicInOut), (0.5, [0.0, 0.0, 10.0], Z, CubicInOut), (1.0, [0.0, 0.0, 6.0], Z, CubicInOut)],),
    (UpperArmR, &[(0.0, [0.0, 0.0, -6.0], Z, CubicInOut), (0.5, [0.0, 0.0, -10.0], Z, CubicInOut), (1.0, [0.0, 0.0, -6.0], Z, CubicInOut)],),
];
#[rustfmt::skip]
const WALK: &[Track] = &[
    (ThighL, &[(0.0, [32.0, 0.0, 0.0], Z, CubicInOut), (0.5, [-32.0, 0.0, 0.0], Z, CubicInOut), (1.0, [32.0, 0.0, 0.0], Z, CubicInOut)],),
    (ThighR, &[(0.0, [-32.0, 0.0, 0.0], Z, CubicInOut), (0.5, [32.0, 0.0, 0.0], Z, CubicInOut), (1.0, [-32.0, 0.0, 0.0], Z, CubicInOut)],),
    (UpperArmL, &[(0.0, [-28.0, 0.0, 6.0], Z, CubicInOut), (0.5, [28.0, 0.0, 6.0], Z, CubicInOut), (1.0, [-28.0, 0.0, 6.0], Z, CubicInOut)],),
    (UpperArmR, &[(0.0, [28.0, 0.0, -6.0], Z, CubicInOut), (0.5, [-28.0, 0.0, -6.0], Z, CubicInOut), (1.0, [28.0, 0.0, -6.0], Z, CubicInOut)],),
    (Hips, &[(0.0, Z, [0.0, 0.035, 0.0], CubicInOut),
            (0.25, Z, Z, CubicInOut),
            (0.5, Z, [0.0, 0.035, 0.0], CubicInOut),
            (0.75, Z, Z, CubicInOut),
            (1.0, Z, [0.0, 0.035, 0.0], CubicInOut),
        ],
    ),
    (Chest, &[(0.0, [4.0, 6.0, 0.0], Z, CubicInOut), (0.5, [4.0, -6.0, 0.0], Z, CubicInOut), (1.0, [4.0, 6.0, 0.0], Z, CubicInOut)],),
];
#[rustfmt::skip]
const MELEE: &[Track] = &[
    (UpperArmR, &[(0.0, Z, Z, CubicInOut), (0.45, [-135.0, 0.0, -10.0], Z, CubicInOut), (0.6, [55.0, 0.0, 0.0], Z, CubicInOut), (1.0, Z, Z, CubicInOut)],),
    (ForearmR, &[(0.0, Z, Z, CubicInOut), (0.45, [-45.0, 0.0, 0.0], Z, CubicInOut), (0.6, Z, Z, CubicInOut)],),
    (Chest, &[(0.0, Z, Z, CubicInOut), (0.45, [-6.0, -22.0, 0.0], Z, CubicInOut), (0.6, [10.0, 26.0, 0.0], Z, CubicInOut), (1.0, Z, Z, CubicInOut)],),
    (Hips, &[(0.0, Z, Z, CubicInOut), (0.6, Z, [0.0, -0.03, 0.06], CubicInOut), (1.0, Z, Z, CubicInOut)],
    ),
];
#[rustfmt::skip]
const RANGED: &[Track] = &[
    (UpperArmL, &[(0.0, Z, Z, CubicInOut), (0.25, [-88.0, 0.0, 0.0], Z, CubicInOut), (0.8, [-88.0, 0.0, 0.0], Z, CubicInOut), (1.0, Z, Z, CubicInOut)],),
    (UpperArmR, &[(0.0, Z, Z, CubicInOut), (0.25, [-82.0, 0.0, 0.0], Z, CubicInOut), (0.55, [-82.0, 25.0, 0.0], Z, CubicInOut), (0.62, [-70.0, 0.0, 0.0], Z, CubicInOut), (1.0, Z, Z, CubicInOut)],),
    (ForearmR, &[(0.0, Z, Z, CubicInOut), (0.25, [-20.0, 0.0, 0.0], Z, CubicInOut), (0.55, [-75.0, 0.0, 0.0], Z, CubicInOut), (0.62, Z, Z, CubicInOut)],),
    (Chest, &[(0.0, Z, Z, CubicInOut), (0.55, [0.0, 0.0, 0.0], Z, CubicInOut), (0.66, [-9.0, 0.0, 0.0], Z, CubicInOut), (1.0, Z, Z, CubicInOut)],),
];
#[rustfmt::skip]
const CAST: &[Track] = &[
    (UpperArmL, &[(0.0, Z, Z, CubicInOut), (0.4, [-155.0, 0.0, 22.0], Z, CubicInOut), (0.6, [-105.0, 0.0, 10.0], Z, CubicInOut), (1.0, Z, Z, CubicInOut)],),
    (UpperArmR, &[(0.0, Z, Z, CubicInOut), (0.4, [-155.0, 0.0, -22.0], Z, CubicInOut), (0.6, [-105.0, 0.0, -10.0], Z, CubicInOut), (1.0, Z, Z, CubicInOut)],),
    (Chest, &[(0.0, Z, Z, CubicInOut), (0.4, [-9.0, 0.0, 0.0], Z, CubicInOut), (0.6, [12.0, 0.0, 0.0], Z, CubicInOut), (1.0, Z, Z, CubicInOut)],),
    (Hips, &[(0.0, Z, Z, CubicInOut), (0.4, Z, [0.0, 0.06, 0.0], CubicInOut), (0.7, Z, Z, CubicInOut)],
    ),
];
#[rustfmt::skip]
const HIT: &[Track] = &[
    (Chest, &[(0.0, Z, Z, CubicInOut), (0.3, [-20.0, 0.0, 7.0], Z, CubicInOut), (1.0, Z, Z, CubicInOut)],),
    (Head, &[(0.0, Z, Z, CubicInOut), (0.3, [-16.0, 0.0, 0.0], Z, CubicInOut), (1.0, Z, Z, CubicInOut)],),
    (Hips, &[(0.0, Z, Z, CubicInOut), (0.3, Z, [0.0, 0.0, -0.07], CubicInOut), (1.0, Z, Z, CubicInOut)],
    ),
];
#[rustfmt::skip]
const DEATH: &[Track] = &[
    (Root, &[(0.0, Z, Z, CubicInOut), (0.75, [0.0, 0.0, 88.0], [0.0, 0.04, 0.0], CubicInOut),
            (1.0, [0.0, 0.0, 90.0], [0.0, -0.05, 0.0], CubicInOut),
        ],
    ),
    (UpperArmL, &[(0.0, Z, Z, CubicInOut), (0.75, [-60.0, 0.0, 40.0], Z, CubicInOut)]),
    (UpperArmR, &[(0.0, Z, Z, CubicInOut), (0.75, [-30.0, 0.0, -50.0], Z, CubicInOut)]),
    (Head, &[(0.0, Z, Z, CubicInOut), (0.75, [20.0, 0.0, 0.0], Z, CubicInOut)]),
];
#[rustfmt::skip]
const SPAWN: &[Track] = &[
    (Root, &[(0.0, Z, [0.0, 0.7, 0.0], CubicInOut),
            (0.65, Z, [0.0, -0.04, 0.0], CubicInOut),
            (1.0, Z, Z, CubicInOut),
        ],
    ),
    (UpperArmL, &[(0.0, [0.0, 0.0, 70.0], Z, CubicInOut), (0.65, [0.0, 0.0, 20.0], Z, CubicInOut), (1.0, Z, Z, CubicInOut)],),
    (UpperArmR, &[(0.0, [0.0, 0.0, -70.0], Z, CubicInOut), (0.65, [0.0, 0.0, -20.0], Z, CubicInOut), (1.0, Z, Z, CubicInOut)],),
];
#[rustfmt::skip]
const EVOLVE: &[Track] = &[
    (Root, &[(0.0, Z, Z, CubicInOut), (0.5, [0.0, 180.0, 0.0], [0.0, 0.4, 0.0], CubicInOut),
            (1.0, [0.0, 360.0, 0.0], Z, CubicInOut),
        ],
    ),
    (UpperArmL, &[(0.0, Z, Z, CubicInOut), (0.5, [-160.0, 0.0, 25.0], Z, CubicInOut), (1.0, Z, Z, CubicInOut)],),
    (UpperArmR, &[(0.0, Z, Z, CubicInOut), (0.5, [-160.0, 0.0, -25.0], Z, CubicInOut), (1.0, Z, Z, CubicInOut)],),
];
#[rustfmt::skip]
const VICTORY: &[Track] = &[
    (Root, &[(0.0, Z, Z, CubicInOut), (0.5, Z, [0.0, 0.16, 0.0], CubicInOut), (1.0, Z, Z, CubicInOut)],
    ),
    (UpperArmL, &[(0.0, [-165.0, 0.0, 25.0], Z, CubicInOut), (1.0, [-165.0, 0.0, 25.0], Z, CubicInOut)],),
    (UpperArmR, &[(0.0, [-165.0, 0.0, -25.0], Z, CubicInOut), (1.0, [-165.0, 0.0, -25.0], Z, CubicInOut)],),
];

impl Clip {
    pub fn events(self) -> &'static [(f32, ClipEvent)] {
        match self {
            Clip::Walk => &[(0.0, ClipEvent::Footstep), (0.3, ClipEvent::Footstep)],
            Clip::AttackMelee => &[(0.30, ClipEvent::Strike)],
            Clip::AttackRanged => &[(0.18, ClipEvent::Release)],
            Clip::Cast => &[(0.22, ClipEvent::Release)],
            _ => &[],
        }
    }

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
