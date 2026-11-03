//! Compact champion-specific cosmetic hit recipes; all targets and outcomes come from the simulation.
use super::impacts::Area;
use super::{spawn, spawn_turned, Motion, VfxAssets};
use bevy::prelude::*;
use tbg_core::UnitKind::{self, *};
pub fn impact(c: &mut Commands, a: &VfxAssets, k: UnitKind, t: u8, b: u8, at: Vec3, area: Area) {
    let power = 1.0 + t.min(2) as f32 * 0.12;
    let mat = match k {
        Gravecutter | LastlightCantor => &a.green,
        Cragclaw | Hushfang | Spinehurler | MireTender | Gorgehowl => &a.turquoise,
        Sunlancer | EclipseDancer | Starbreaker | Dawnmender | Prismancer => {
            if b == 1 {
                &a.white
            } else {
                &a.gold
            }
        }
        NacreDuelist | ReefLancer | TrenchFist | PearlSlinger | AbyssHarpooner | UndertowHerald
        | TrenchLamplighter | FoamWeaver => {
            if b == 1 {
                &a.violet
            } else {
                &a.turquoise
            }
        }
        Colossus => {
            if b == 0 {
                &a.gold
            } else {
                &a.amber
            }
        }
        PodSnapper => &a.green,
        // GD-04: storm cyan; the rupture lance is gold, crimson on the B branch.
        Volthar => &a.water,
        Astreon => {
            if b == 1 {
                &a.blood
            } else {
                &a.gold
            }
        }
        _ => &a.amber,
    };
    match k {
        Volthar => {
            chips(c, a, &a.white, at, 5, 0.12 * power);
            spawn(
                c,
                &a.ball,
                mat,
                at,
                Motion::Flash { size: 0.22 * power },
                0.0,
                0.14,
            );
            spawn(
                c,
                &a.ring,
                mat,
                at.with_y(0.05),
                Motion::Grow {
                    radius: 0.45 * power,
                },
                0.02,
                0.22,
            );
        }
        Astreon => {
            // A narrow piercing streak and flash on the struck unit only.
            spawn(
                c,
                &a.chip,
                mat,
                at,
                Motion::Segment {
                    size: Vec3::new(0.03, 0.03, 0.42 * power),
                },
                0.0,
                0.16,
            );
            spawn(
                c,
                &a.ball,
                &a.white,
                at,
                Motion::Flash { size: 0.16 * power },
                0.0,
                0.10,
            );
        }
        Colossus => {
            chips(c, a, &a.bone, at, 8, 0.18 * power);
            for i in 0..2 {
                spawn(
                    c,
                    &a.ring,
                    mat,
                    at.with_y(0.04),
                    Motion::Grow {
                        radius: area.splash.max(1.8) * (0.8 + i as f32 * 0.2),
                    },
                    i as f32 * 0.07,
                    0.34,
                );
            }
        }
        NacreDuelist | ReefLancer => {
            spawn(
                c,
                &a.crescent,
                mat,
                at,
                Motion::Sweep {
                    size: 0.24 * power,
                    turn: if k == ReefLancer { 0.45 } else { 1.1 },
                },
                0.0,
                0.17,
            );
        }
        TrenchFist => {
            chips(c, a, &a.bone, at, 4, 0.10);
            spawn(
                c,
                &a.ring,
                mat,
                at.with_y(0.04),
                Motion::Grow {
                    radius: 0.25 * power,
                },
                0.0,
                0.20,
            );
        }
        PearlSlinger => {
            chips(c, a, &a.white, at, 3, 0.065);
            spawn(c, &a.ball, mat, at, Motion::Flash { size: 0.08 }, 0.0, 0.10);
        }
        AbyssHarpooner => {
            spawn(
                c,
                &a.chip,
                &a.bone,
                at,
                Motion::Segment {
                    size: Vec3::new(0.02, 0.02, 0.20),
                },
                0.0,
                0.15,
            );
        }
        UndertowHerald => {
            for i in 0..2 {
                spawn(
                    c,
                    &a.ring,
                    mat,
                    at.with_y(0.05),
                    Motion::Grow {
                        radius: 0.22 + i as f32 * 0.10,
                    },
                    i as f32 * 0.05,
                    0.22,
                );
            }
        }
        TrenchLamplighter => {
            spawn(
                c,
                &a.ring,
                &a.amber,
                at.with_y(0.05),
                Motion::Grow {
                    radius: 0.28 * power,
                },
                0.0,
                0.22,
            );
            chips(c, a, &a.white, at, 3, 0.05);
        }
        FoamWeaver => {
            spawn(
                c,
                &a.ring,
                mat,
                at,
                Motion::Grow { radius: 0.19 },
                0.0,
                0.26,
            );
            spawn(c, &a.ball, &a.white, at, Motion::Rise, 0.05, 0.22);
        }
        PodSnapper => {
            chips(c, a, &a.green, at, 3, 0.045);
            spawn(
                c,
                &a.ball,
                &a.amber,
                at,
                Motion::Flash { size: 0.06 },
                0.0,
                0.10,
            );
        }
        Gravecutter | EclipseDancer | Leafcutter => spawn(
            c,
            &a.crescent,
            mat,
            at,
            Motion::Sweep {
                size: 0.28 * power,
                turn: if k == EclipseDancer { -1.1 } else { 0.9 },
            },
            0.0,
            0.18,
        ),
        LastlightCantor => {
            spawn(
                c,
                &a.ring,
                mat,
                at,
                Motion::Grow {
                    radius: 0.24 * power,
                },
                0.0,
                0.28,
            );
            spawn(c, &a.ball, &a.bone, at, Motion::Rise, 0.05, 0.24);
        }
        Cragclaw => chips(c, a, mat, at, 3, 0.12 * power),
        Hushfang => {
            for x in [-0.045, 0.045] {
                spawn(
                    c,
                    &a.chip,
                    &a.bone,
                    at + Vec3::X * x,
                    Motion::Segment {
                        size: Vec3::new(0.014, 0.12, 0.015),
                    },
                    0.0,
                    0.16,
                );
            }
        }
        Spinehurler | Needlewing => {
            spawn(
                c,
                &a.chip,
                &a.bone,
                at,
                Motion::Segment {
                    size: Vec3::new(0.018, 0.018, 0.18 * power),
                },
                0.0,
                0.13,
            );
            spawn(c, &a.ball, mat, at, Motion::Flash { size: 0.07 }, 0.0, 0.10);
        }
        MireTender => {
            for x in [-0.08, 0.0, 0.08] {
                spawn(c, &a.ball, mat, at + Vec3::X * x, Motion::Rise, 0.0, 0.32);
            }
        }
        Gorgehowl => {
            for i in 0..2 {
                spawn(
                    c,
                    &a.ring,
                    mat,
                    at,
                    Motion::Grow {
                        radius: 0.25 + i as f32 * 0.08,
                    },
                    i as f32 * 0.045,
                    0.18,
                );
            }
        }
        Sunlancer => {
            spawn(
                c,
                &a.chip,
                mat,
                at,
                Motion::Segment {
                    size: Vec3::new(0.025, 0.025, 0.28),
                },
                0.0,
                0.14,
            );
            spawn(
                c,
                &a.ring,
                &a.gold,
                at,
                Motion::Grow { radius: 0.15 },
                0.0,
                0.18,
            );
        }
        Starbreaker => {
            spawn(
                c,
                &a.ring,
                mat,
                at.with_y(0.04),
                Motion::Grow {
                    radius: 0.35 * power,
                },
                0.0,
                0.24,
            );
            chips(c, a, &a.white, at, 4, 0.10);
        }
        Dawnmender => {
            spawn(
                c,
                &a.ring,
                mat,
                at,
                Motion::Grow { radius: 0.19 },
                0.0,
                0.28,
            );
            spawn(
                c,
                &a.ball,
                &a.white,
                at + Vec3::Y * 0.04,
                Motion::Rise,
                0.0,
                0.26,
            );
        }
        Prismancer => {
            for i in 0..3 {
                let angle = i as f32 * std::f32::consts::TAU / 3.0;
                let d = Vec3::new(angle.cos(), angle.sin(), 0.0);
                spawn(
                    c,
                    &a.chip,
                    mat,
                    at + d * 0.05,
                    Motion::Burst { dir: d * 0.20 },
                    0.0,
                    0.20,
                );
            }
        }
        Blastbeetle => {
            spawn(
                c,
                &a.ring,
                &a.amber,
                at.with_y(0.04),
                Motion::Grow {
                    radius: area.splash.max(0.22),
                },
                0.0,
                0.30,
            );
            chips(c, a, mat, at, 4, 0.09);
        }
        NectarNurse => {
            spawn(c, &a.ball, mat, at, Motion::Flash { size: 0.07 }, 0.0, 0.10);
            spawn(
                c,
                &a.ring,
                mat,
                at,
                Motion::Grow { radius: 0.12 },
                0.0,
                0.16,
            );
        }
        Stridulator => {
            for i in 0..3 {
                spawn_turned(
                    c,
                    &a.ring,
                    mat,
                    (at, Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)),
                    Motion::Grow {
                        radius: 0.16 + i as f32 * 0.06,
                    },
                    (i as f32 * 0.025, 0.15),
                );
            }
        }
        BasaltBulwark => chips(c, a, &a.white, at, 3, 0.10 * power),
        CinderMaul => {
            chips(c, a, &a.amber, at, 4, 0.06 * power);
            spawn(
                c,
                &a.ring,
                &a.amber,
                at,
                Motion::Grow {
                    radius: 0.18 * power,
                },
                0.0,
                0.18,
            );
        }
        HookBrawler => spawn(
            c,
            &a.crescent,
            &a.white,
            at,
            Motion::Sweep {
                size: 0.17 * power,
                turn: -0.7,
            },
            0.0,
            0.16,
        ),
        FlintRunner => {
            for x in [-0.035, 0.035] {
                spawn(
                    c,
                    &a.chip,
                    &a.white,
                    at + Vec3::X * x,
                    Motion::Segment {
                        size: Vec3::new(0.012, 0.12, 0.05),
                    },
                    0.0,
                    0.14,
                );
            }
        }
        ClayBinder => {
            chips(c, a, &a.amber, at, 3, 0.08);
            spawn(
                c,
                &a.ring,
                &a.amber,
                at.with_y(0.03),
                Motion::Grow {
                    radius: 0.23 * power,
                },
                0.0,
                0.26,
            );
        }
        GustDrummer => {
            spawn(
                c,
                &a.ring,
                &a.green,
                at.with_y(0.04),
                Motion::Grow {
                    radius: area.splash.max(0.20),
                },
                0.0,
                0.35,
            );
            chips(c, a, &a.root, at.with_y(0.15), 4, 0.08);
            for x in [-0.10, 0.0, 0.10] {
                spawn(
                    c,
                    &a.ball,
                    &a.green,
                    at + Vec3::X * x,
                    Motion::Rise,
                    0.0,
                    0.28,
                );
            }
        }
        HideMender => {
            for y in [0.0, 0.06] {
                spawn(
                    c,
                    &a.ring,
                    &a.gold,
                    at + Vec3::Y * y,
                    Motion::Grow { radius: 0.12 },
                    0.0,
                    0.25,
                );
            }
        }
        PebbleSlinger => chips(c, a, &a.white, at, 2, 0.055 * power),
        RivetBallista => {
            chips(c, a, &a.gold, at, 2, 0.035);
            spawn(
                c,
                &a.chip,
                &a.white,
                at,
                Motion::Segment {
                    size: Vec3::new(0.025, 0.025, 0.10),
                },
                0.0,
                0.12,
            );
        }
        _ => {}
    }
}
fn chips(
    c: &mut Commands,
    a: &VfxAssets,
    mat: &Handle<StandardMaterial>,
    at: Vec3,
    count: usize,
    size: f32,
) {
    for i in 0..count {
        let angle = i as f32 * std::f32::consts::TAU / count as f32;
        spawn(
            c,
            &a.chip,
            mat,
            at,
            Motion::Debris {
                vel: Vec3::new(angle.cos(), 1.2, angle.sin()) * 0.5,
                size,
            },
            0.0,
            0.28,
        );
    }
}

pub fn evolution(c: &mut Commands, a: &VfxAssets, k: UnitKind, at: Vec3) {
    impact(c, a, k, 2, 0, at, Area::default());
    let mat = match k {
        Gravecutter | LastlightCantor => &a.green,
        Cragclaw | Hushfang | Spinehurler | MireTender | Gorgehowl => &a.turquoise,
        Sunlancer | EclipseDancer | Starbreaker | Dawnmender | Prismancer => &a.gold,
        Volthar => &a.water,
        Astreon => &a.gold,
        _ => &a.amber,
    };
    for i in 0..3 {
        spawn(
            c,
            &a.ball,
            mat,
            at + Vec3::X * (i as f32 - 1.0) * 0.10,
            Motion::Rise,
            i as f32 * 0.04,
            0.38,
        );
    }
}
