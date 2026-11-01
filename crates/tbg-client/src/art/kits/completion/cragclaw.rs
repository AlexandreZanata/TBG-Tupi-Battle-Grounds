//! Authored five-form low-poly figure; proportions and palette follow the shared rig-v2 contract.
use super::forms::*;
pub fn spec(t: u8, b: u8) -> FigureSpec {
    let p = plan(t);
    let f = Form::of(t, b);

    let mut v = humanoid(&p, CHARCOAL, DARK, CHARCOAL, GUNMETAL);
    v.push(
        part(Body(Head), Cone(0.14, 0.28), CHARCOAL, [0.0, 0.0, 0.16])
            .rot([90.0, 0.0, 0.0])
            .scale([1.15, 1.0, 0.65]),
    );
    v.push(part(
        Body(ForearmR),
        Bevel(if f.a { 0.28 } else { 0.21 }, 0.25, 0.25, 0.04),
        if f.a { IVORY } else { ROCK },
        [0.0, -0.11, 0.07],
    ));
    v.push(
        part(
            Prop(Socket::WeaponR),
            Bevel(0.28, 0.045, if f.a { 0.50 } else { 0.37 }, 0.025),
            IVORY,
            [0.0, 0.0, 0.19],
        )
        .rot([0.0, -15.0, 0.0]),
    );
    v.push(part(
        Body(UpperArmL),
        Box(0.06, 0.13, 0.07),
        CHARCOAL,
        [0.0, -0.05, 0.0],
    ));
    for i in 0..3 {
        v.push(
            part(
                Body(Spine),
                Cone(0.07, 0.16),
                if f.b { TURQUOISE } else { ROCK },
                [0.0, 0.08 + i as f32 * 0.10, -0.18],
            )
            .rot([-35.0, 0.0, 0.0]),
        );
    }
    if f.b {
        v.push(
            part(
                Body(Head),
                Box(0.21, 0.035, 0.035),
                TURQUOISE,
                [0.0, 0.06, 0.20],
            )
            .glow(),
        );
    }
    if f.t2 {
        v.push(
            part(
                Body(ForearmR),
                Box(0.035, 0.24, 0.29),
                if f.b { TURQUOISE } else { GOLD },
                [0.0, -0.10, 0.12],
            )
            .rot([10.0, 0.0, 0.0]),
        );
    }

    for side in [-1.0, 1.0] {
        v.push(part(
            Body(Head),
            Ball(0.032),
            CHARCOAL,
            [side * 0.085, 0.04, 0.125],
        ));
        v.push(
            part(
                Body(Head),
                Ball(0.022),
                TURQUOISE,
                [side * 0.085, 0.045, 0.15],
            )
            .glow(),
        );
    }
    FigureSpec { plan: p, parts: v }
}
pub fn plan(t: u8) -> Proportions {
    let mut p = body::plan(false, 1.07, t);
    p.arm *= 1.15;
    p
}
