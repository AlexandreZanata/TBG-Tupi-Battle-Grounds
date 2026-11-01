//! Hushfang's four-legged saber-cat geometry, fixed anatomy and grounded crouch gait.
use super::super::{tools, Moment};
use super::forms::*;
use crate::art::{
    anim::{Clip, Pose},
    ik,
};
pub fn plan(t: u8) -> Proportions {
    let mut p = body::plan(false, 1.0, t);
    p.leg = 0.36;
    p.arm = 0.18;
    p
}
pub fn rest(n: Node, p: &Proportions) -> Vec3 {
    match n {
        Body(Hips) => Vec3::new(0.0, 0.385, -0.24),
        Body(Spine) => Vec3::new(0.0, 0.0, 0.12),
        Body(Chest) => Vec3::new(0.0, 0.015, 0.27),
        Body(Head) => Vec3::new(0.0, 0.07, 0.24),
        Body(UpperArmL) => Vec3::new(0.14, -0.01, 0.03),
        Body(UpperArmR) => Vec3::new(-0.14, -0.01, 0.03),
        Body(ThighL) => Vec3::new(0.14, 0.005, -0.04),
        Body(ThighR) => Vec3::new(-0.14, 0.005, -0.04),
        Body(ForearmL | ForearmR | ShinL | ShinR) => Vec3::NEG_Y * 0.18,
        Body(HandL | HandR | FootL | FootR) => Vec3::NEG_Y * 0.18,
        Secondary(Spring::Back) => Vec3::new(0.0, 0.02, -0.10),
        _ => rig::rest(n, p),
    }
}
pub fn spec(t: u8, b: u8) -> FigureSpec {
    let p = plan(t);
    let f = Form::of(t, b);
    let mut v = vec![
        part(Body(Hips), Ball(0.17), CHARCOAL, [0.0, 0.0, 0.0]).scale([0.95, 0.8, 1.3]),
        part(Body(Spine), Ball(0.18), CHARCOAL, [0.0, 0.0, 0.06]).scale([0.9, 0.8, 1.5]),
        part(
            Body(Chest),
            Ball(0.19),
            if f.b { DARK } else { CHARCOAL },
            [0.0, 0.0, 0.03],
        )
        .scale([0.9, 1.0, 1.2]),
        part(Body(Head), Ball(0.15), GUNMETAL, [0.0, 0.0, 0.03]).scale([1.1, 0.85, 1.2]),
        part(
            Body(Head),
            Bevel(0.18, 0.10, 0.19, 0.025),
            CHARCOAL,
            [0.0, -0.055, 0.15],
        ),
        part(
            Secondary(Spring::Back),
            Cone(0.055, 0.49),
            CHARCOAL,
            [0.0, 0.0, -0.23],
        )
        .rot([-95.0, 0.0, 0.0]),
    ];
    for s in [-1.0, 1.0] {
        v.push(
            part(
                Body(Head),
                Cone(0.06, 0.19),
                CHARCOAL,
                [s * 0.13, 0.15, -0.04],
            )
            .rot([-35.0, 0.0, s * -18.0]),
        );
        v.push(
            part(
                Body(Head),
                Cone(
                    if f.a { 0.047 } else { 0.035 },
                    if f.a { 0.25 } else { 0.18 },
                ),
                IVORY,
                [s * 0.09, -0.13, 0.20],
            )
            .rot([170.0, 0.0, 0.0]),
        );
        v.push(
            part(
                Body(Head),
                Box(0.04, 0.024, 0.025),
                TURQUOISE,
                [s * 0.12, 0.015, 0.15],
            )
            .glow(),
        );
        if f.b {
            v.push(
                part(
                    Body(Chest),
                    Box(0.025, 0.15, 0.16),
                    TURQUOISE,
                    [s * 0.16, 0.02, 0.04],
                )
                .rot([0.0, 0.0, s * -20.0]),
            );
        }
    }
    for (u, l, foot) in [
        (UpperArmL, ForearmL, HandL),
        (UpperArmR, ForearmR, HandR),
        (ThighL, ShinL, FootL),
        (ThighR, ShinR, FootR),
    ] {
        v.push(part(
            Body(u),
            Frustum(0.065, 0.045, 0.18),
            CHARCOAL,
            [0.0, -0.09, 0.0],
        ));
        v.push(part(Body(l), Cyl(0.035, 0.18), GUNMETAL, [0.0, -0.09, 0.0]));
        v.push(part(
            Body(foot),
            Bevel(0.11, 0.06, 0.16, 0.02),
            CHARCOAL,
            [0.0, -0.025, 0.04],
        ));
        for x in [-0.03, 0.03] {
            v.push(
                part(Body(foot), Cone(0.014, 0.06), IVORY, [x, -0.02, 0.13]).rot([90.0, 0.0, 0.0]),
            );
        }
    }
    if f.t2 {
        for i in 0..3 {
            v.push(
                part(
                    Body(Spine),
                    Cone(0.035, 0.12),
                    if f.a { IVORY } else { TURQUOISE },
                    [0.0, 0.15, -0.12 + i as f32 * 0.12],
                )
                .rot([-25.0, 0.0, 0.0]),
            );
        }
    }
    team(&mut v, Body(Chest), 0.07);
    FigureSpec { plan: p, parts: v }
}
pub fn pose(m: &Moment, p: &mut Pose) {
    p.fill((Vec3::ZERO, Vec3::ZERO));
    let breath = (m.clock * 1.7).sin();
    tools::shift(p, Chest, [0.0, breath * 0.006, 0.0]);
    match m.clip {
        Clip::Walk => walk(m, p),
        Clip::AttackMelee | Clip::AttackRanged | Clip::Cast => {
            let coil = tools::hold(m.t, [0.0, 0.16, 0.20, 0.30]);
            let hit = tools::hold(m.t, [0.20, 0.30, 0.35, 0.60]);
            tools::shift(p, Chest, [0.0, -0.04 * coil, 0.08 * hit]);
            tools::set(p, Head, [-16.0 * hit, 0.0, 0.0]);
            tools::set(p, ForearmL, [-12.0 * hit, 0.0, 0.0]);
        }
        Clip::Victory => {
            let t = m.t * std::f32::consts::TAU / 3.0;
            tools::set(p, Head, [4.0 * t.sin(), 12.0 * (t * 2.0).sin(), 0.0]);
            tools::shift(p, Chest, [0.0, 0.025 * t.sin().max(0.0), 0.0]);
            tools::set(p, HandL, [-12.0 * (t * 2.0).sin().max(0.0), 0.0, 0.0]);
            tools::set(p, HandR, [-12.0 * (-t * 2.0).sin().max(0.0), 0.0, 0.0]);
        }
        Clip::Death => {
            let k = tools::window(m.t, 0.0, 0.75);
            tools::shift(p, Hips, [0.0, -0.27 * k, 0.0]);
            tools::set(p, Chest, [0.0, 0.0, 35.0 * k]);
            tools::set(p, Head, [28.0 * k, 0.0, 0.0]);
            for j in [ForearmL, ForearmR, ShinL, ShinR] {
                tools::set(p, j, [-75.0 * k, 0.0, 0.0]);
            }
        }
        Clip::Spawn | Clip::Evolve => {
            let k = 1.0 - tools::window(m.t, 0.0, 0.5);
            tools::shift(p, Hips, [0.0, -0.18 * k, 0.0]);
            tools::set(p, Head, [20.0 * k, 0.0, 0.0]);
        }
        Clip::Hit => tools::shift(
            p,
            Chest,
            [
                0.0,
                -0.02,
                -0.04 * tools::hold(m.t, [0.0, 0.04, 0.12, 0.25]),
            ],
        ),
        Clip::Idle => {}
    }
    p[Secondary(Spring::Back).index()].0.y = 8.0 * (m.clock * 1.2).sin();
    if m.clip != Clip::Death && m.clip != Clip::Walk {
        plant(m, p);
    }
}
fn walk(m: &Moment, p: &mut Pose) {
    tools::shift(p, Hips, [0.0, -0.04, 0.0]);
    plant(m, p);
}
fn plant(m: &Moment, p: &mut Pose) {
    let body = plan(m.tier);
    let hips = frame(p, &body, &[Root, Hips]);
    let chest = frame(p, &body, &[Root, Hips, Spine, Chest]);
    let stride = super::super::Gait::LIGHT.cycle(body.leg);
    for (u, l, foot, side, offset, front) in [
        (UpperArmL, ForearmL, HandL, 1.0, 0.0, true),
        (UpperArmR, ForearmR, HandR, -1.0, 0.5, true),
        (ThighL, ShinL, FootL, 1.0, 0.5, false),
        (ThighR, ShinR, FootR, -1.0, 0.0, false),
    ] {
        let phase = (m.walk.unwrap_or(0.0) + offset).rem_euclid(1.0);
        let swing = ((phase - 0.5) * 2.0).clamp(0.0, 1.0);
        let (along, lift) = if m.clip == Clip::Walk {
            if phase < 0.5 {
                (stride * (0.25 - phase), 0.0)
            } else {
                (
                    stride * (-0.25 + 0.5 * swing * swing * (3.0 - 2.0 * swing)),
                    (swing * std::f32::consts::PI).sin() * 0.06,
                )
            }
        } else {
            (0.0, 0.0)
        };
        let parent = if front { chest } else { hips };
        let start = rest(Body(u), &body);
        let target = parent.inverse().transform_point3(Vec3::new(
            side * 0.14,
            0.03 + lift,
            if front { 0.18 + along } else { -0.28 + along },
        ));
        let (a, b) = ik::two_bone(
            start,
            target,
            0.18,
            0.18,
            if front { Vec3::Z } else { Vec3::NEG_Z },
        );
        let (_, rotation, _) = parent.to_scale_rotation_translation();
        p[u.index()].0 = ik::degrees(a);
        p[l.index()].0 = ik::degrees(b);
        let toe = if m.clip == Clip::Victory {
            ik::quat(p[foot.index()].0)
        } else {
            Quat::IDENTITY
        };
        p[foot.index()].0 = ik::degrees((rotation * a * b).inverse() * toe);
    }
}

fn frame(p: &Pose, b: &Proportions, chain: &[rig::Joint]) -> Mat4 {
    chain.iter().fold(Mat4::IDENTITY, |frame, j| {
        let (r, o) = p[j.index()];
        frame * Mat4::from_rotation_translation(ik::quat(r), rest(Body(*j), b) + o)
    })
}
