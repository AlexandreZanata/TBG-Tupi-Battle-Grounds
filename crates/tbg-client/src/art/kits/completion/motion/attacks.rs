//! Completion-owned attack tracks retain contact/release timing after PR44 removes unused common tables.
use crate::art::{anim::Easing::*, kits::tools::Track, rig::Joint::*};
const Z: [f32; 3] = [0.0; 3];

#[rustfmt::skip]
pub(super) const CHOP: &[Track] = &[
    (UpperArmR, &[(0.0, [-30.0, 0.0, -10.0], Z, CubicInOut), (0.32, [-190.0, 0.0, -8.0], Z, CubicIn), (0.484, [-45.0, 0.0, -6.0], Z, Linear), (0.645, [-40.0, 0.0, -6.0], Z, CubicInOut), (1.0, [-30.0, 0.0, -10.0], Z, CubicInOut)]),
    (ForearmR, &[(0.0, [-40.0, 0.0, 0.0], Z, CubicInOut), (0.32, [-55.0, 0.0, 0.0], Z, CubicIn), (0.484, [-8.0, 0.0, 0.0], Z, Linear), (1.0, [-40.0, 0.0, 0.0], Z, CubicInOut)]),
    (UpperArmL, &[(0.0, [-30.0, 0.0, 10.0], Z, CubicInOut), (0.32, [-185.0, 0.0, 6.0], Z, CubicIn), (0.484, [-50.0, 0.0, 6.0], Z, Linear), (0.645, [-45.0, 0.0, 6.0], Z, CubicInOut), (1.0, [-30.0, 0.0, 10.0], Z, CubicInOut)]),
    (ForearmL, &[(0.0, [-40.0, 0.0, 0.0], Z, CubicInOut), (0.32, [-50.0, 0.0, 0.0], Z, CubicIn), (0.484, [-10.0, 0.0, 0.0], Z, Linear), (1.0, [-40.0, 0.0, 0.0], Z, CubicInOut)]),
    (Spine, &[(0.0, [4.0, 0.0, 0.0], Z, CubicInOut), (0.32, [-14.0, 0.0, 0.0], Z, CubicIn), (0.484, [26.0, 0.0, 0.0], Z, Linear), (0.645, [24.0, 0.0, 0.0], Z, CubicInOut), (1.0, [4.0, 0.0, 0.0], Z, CubicInOut)]),
    (Hips, &[(0.0, Z, [0.0, -0.02, 0.0], CubicInOut), (0.32, Z, [0.0, 0.02, -0.03], CubicIn), (0.484, Z, [0.0, -0.07, 0.06], Linear), (0.645, Z, [0.0, -0.06, 0.05], CubicInOut), (1.0, Z, [0.0, -0.02, 0.0], CubicInOut)]),
];

#[rustfmt::skip]
pub(super) const THRUST: &[Track] = &[
    (UpperArmR, &[(0.0, [-35.0, 0.0, -12.0], Z, CubicInOut), (0.32, [-60.0, 0.0, -25.0], Z, CubicIn), (0.484, [-92.0, 0.0, -4.0], Z, Linear), (0.645, [-88.0, 0.0, -4.0], Z, CubicInOut), (1.0, [-35.0, 0.0, -12.0], Z, CubicInOut)]),
    (ForearmR, &[(0.0, [-40.0, 0.0, 0.0], Z, CubicInOut), (0.32, [-95.0, 0.0, 0.0], Z, CubicIn), (0.484, [-4.0, 0.0, 0.0], Z, Linear), (1.0, [-40.0, 0.0, 0.0], Z, CubicInOut)]),
    (Chest, &[(0.0, [4.0, 0.0, 0.0], Z, CubicInOut), (0.32, [-4.0, -25.0, 0.0], Z, CubicIn), (0.484, [12.0, 20.0, 0.0], Z, Linear), (1.0, [4.0, 0.0, 0.0], Z, CubicInOut)]),
    (Hips, &[(0.0, Z, [0.0, -0.02, 0.0], CubicInOut), (0.32, Z, [0.0, -0.03, -0.06], CubicIn), (0.484, Z, [0.0, -0.07, 0.1], Linear), (0.645, Z, [0.0, -0.06, 0.09], CubicInOut), (1.0, Z, [0.0, -0.02, 0.0], CubicInOut)]),
];

