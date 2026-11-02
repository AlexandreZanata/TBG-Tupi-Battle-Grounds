//! Authored five-form low-poly figure; proportions and palette follow the shared rig-v2 contract.
use super::forms::*;
pub fn spec(t: u8, b: u8) -> FigureSpec {
    let p = plan(t);
    let f = Form::of(t, b);

    let mut v = insect(&p, f, 0.09);
    antennae(&mut v, 0.09, false);
    for s in [-1.0, 1.0] {
        v.push(
            part(
                Body(Head),
                Ball(0.10),
                if f.b { TURQUOISE } else { AMBER },
                [s * 0.11, 0.03, 0.045],
            )
            .scale([0.7, 1.0, 0.8]),
        );
        for j in 0..2 {
            v.push(
                part(
                    Prop(Socket::WeaponR),
                    Bevel(0.07, if f.b { 0.48 } else { 0.38 }, 0.018, 0.01),
                    if f.b { SKY } else { LINEN },
                    [s * (0.13 + j as f32 * 0.07), 0.17, -0.15],
                )
                .rot([-28.0, 0.0, s * (-10.0 - j as f32 * 13.0)]),
            );
        }
    }
    for i in 0..4 {
        v.push(
            part(
                Body(Hips),
                Frustum(0.07 - i as f32 * 0.01, 0.06 - i as f32 * 0.01, 0.13),
                if i % 2 == 0 { MOSS } else { AMBER },
                [0.0, 0.02, -0.14 - i as f32 * 0.105],
            )
            .rot([-80.0, 0.0, 0.0]),
        );
    }
    let w = Body(HandR);
    v.push(part(w, Frustum(0.065, 0.045, 0.27), CHITIN, [0.0, 0.0, 0.11]).rot([90.0, 0.0, 0.0]));
    needle(
        &mut v,
        Secondary(Spring::HeadGear),
        if f.a { 0.55 } else { 0.40 },
        IVORY,
    );
    if f.a {
        v.push(part(w, Box(0.12, 0.04, 0.30), AMBER, [0.0, 0.0, 0.08]));
    }
    if f.t2 {
        v.push(
            part(Body(Head), Cone(0.035, 0.18), IVORY, [0.0, 0.16, -0.01]).rot([-25.0, 0.0, 0.0]),
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
