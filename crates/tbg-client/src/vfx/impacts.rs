//! Impact, muzzle and footfall recipes: which shared shapes, colors and
//! motions each uses. Radii come from the attacker's real splash.

use std::f32::consts::TAU;

use bevy::prelude::*;

use super::{spawn, spawn_turned, Motion, VfxAssets};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Impact {
    /// Siege Walker shells.
    Blast,
    GreatBlast,
    Shrapnel,
    Shockwave,
    /// Rootcaller eruptions.
    Roots,
    Strangle,
    Spores,
    /// Bone Brute strikes.
    Cleave,
    Butcher,
    Crush,
    Reap,
    /// Stronghold shots.
    Cannon,
    GreatCannon,
    Bolt,
}

/// What the attacker brings to an impact: splash radius (tiles, 0 = none)
/// and how long a root holds (seconds).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Area {
    pub splash: f32,
    pub hold: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Muzzle {
    Bombard,
    GreatBombard,
    Blunderbuss,
    Cannon,
    GreatCannon,
    Bolt,
}

/// `n` unit directions around +Y, starting at `phase`.
fn around(n: usize, phase: f32) -> impl Iterator<Item = Vec3> {
    (0..n).map(move |i| {
        let a = phase + i as f32 / n as f32 * TAU;
        Vec3::new(a.cos(), 0.0, a.sin())
    })
}

pub fn land(commands: &mut Commands, a: &VfxAssets, impact: Impact, at: Vec3, area: Area) {
    let ground = at.with_y(0.05);
    match impact {
        Impact::Blast => blast(commands, a, at, area.splash.max(0.6), 1.0),
        Impact::GreatBlast => {
            blast(commands, a, at, area.splash.max(0.8), 1.45);
            spawn(
                commands,
                &a.puff,
                &a.gold,
                at,
                Motion::Flash { size: 0.5 },
                0.0,
                0.16,
            );
        }
        Impact::Shrapnel => {
            blast(commands, a, at, area.splash.max(0.6), 1.1);
            for dir in around(9, 0.3) {
                let d = (dir + Vec3::Y * 0.15) * area.splash.max(1.0) * 1.2;
                spawn(
                    commands,
                    &a.ball,
                    &a.orange,
                    at,
                    Motion::Burst { dir: d },
                    0.0,
                    0.3,
                );
            }
        }
        Impact::Shockwave => {
            blast(commands, a, at, area.splash.max(0.6), 1.15);
            spawn(
                commands,
                &a.ring,
                &a.fire,
                ground,
                Motion::Grow {
                    radius: area.splash.max(1.0) * 0.6,
                },
                0.06,
                0.3,
            );
            roots_ring(
                commands,
                a,
                ground,
                &a.rock,
                7,
                (0.45, 0.22),
                area.hold.max(0.5),
            );
        }
        Impact::Roots => {
            roots_ring(
                commands,
                a,
                ground,
                &a.root,
                5,
                (0.36, 0.8),
                area.hold.max(0.6),
            );
            roots_ring(
                commands,
                a,
                ground,
                &a.leaf,
                4,
                (0.3, 0.55),
                area.hold.max(0.6),
            );
            dust(commands, a, ground, 4, 0.14);
        }
        Impact::Strangle => {
            roots_ring(
                commands,
                a,
                ground,
                &a.root,
                7,
                (0.4, 1.0),
                area.hold.max(0.6),
            );
            roots_ring(
                commands,
                a,
                ground,
                &a.amber,
                4,
                (0.26, 0.6),
                area.hold.max(0.6),
            );
            dust(commands, a, ground, 5, 0.14);
        }
        Impact::Spores => {
            roots_ring(
                commands,
                a,
                ground,
                &a.root,
                5,
                (0.36, 0.8),
                area.hold.max(0.6),
            );
            let r = area.splash.max(0.8);
            spawn(
                commands,
                &a.ring,
                &a.spore,
                ground,
                Motion::Grow { radius: r },
                0.0,
                0.45,
            );
            for (i, dir) in around(7, 0.5).enumerate() {
                let vel = dir * r * 0.9 + Vec3::Y * (0.3 + 0.1 * i as f32 % 0.3);
                spawn(
                    commands,
                    &a.puff,
                    &a.spore,
                    at.with_y(0.3),
                    Motion::Puff { vel, size: 0.07 },
                    0.0,
                    0.9,
                );
            }
        }
        Impact::Cleave => slam(commands, a, ground, 0.75, 4),
        Impact::Butcher => {
            slam(commands, a, ground, 0.8, 6);
            spawn(
                commands,
                &a.puff,
                &a.flash,
                at.with_y(0.35),
                Motion::Flash { size: 0.2 },
                0.0,
                0.1,
            );
        }
        Impact::Crush => {
            slam(commands, a, ground, 0.95, 6);
            spawn(
                commands,
                &a.ring,
                &a.green,
                ground,
                Motion::Grow { radius: 0.9 },
                0.04,
                0.4,
            );
        }
        Impact::Reap => {
            let r = area.splash.max(1.0);
            spawn(
                commands,
                &a.ring,
                &a.green,
                ground,
                Motion::Grow { radius: r },
                0.0,
                0.4,
            );
            slam(commands, a, ground, r * 0.6, 5);
        }
        Impact::Cannon => cannon(commands, a, at, 1.0),
        Impact::GreatCannon => cannon(commands, a, at, 1.5),
        Impact::Bolt => {
            dust(commands, a, ground, 3, 0.09);
            for dir in around(3, 1.0) {
                let vel = dir * 0.8 + Vec3::Y * 1.4;
                spawn(
                    commands,
                    &a.chip,
                    &a.root,
                    at,
                    Motion::Debris { vel, size: 0.035 },
                    0.0,
                    0.45,
                );
            }
        }
    }
}

