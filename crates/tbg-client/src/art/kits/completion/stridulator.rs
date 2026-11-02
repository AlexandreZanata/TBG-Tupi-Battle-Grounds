//! Authored five-form low-poly figure; proportions and palette follow the shared rig-v2 contract.
use super::forms::*;
pub fn spec(t: u8, b: u8) -> FigureSpec {
    let p = plan(t);
    let f = Form::of(t, b);

    let mut v = insect(&p, f, 0.10);
    antennae(&mut v, 0.21, false);
    for s in [-1.0, 1.0] {
        v.push(
            part(
                Secondary(Spring::HeadGear),
                Bevel(0.15, if f.b { 0.43 } else { 0.35 }, 0.025, 0.025),
                MOSS,
                [s * 0.08, 0.12, -0.15],
            )
            .rot([-15.0, 0.0, s * -9.0]),
        );
    }
    for i in 0..5 {
        v.push(part(
            Body(Chest),
            Bevel(0.22 - i as f32 * 0.017, 0.028, 0.05, 0.008),
            if f.a { IVORY } else { AMBER },
            [0.0, 0.28 - i as f32 * 0.044, 0.14],
        ));
    }
    if f.b {
        for s in [-1.0, 1.0] {
            v.push(
                part(
                    Body(Head),
                    Ring(0.014, 0.09, 220.0),
                    AMBER,
                    [s * 0.12, 0.03, 0.0],
                )
                .rot([0.0, 0.0, s * 45.0]),
            );
        }
    }
    if f.t2 {
        v.push(
            part(
                Body(Chest),
                Ring(0.016, 0.13, 300.0),
                if f.a { IVORY } else { SAP },
                [0.0, 0.17, 0.17],
            )
            .rot([90.0, 0.0, 0.0]),
        );
    }
    FigureSpec { plan: p, parts: v }
}
pub fn plan(t: u8) -> Proportions {
    let mut p = body::plan(false, 1.0, t);
    p.torso.x *= 0.83;
    p.arm *= 0.95;
    p
}
