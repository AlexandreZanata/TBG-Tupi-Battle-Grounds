//! Authored five-form low-poly figure; proportions and palette follow the shared rig-v2 contract.
use super::forms::*;
pub fn spec(t: u8, b: u8) -> FigureSpec {
    let p = plan(t);
    let f = Form::of(t, b);

    let mut v = humanoid(&p, SKIN, NAVY, IVORY, GOLD);
    body::cape(&mut v, NAVY, 0.28, 0.25);
    v.push(part(
        Body(Head),
        Box(0.22, 0.035, 0.035),
        GOLD,
        [0.0, 0.06, 0.12],
    ));
    v.push(part(Body(Head), Cone(0.06, 0.11), IVORY, [0.0, 0.10, 0.13]).scale([1.0, 1.0, 0.3]));
    let w = Prop(Socket::WeaponR);
    for s in [-1.0, 1.0] {
        v.push(
            part(w, Box(0.025, 0.025, 0.27), GOLD, [s * 0.12, 0.0, 0.10]).rot([
                0.0,
                s * -12.0,
                0.0,
            ]),
        );
    }
    v.push(
        part(
            w,
            Prism(if f.a { 0.15 } else { 0.11 }, 0.26),
            if f.b { SKY } else { IVORY },
            [0.0, 0.0, 0.20],
        )
        .rot([90.0, 0.0, 0.0])
        .scale([1.0, 1.0, 0.5])
        .glow(),
    );
    if f.b {
        v.push(
            part(
                w,
                Ring(0.018, if f.t2 { 0.20 } else { 0.15 }, 300.0),
                GOLD,
                [0.0, 0.0, 0.15],
            )
            .rot([90.0, 0.0, 0.0]),
        );
    }
    if f.a {
        v.push(part(
            Body(ForearmR),
            Bevel(0.16, 0.18, 0.17, 0.02),
            IVORY,
            [0.0, -0.07, 0.0],
        ));
    }
    if f.t2 {
        v.push(part(Body(Chest), Cone(0.11, 0.16), GOLD, [0.0, 0.21, 0.14]).scale([1.0, 1.0, 0.3]));
    }

    FigureSpec { plan: p, parts: v }
}
pub fn plan(t: u8) -> Proportions {
    body::plan(false, 1.0, t)
}