/// Fire core, white flash, rolling smoke, thrown rocks and a ground ring.
fn blast(commands: &mut Commands, a: &VfxAssets, at: Vec3, radius: f32, size: f32) {
    spawn(
        commands,
        &a.puff,
        &a.fire,
        at,
        Motion::Flash { size: 0.38 * size },
        0.0,
        0.2,
    );
    spawn(
        commands,
        &a.puff,
        &a.flash,
        at,
        Motion::Flash { size: 0.24 * size },
        0.0,
        0.12,
    );
    for (i, dir) in around(6, 0.4).enumerate() {
        let mat = if i % 2 == 0 { &a.soot } else { &a.smoke };
        let vel = dir * 0.7 * radius + Vec3::Y * (0.8 + 0.15 * i as f32);
        spawn(
            commands,
            &a.puff,
            mat,
            at,
            Motion::Puff {
                vel,
                size: 0.2 * size,
            },
            0.02,
            0.75,
        );
    }
    for dir in around(5, 1.1) {
        let vel = dir * 1.4 * radius + Vec3::Y * 2.2;
        spawn(
            commands,
            &a.chip,
            &a.rock,
            at,
            Motion::Debris {
                vel,
                size: 0.06 * size,
            },
            0.0,
            0.6,
        );
    }
    spawn(
        commands,
        &a.ring,
        &a.dust,
        at.with_y(0.05),
        Motion::Grow { radius },
        0.0,
        0.35,
    );
}

/// A heavy blow on the ground: dust ring, dust puffs, bone and rock chips.
fn slam(commands: &mut Commands, a: &VfxAssets, ground: Vec3, radius: f32, chips: usize) {
    spawn(
        commands,
        &a.ring,
        &a.dust,
        ground,
        Motion::Grow { radius },
        0.0,
        0.3,
    );
    dust(commands, a, ground, 5, 0.16);
    for (i, dir) in around(chips, 0.7).enumerate() {
        let mat = if i % 2 == 0 { &a.bone } else { &a.rock };
        let vel = dir * 1.2 + Vec3::Y * (1.8 + 0.2 * (i % 3) as f32);
        spawn(
            commands,
            &a.chip,
            mat,
            ground,
            Motion::Debris { vel, size: 0.05 },
            0.0,
            0.55,
        );
    }
}

fn cannon(commands: &mut Commands, a: &VfxAssets, at: Vec3, size: f32) {
    spawn(
        commands,
        &a.puff,
        &a.fire,
        at,
        Motion::Flash { size: 0.26 * size },
        0.0,
        0.14,
    );
    dust(commands, a, at.with_y(0.05), 5, 0.18 * size);
    for dir in around(4, 0.2) {
        let vel = dir * 1.1 + Vec3::Y * 2.0;
        spawn(
            commands,
            &a.chip,
            &a.rock,
            at,
            Motion::Debris {
                vel,
                size: 0.05 * size,
            },
            0.0,
            0.55,
        );
    }
    spawn(
        commands,
        &a.ring,
        &a.dust,
        at.with_y(0.05),
        Motion::Grow { radius: 0.6 * size },
        0.0,
        0.3,
    );
}

