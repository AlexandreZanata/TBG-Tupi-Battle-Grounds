//! Authored five-form low-poly figure; proportions and palette follow the shared rig-v2 contract.
use super::forms::*;
pub fn spec(t: u8, b: u8) -> FigureSpec {
    let p = plan(t);
    let f = Form::of(t, b);

    let mut v = humanoid(&p, SKIN, IVORY, NAVY, IVORY);
    v.push(
        part(Body(Chest), Cone(0.19, 0.25), IVORY, [0.0, 0.20, 0.10])
            .scale([1.0, 1.0, 0.35])
            .rot([0.0, 0.0, 180.0]),
    );
    for s in [-1.0, 1.0] {
        v.push(
            part(
                Body(Head),
                Box(0.025, if f.a { 0.35 } else { 0.25 }, 0.05),
                GOLD,
                [s * 0.10, 0.16, -0.05],
            )
            .rot([0.0, 0.0, s * -22.0]),
        );
    }
    let w = Prop(Socket::WeaponR);
    v.push(part(w, Cyl(0.020, 1.02), NAVY, [0.0, 0.0, 0.22]).rot([90.0, 0.0, 0.0]));
    v.push(
        part(
            w,
            Cone(if f.a { 0.09 } else { 0.065 }, 0.27),
            GOLD,
            [0.0, 0.0, 0.84],
        )
        .rot([90.0, 0.0, 0.0])
        .scale([0.7, 1.0, 0.35]),
    );
    if f.b {
        v.push(
            part(
                w,
                Ring(0.017, if f.t2 { 0.17 } else { 0.12 }, 310.0),
                GOLD,
                [0.0, 0.0, 0.62],
            )
            .rot([90.0, 0.0, 0.0]),
        );
    }
    if f.a {
        v.push(part(
            Body(UpperArmR),
            Bevel(0.16, 0.11, 0.17, 0.02),
            IVORY,
            [0.0, 0.0, 0.0],
        ));
    }
    if f.t2 {
        v.push(part(
            Body(Chest),
            Box(0.035, 0.25, 0.02),
            GOLD,
            [0.0, 0.17, 0.16],
        ));
    }

    FigureSpec { plan: p, parts: v }
}
pub fn plan(t: u8) -> Proportions {
    let mut p = body::plan(false, 1.0, t);
    p.arm = 0.20;
    p
}
