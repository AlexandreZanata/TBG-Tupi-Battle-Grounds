//! Authored five-form low-poly figure; proportions and palette follow the shared rig-v2 contract.
use super::forms::*;
pub fn spec(t: u8, b: u8) -> FigureSpec {
    let p = plan(t);
    let f = Form::of(t, b);

    let mut v = Vec::new();
    body::skeleton(&mut v, &p, BONE);
    body::tunic(&mut v, &p, VIOLET, BONE, RUST);
    v.push(
        part(
            Body(Hips),
            Frustum(0.18, 0.27, 0.30),
            VIOLET,
            [0.0, -0.10, 0.0],
        )
        .scale([1.0, 1.0, 0.7]),
    );
    for i in -2..=2 {
        let a = i as f32 * 18.0;
        v.push(
            part(
                Body(Head),
                Box(0.07, if f.b { 0.35 } else { 0.27 }, 0.025),
                if i % 2 == 0 { VIOLET } else { DARK },
                [i as f32 * 0.075, 0.12, -0.15],
            )
            .rot([0.0, 0.0, -a]),
        );
    }
    bell(
        &mut v,
        Prop(Socket::WeaponR),
        if f.a { 0.18 } else { 0.14 },
        BRASS,
    );
    v.push(part(
        Prop(Socket::OffhandL),
        Box(0.12, 0.04, 0.14),
        BONE,
        [0.0, 0.0, 0.03],
    ));
    if f.a {
        for i in 0..3 {
            v.push(
                part(
                    Body(Chest),
                    Ring(0.008, 0.045, 360.0),
                    BRASS,
                    [-0.09 + i as f32 * 0.09, 0.21, 0.13],
                )
                .rot([90.0, 0.0, 0.0]),
            );
        }
    }
    if f.t2 {
        v.push(
            part(
                Prop(Socket::WeaponR),
                Ring(0.014, 0.19, 300.0),
                if f.b { GLOW } else { BONE },
                [0.0, -0.08, 0.0],
            )
            .glow(),
        );
    }
    FigureSpec { plan: p, parts: v }
}
pub fn plan(t: u8) -> Proportions {
    let p = body::plan(false, 1.0, t);
    p
}
