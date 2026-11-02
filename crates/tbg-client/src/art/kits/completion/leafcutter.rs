//! Authored five-form low-poly figure; proportions and palette follow the shared rig-v2 contract.
use super::forms::*;
pub fn spec(t: u8, b: u8) -> FigureSpec {
    let p = plan(t);
    let f = Form::of(t, b);

    let mut v = insect(&p, f, 0.13);
    antennae(&mut v, 0.22, true);
    v.push(part(Body(Head), Ball(0.12), MOSS, [0.0, -0.045, 0.08]).scale([1.2, 0.8, 1.05]));
    for s in [-1.0, 1.0] {
        v.push(
            part(Body(Head), Cone(0.04, 0.12), AMBER, [s * 0.09, -0.07, 0.15]).rot([
                70.0,
                0.0,
                s * 25.0,
            ]),
        );
    }
    let w = Prop(Socket::WeaponR);
    blade(&mut v, w, if f.a { 0.54 } else { 0.42 }, IVORY, false);
    for i in 0..4 {
        v.push(
            part(
                w,
                Cone(0.045, 0.09),
                if f.b { AMBER } else { IVORY },
                [0.06, 0.0, 0.13 + i as f32 * 0.09],
            )
            .rot([0.0, 0.0, -65.0]),
        );
    }
    if f.a {
        v.push(part(
            Body(UpperArmR),
            Bevel(0.16, 0.08, 0.14, 0.02),
            IVORY,
            [0.0, 0.0, 0.0],
        ));
    }
    if f.t2 {
        v.push(
            part(Body(Head), Box(0.04, 0.25, 0.10), AMBER, [0.0, 0.15, -0.02])
                .rot([-25.0, 0.0, 0.0]),
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