fn dust(commands: &mut Commands, a: &VfxAssets, ground: Vec3, n: usize, size: f32) {
    for (i, dir) in around(n, 0.2).enumerate() {
        let vel = dir * 0.9 + Vec3::Y * (0.25 + 0.05 * i as f32);
        spawn(
            commands,
            &a.puff,
            &a.dust,
            ground + dir * 0.1,
            Motion::Puff { vel, size },
            0.0,
            0.6,
        );
    }
}

/// Spikes bursting out of the ground around a point, leaning inward.
fn roots_ring(
    commands: &mut Commands,
    a: &VfxAssets,
    ground: Vec3,
    mat: &Handle<StandardMaterial>,
    n: usize,
    (radius, height): (f32, f32),
    hold: f32,
) {
    for (i, dir) in around(n, 0.35).enumerate() {
        let tilt = Quat::from_rotation_arc(Vec3::Y, (Vec3::Y * 1.6 - dir).normalize());
        let h = height * (0.8 + 0.25 * ((i * 7 % 5) as f32 / 4.0));
        spawn_turned(
            commands,
            &a.spike,
            mat,
            (ground.with_y(0.0) + dir * radius, tilt),
            Motion::Sprout {
                size: Vec3::new(0.17, h, 0.17),
                hold,
            },
            (0.01 * i as f32, 0.29 + hold),
        );
    }
}

/// Flash, smoke and (for shoulder guns) a backblast behind `back`.
pub fn muzzle(
    commands: &mut Commands,
    a: &VfxAssets,
    kind: Muzzle,
    at: Vec3,
    dir: Vec3,
    back: Option<Vec3>,
) {
    let size = match kind {
        Muzzle::Bombard | Muzzle::Cannon => 1.0,
        Muzzle::Blunderbuss => 1.25,
        Muzzle::GreatBombard | Muzzle::GreatCannon => 1.5,
        Muzzle::Bolt => {
            spawn(
                commands,
                &a.puff,
                &a.dust,
                at,
                Motion::Puff {
                    vel: dir * 0.6,
                    size: 0.06,
                },
                0.0,
                0.3,
            );
            return;
        }
    };
    spawn(
        commands,
        &a.puff,
        &a.fire,
        at + dir * 0.08,
        Motion::Flash { size: 0.2 * size },
        0.0,
        0.1,
    );
    spawn(
        commands,
        &a.puff,
        &a.flash,
        at + dir * 0.05,
        Motion::Flash { size: 0.12 * size },
        0.0,
        0.07,
    );
    for i in 0..3 {
        let f = i as f32;
        let vel = dir * (1.6 - 0.4 * f) + Vec3::Y * (0.3 + 0.15 * f);
        spawn(
            commands,
            &a.puff,
            &a.smoke,
            at,
            Motion::Puff {
                vel,
                size: (0.11 + 0.03 * f) * size,
            },
            0.01 * f,
            0.6,
        );
    }
    if kind == Muzzle::Blunderbuss {
        for side in [-1.0, 1.0] {
            let fan = (dir + dir.cross(Vec3::Y) * side * 0.6).normalize();
            spawn(
                commands,
                &a.ball,
                &a.orange,
                at,
                Motion::Burst { dir: fan * 0.6 },
                0.0,
                0.14,
            );
        }
    }
    if let Some(back) = back {
        for i in 0..2 {
            let vel = -dir * (1.2 + 0.5 * i as f32) + Vec3::Y * 0.2;
            spawn(
                commands,
                &a.puff,
                &a.smoke,
                back,
                Motion::Puff {
                    vel,
                    size: 0.12 * size,
                },
                0.0,
                0.55,
            );
        }
        spawn(
            commands,
            &a.puff,
            &a.fire,
            back,
            Motion::Flash { size: 0.1 * size },
            0.0,
            0.08,
        );
    }
}

/// A kit's own Evolution burst, on top of the shared gold one.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Burst {
    /// Machines: sparks, a fire ring and steam.
    Forge,
    /// Undead: a green soul flare and rising wisps.
    Soul,
    /// Plants: a ring of sprouting roots and a shower of leaves.
    Bloom,
}

