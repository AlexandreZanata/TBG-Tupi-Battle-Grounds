//! The effect recipes: which shared shapes, colors, and motions each uses.

use bevy::prelude::*;

use super::{spawn, Motion, VfxAssets};

/// Splash damage lands after the attack windup (0.3 s).
pub fn splash(commands: &mut Commands, a: &VfxAssets, at: Vec3, radius_tiles: f32) {
    let at = Vec3::new(at.x, 0.06, at.z);
    spawn(
        commands,
        &a.ring,
        &a.white,
        at,
        Motion::Grow {
            radius: radius_tiles,
        },
        0.3,
        0.35,
    );
}

pub fn heal(commands: &mut Commands, a: &VfxAssets, at: Vec3) {
    for (dx, dz) in [(0.15, 0.0), (-0.12, 0.1), (0.0, -0.14), (0.08, 0.12)] {
        spawn(
            commands,
            &a.ball,
            &a.green,
            at + Vec3::new(dx, 0.4, dz),
            Motion::Rise,
            0.0,
            0.6,
        );
    }
}

pub fn evolve(commands: &mut Commands, a: &VfxAssets, at: Vec3) {
    let ground = Vec3::new(at.x, 0.08, at.z);
    spawn(
        commands,
        &a.ring,
        &a.gold,
        ground,
        Motion::Grow { radius: 1.1 },
        0.0,
        0.5,
    );
    for i in 0..6 {
        let ang = i as f32 / 6.0 * std::f32::consts::TAU;
        let dir = Vec3::new(ang.cos(), 0.8, ang.sin());
        spawn(
            commands,
            &a.ball,
            &a.gold,
            at + Vec3::Y * 0.6,
            Motion::Burst { dir },
            0.0,
            0.5,
        );
    }
}

pub fn synergy(commands: &mut Commands, a: &VfxAssets, at: Vec3, own: bool) {
    let mat = if own { &a.ally } else { &a.enemy };
    spawn(
        commands,
        &a.ring,
        mat,
        Vec3::new(at.x, 0.1, at.z),
        Motion::Grow { radius: 0.8 },
        0.0,
        0.6,
    );
}

pub fn sparks(commands: &mut Commands, a: &VfxAssets, at: Vec3) {
    for i in 0..5 {
        let ang = i as f32 * 1.3;
        let dir = Vec3::new(ang.cos() * 0.6, 1.0, ang.sin() * 0.6);
        spawn(
            commands,
            &a.ball,
            &a.orange,
            at,
            Motion::Burst { dir },
            0.0,
            0.35,
        );
    }
}

pub fn puff(commands: &mut Commands, a: &VfxAssets, at: Vec3) {
    for i in 0..6 {
        let ang = i as f32 / 6.0 * std::f32::consts::TAU;
        let dir = Vec3::new(ang.cos(), 0.3, ang.sin()) * 0.7;
        spawn(
            commands,
            &a.ball,
            &a.grey,
            at + Vec3::Y * 0.3,
            Motion::Burst { dir },
            0.55,
            0.45,
        );
    }
}
