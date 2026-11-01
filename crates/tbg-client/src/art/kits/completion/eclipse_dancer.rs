//! Authored five-form low-poly figure; proportions and palette follow the shared rig-v2 contract.
use super::forms::*;
pub fn spec(t: u8, b: u8) -> FigureSpec {
    let p = plan(t);
    let f = Form::of(t, b);

    let mut v = humanoid(&p, SKIN, NAVY, NAVY, IVORY);
    for s in [-1.0, 1.0] {
        v.push(
            part(
                Body(Hips),
                Box(0.12, if s > 0.0 { 0.24 } else { 0.16 }, 0.025),
                if s > 0.0 { IVORY } else { NAVY },
                [s * 0.12, -0.06, 0.10],
            )
            .rot([0.0, 0.0, s * -15.0]),
        );
        v.push(
            part(
                Body(Head),
                Ring(0.016, 0.19, 155.0),
                if s > 0.0 { GOLD } else { IVORY },
                [s * 0.025, 0.12, -0.07],
            )
            .rot([0.0, 0.0, if s > 0.0 { -70.0 } else { 110.0 }]),
        );
    }
    blade(
        &mut v,
        Prop(Socket::WeaponR),
        if f.a { 0.55 } else { 0.42 },
        GOLD,
        false,
    );
    blade(
        &mut v,
        Prop(Socket::OffhandL),
        if f.b { 0.48 } else { 0.35 },
        IVORY,
        true,
    );
    if f.a {
        v.push(
            part(
                Body(UpperArmR),
                Box(0.15, 0.10, 0.18),
                GOLD,
                [0.0, 0.0, 0.0],
            )
            .rot([0.0, 0.0, -20.0]),
        );
    }
    if f.b {
        v.push(
            part(
                Body(UpperArmL),
                Ring(0.022, 0.11, 210.0),
                IVORY,
                [0.0, 0.0, 0.0],
            )
            .rot([90.0, 0.0, 0.0]),
        );
    }
    if f.t2 {
        v.push(
            part(
                Body(Chest),
                Bevel(0.12, 0.12, 0.04, 0.02),
                if f.a { GOLD } else { IVORY },
                [0.0, 0.19, 0.14],
            )
            .rot([0.0, 0.0, 45.0]),
        );
    }

    FigureSpec { plan: p, parts: v }
}
pub fn plan(t: u8) -> Proportions {
    let p = body::plan(false, 1.0, t);
    p
}
