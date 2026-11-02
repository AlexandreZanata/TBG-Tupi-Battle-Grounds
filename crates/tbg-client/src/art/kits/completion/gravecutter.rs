//! Authored five-form low-poly figure; proportions and palette follow the shared rig-v2 contract.
use super::forms::*;
pub fn spec(t: u8, b: u8) -> FigureSpec {
    let p = plan(t);
    let f = Form::of(t, b);

    let mut v = Vec::new();
    body::skeleton(&mut v, &p, BONE);
    v.extend([
        part(
            Body(Chest),
            Bevel(0.29, 0.24, 0.20, 0.025),
            VIOLET,
            [0.0, 0.18, -0.015],
        ),
        part(
            Body(Hips),
            Box(0.10, 0.21, 0.025),
            VIOLET,
            [-0.10, -0.06, -0.10],
        )
        .rot([0.0, 0.0, -12.0]),
        part(
            Body(Hips),
            Box(0.10, 0.21, 0.025),
            DARK,
            [0.10, -0.06, -0.10],
        )
        .rot([0.0, 0.0, 12.0]),
        part(Body(Head), Box(0.14, 0.04, 0.02), DARK, [0.0, -0.03, 0.13]),
        part(
            Prop(Socket::WeaponR),
            Ring(0.026, if f.a { 0.23 } else { 0.18 }, 225.0),
            if f.a { BONE } else { STEEL },
            [0.0, 0.0, 0.24],
        )
        .rot([90.0, 0.0, -25.0]),
        part(
            Prop(Socket::WeaponR),
            Cyl(0.022, 0.14),
            RUST,
            [0.0, 0.0, 0.04],
        )
        .rot([90.0, 0.0, 0.0]),
    ]);
    for i in 0..5 {
        v.push(
            part(
                Body(ForearmR),
                Ring(0.008, 0.025, 360.0),
                RUST,
                [0.025, -0.025 - i as f32 * 0.025, 0.025],
            )
            .rot([if i % 2 == 0 { 90.0 } else { 0.0 }, 0.0, 0.0]),
        );
    }
    if f.a {
        v.push(part(
            Body(UpperArmR),
            Bevel(0.19, 0.10, 0.16, 0.02),
            BONE,
            [0.0, -0.02, 0.0],
        ));
    }
    if f.b {
        v.push(
            part(
                Body(Head),
                Ring(0.015, 0.16, 210.0),
                GLOW,
                [0.0, 0.05, -0.035],
            )
            .rot([90.0, 0.0, 0.0])
            .glow(),
        );
    }
    if f.t2 {
        v.push(
            part(
                Prop(Socket::WeaponR),
                Cone(0.045, 0.18),
                if f.a { BONE } else { GLOW },
                [0.12, 0.0, 0.33],
            )
            .rot([70.0, 0.0, -20.0]),
        );
    }
    FigureSpec { plan: p, parts: v }
}
pub fn plan(t: u8) -> Proportions {
    body::plan(false, 1.0, t)
}