#[rustfmt::skip]
pub(super) const THROW: &[Track] = &[
    (UpperArmR, &[(0.0, [-20.0, 0.0, -10.0], Z, CubicInOut), (0.2, [-170.0, 0.0, -25.0], Z, CubicIn), (0.29, [-80.0, 0.0, -10.0], Z, CubicOut), (0.45, [-30.0, 0.0, -8.0], Z, CubicInOut), (1.0, [-20.0, 0.0, -10.0], Z, CubicInOut)]),
    (ForearmR, &[(0.0, [-30.0, 0.0, 0.0], Z, CubicInOut), (0.2, [-90.0, 0.0, 0.0], Z, CubicIn), (0.29, [-10.0, 0.0, 0.0], Z, CubicOut), (1.0, [-30.0, 0.0, 0.0], Z, CubicInOut)]),
    (UpperArmL, &[(0.0, [-10.0, 0.0, 10.0], Z, CubicInOut), (0.2, [-70.0, 0.0, 20.0], Z, CubicIn), (0.29, [-20.0, 0.0, 25.0], Z, CubicOut), (1.0, [-10.0, 0.0, 10.0], Z, CubicInOut)]),
    (Chest, &[(0.0, [2.0, 0.0, 0.0], Z, CubicInOut), (0.2, [-12.0, -30.0, 0.0], Z, CubicIn), (0.29, [14.0, 25.0, 0.0], Z, CubicOut), (1.0, [2.0, 0.0, 0.0], Z, CubicInOut)]),
    (Hips, &[(0.0, Z, Z, CubicInOut), (0.2, Z, [0.0, 0.0, -0.05], CubicIn), (0.29, Z, [0.0, -0.04, 0.06], CubicOut), (1.0, Z, Z, CubicInOut)]),
];

#[rustfmt::skip]
pub(super) const POINT: &[Track] = &[
    (UpperArmR, &[(0.0, [-20.0, 0.0, -10.0], Z, CubicInOut), (0.25, [-150.0, 0.0, -20.0], Z, CubicIn), (0.355, [-85.0, 0.0, -6.0], Z, CubicOut), (0.6, [-80.0, 0.0, -6.0], Z, CubicInOut), (1.0, [-20.0, 0.0, -10.0], Z, CubicInOut)]),
    (ForearmR, &[(0.0, [-30.0, 0.0, 0.0], Z, CubicInOut), (0.25, [-20.0, 0.0, 0.0], Z, CubicIn), (0.355, [-5.0, 0.0, 0.0], Z, CubicOut), (1.0, [-30.0, 0.0, 0.0], Z, CubicInOut)]),
    (UpperArmL, &[(0.0, [-10.0, 0.0, 10.0], Z, CubicInOut), (0.25, [-40.0, 0.0, 40.0], Z, CubicIn), (0.355, [-20.0, 0.0, 30.0], Z, CubicOut), (1.0, [-10.0, 0.0, 10.0], Z, CubicInOut)]),
    (Chest, &[(0.0, Z, Z, CubicInOut), (0.25, [-10.0, -10.0, 0.0], Z, CubicIn), (0.355, [10.0, 10.0, 0.0], Z, CubicOut), (1.0, Z, Z, CubicInOut)]),
    (Hips, &[(0.0, Z, Z, CubicInOut), (0.25, Z, [0.0, 0.03, 0.0], CubicIn), (0.355, Z, [0.0, -0.03, 0.03], CubicOut), (1.0, Z, Z, CubicInOut)]),
];

#[rustfmt::skip]
pub(super) const CAST: &[Track] = &[
    (UpperArmL, &[(0.0, [-15.0, 0.0, 10.0], Z, CubicInOut), (0.25, [-160.0, 0.0, 25.0], Z, CubicIn), (0.355, [-95.0, 0.0, 8.0], Z, CubicOut), (0.6, [-90.0, 0.0, 8.0], Z, CubicInOut), (1.0, [-15.0, 0.0, 10.0], Z, CubicInOut)]),
    (UpperArmR, &[(0.0, [-15.0, 0.0, -10.0], Z, CubicInOut), (0.25, [-160.0, 0.0, -25.0], Z, CubicIn), (0.355, [-95.0, 0.0, -8.0], Z, CubicOut), (0.6, [-90.0, 0.0, -8.0], Z, CubicInOut), (1.0, [-15.0, 0.0, -10.0], Z, CubicInOut)]),
    (ForearmL, &[(0.0, [-20.0, 0.0, 0.0], Z, CubicInOut), (0.355, [-5.0, 0.0, 0.0], Z, CubicInOut), (1.0, [-20.0, 0.0, 0.0], Z, CubicInOut)]),
    (ForearmR, &[(0.0, [-20.0, 0.0, 0.0], Z, CubicInOut), (0.355, [-5.0, 0.0, 0.0], Z, CubicInOut), (1.0, [-20.0, 0.0, 0.0], Z, CubicInOut)]),
    (Chest, &[(0.0, Z, Z, CubicInOut), (0.25, [-12.0, 0.0, 0.0], Z, CubicIn), (0.355, [14.0, 0.0, 0.0], Z, CubicOut), (1.0, Z, Z, CubicInOut)]),
    (Hips, &[(0.0, Z, Z, CubicInOut), (0.25, Z, [0.0, 0.04, 0.0], CubicIn), (0.355, Z, [0.0, -0.05, 0.04], CubicOut), (1.0, Z, Z, CubicInOut)]),
];
