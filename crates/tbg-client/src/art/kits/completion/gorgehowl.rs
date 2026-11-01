//! Authored five-form low-poly figure; proportions and palette follow the shared rig-v2 contract.
use super::forms::*;
pub fn spec(t: u8, b: u8) -> FigureSpec {
    let p = plan(t);
    let f = Form::of(t, b);

    let mut v = humanoid(&p, CHARCOAL, CHARCOAL, GUNMETAL, CHARCOAL);
    v.push(part(
        Body(Head),
        Bevel(0.34, 0.15, 0.25, 0.04),
        GUNMETAL,
        [0.0, -0.025, 0.04],
    ));
    v.push(part(
        Body(Head),
        Box(0.21, 0.065, 0.03),
        DARK,
        [0.0, -0.08, 0.18],
    ));
    for s in [-1.0, 1.0] {
        v.push(
            part(
                Secondary(Spring::HeadGear),
                Ring(0.027, if f.a { 0.24 } else { 0.17 }, 220.0),
                IVORY,
                [s * 0.20, 0.13, -0.02],
            )
            .rot([0.0, 0.0, s * -55.0])
            .scale([0.7, 1.2, 0.65]),
        );
        v.push(
            part(
                Secondary(Spring::HeadGear),
                Cone(0.07, if f.b { 0.31 } else { 0.18 }),
                TURQUOISE,
                [s * 0.22, 0.18, -0.045],
            )
            .rot([0.0, 0.0, s * -18.0])
            .scale([0.4, 1.0, 0.6]),
        );
        v.push(part(Body(Head), Ball(0.025), TURQUOISE, [s * 0.12, 0.04, 0.15]).glow());
    }
    for i in 0..4 {
        v.push(
            part(
                Body(Chest),
                Bevel(0.24 - i as f32 * 0.026, 0.045, 0.035, 0.01),
                if f.b { TURQUOISE } else { IVORY },
                [0.0, 0.28 - i as f32 * 0.06, 0.135],
            )
            .rot([-12.0, 0.0, 0.0]),
        );
    }
    if f.t2 {
        for s in [-1.0, 1.0] {
            v.push(
                part(
                    Body(Chest),
                    Cone(0.065, 0.18),
                    IVORY,
                    [s * 0.14, 0.28, -0.08],
                )
                .rot([-30.0, 0.0, s * -25.0]),
            );
        }
    }
    team(&mut v, Body(Hips), 0.10);
    FigureSpec { plan: p, parts: v }
}
pub fn plan(t: u8) -> Proportions {
    let p = body::plan(false, 1.0, t);
    p
}