pub fn evolution(commands: &mut Commands, a: &VfxAssets, burst: Burst, at: Vec3) {
    let ground = at.with_y(0.05);
    match burst {
        Burst::Forge => {
            spawn(
                commands,
                &a.ring,
                &a.fire,
                ground,
                Motion::Grow { radius: 1.2 },
                0.0,
                0.45,
            );
            spawn(
                commands,
                &a.puff,
                &a.fire,
                at,
                Motion::Flash { size: 0.45 },
                0.0,
                0.2,
            );
            for (i, dir) in around(10, 0.0).enumerate() {
                let up = dir * 0.7 + Vec3::Y * (1.2 + 0.1 * (i % 3) as f32);
                spawn(
                    commands,
                    &a.ball,
                    &a.orange,
                    at,
                    Motion::Burst { dir: up },
                    0.0,
                    0.45,
                );
            }
            for dir in around(5, 0.5) {
                let vel = dir * 0.5 + Vec3::Y * 0.9;
                spawn(
                    commands,
                    &a.puff,
                    &a.smoke,
                    at,
                    Motion::Puff { vel, size: 0.2 },
                    0.05,
                    0.8,
                );
            }
        }
        Burst::Soul => {
            spawn(
                commands,
                &a.ring,
                &a.green,
                ground,
                Motion::Grow { radius: 1.3 },
                0.0,
                0.5,
            );
            spawn(
                commands,
                &a.puff,
                &a.green,
                at,
                Motion::Flash { size: 0.5 },
                0.0,
                0.22,
            );
            for (i, dir) in around(8, 0.3).enumerate() {
                let vel = dir * 0.35 + Vec3::Y * (1.0 + 0.15 * (i % 2) as f32);
                spawn(
                    commands,
                    &a.puff,
                    &a.green,
                    at + dir * 0.3,
                    Motion::Puff { vel, size: 0.09 },
                    0.02 * i as f32,
                    0.9,
                );
            }
        }
        Burst::Bloom => {
            roots_ring(commands, a, ground, &a.root, 7, (0.55, 0.6), 0.35);
            roots_ring(commands, a, ground, &a.leaf, 5, (0.75, 0.35), 0.3);
            for (i, dir) in around(10, 0.2).enumerate() {
                let vel = dir * 1.1 + Vec3::Y * (2.4 + 0.2 * (i % 3) as f32);
                spawn(
                    commands,
                    &a.chip,
                    &a.leaf,
                    at,
                    Motion::Debris { vel, size: 0.06 },
                    0.0,
                    0.8,
                );
            }
            for dir in around(5, 0.9) {
                spawn(
                    commands,
                    &a.puff,
                    &a.spore,
                    at,
                    Motion::Puff {
                        vel: dir * 0.5 + Vec3::Y * 0.6,
                        size: 0.06,
                    },
                    0.1,
                    0.8,
                );
            }
        }
    }
}

/// A heavy footfall: a low dust ring and a few puffs.
pub fn stomp(commands: &mut Commands, a: &VfxAssets, foot: Vec3, size: f32) {
    let ground = foot.with_y(0.04);
    spawn(
        commands,
        &a.ring,
        &a.dust,
        ground,
        Motion::Grow { radius: 0.3 * size },
        0.0,
        0.3,
    );
    for dir in around(3, foot.x * 3.0) {
        let vel = dir * 0.5 + Vec3::Y * 0.12;
        spawn(
            commands,
            &a.puff,
            &a.dust,
            ground,
            Motion::Puff {
                vel,
                size: 0.07 * size,
            },
            0.0,
            0.45,
        );
    }
}

/// A fast crescent following a wide swing (scythe, cleaver sweep).
pub fn sweep(
    commands: &mut Commands,
    a: &VfxAssets,
    center: Vec3,
    facing: f32,
    radius: f32,
    glow: bool,
) {
    let mat = if glow { &a.green } else { &a.white };
    let start = Quat::from_rotation_y(facing + 1.2);
    spawn_turned(
        commands,
        &a.crescent,
        mat,
        (center, start),
        Motion::Sweep {
            size: radius,
            turn: -2.4,
        },
        (0.0, 0.22),
    );
}
