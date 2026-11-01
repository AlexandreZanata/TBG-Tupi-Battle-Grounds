//! Short-lived shared-mesh smoke, warm flames and material-matched damage fragments.
use super::{spawn, Motion, VfxAssets};
use bevy::prelude::*;

pub fn burn(c: &mut Commands, a: &VfxAssets, at: Vec3, fire: bool, stage: u8) {
    if !crate::config::low_motion() {
        spawn(
            c,
            &a.puff,
            &a.soot,
            at + Vec3::new(0.0, 0.20, -0.06),
            Motion::Puff {
                vel: Vec3::new(0.03, 0.35, 0.0),
                size: 0.025 + f32::from(stage) * 0.001,
            },
            0.0,
            0.8,
        );
    }
    if fire {
        spawn(
            c,
            &a.spike,
            &a.orange,
            at,
            Motion::Sprout {
                size: Vec3::new(0.15, 0.42, 0.15),
                hold: 0.12,
            },
            0.0,
            0.38,
        );
        spawn(
            c,
            &a.spike,
            &a.fire,
            at + Vec3::Y * 0.015,
            Motion::Sprout {
                size: Vec3::new(0.07, 0.28, 0.07),
                hold: 0.10,
            },
            0.0,
            0.32,
        );
    }
}

pub fn debris(c: &mut Commands, a: &VfxAssets, at: Vec3, skipped: u8, timber: bool) {
    let count = if crate::config::low_motion() {
        1
    } else {
        skipped.min(4)
    };
    for i in 0..count {
        let sign = if i % 2 == 0 { 1.0 } else { -1.0 };
        spawn(
            c,
            &a.chip,
            if timber { &a.root } else { &a.rock },
            at + Vec3::new(sign * 0.45, 0.8, 0.5),
            Motion::Debris {
                vel: Vec3::new(sign * 0.6, 0.7, 0.3),
                size: 0.065,
            },
            0.0,
            0.9,
        );
    }
}
