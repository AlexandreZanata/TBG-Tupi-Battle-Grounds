//! Authored five-form low-poly figure; proportions and palette follow the shared rig-v2 contract.
use super::forms::*;
pub fn spec(t: u8, b: u8) -> FigureSpec {
    let p = plan(t);
    let f = Form::of(t, b);

    let mut v = insect(&p, f, 0.17);
    antennae(&mut v, 0.17, false);
    v.push(part(Body(Hips), Ball(0.19), AMBER, [0.0, 0.05, -0.13]).scale([1.0, 0.8, 1.0]));
    for i in 0..3 {
        v.push(
            part(
                Body(Hips),
                Ring(0.021, 0.15, 330.0),
                CHITIN,
                [0.0, 0.02, -0.07 - i as f32 * 0.055],
            )
            .rot([90.0, 0.0, 0.0]),
        );
    }
    for s in [-1.0, 1.0] {
        v.push(
            part(
                Secondary(Spring::HeadGear),
                Ball(0.15),
                LINEN,
                [s * 0.12, 0.26, -0.12],
            )
            .scale([0.55, 1.25, 0.15])
            .rot([0.0, 0.0, s * -18.0]),
        );
    }
    v.push(part(
        Body(Hips),
        Box(0.24, 0.23, 0.02),
        LINEN,
        [0.0, -0.07, 0.15],
    ));
    bowl(
        &mut v,
        Prop(Socket::OffhandL),
        if f.b { 0.18 } else { 0.14 },
        OAK,
        SAP,
    );
    spoon(&mut v, Prop(Socket::WeaponR), 0.24, AMBER);
    if f.a {
        v.push(
            part(
                Body(Chest),
                Ring(0.018, 0.09, 300.0),
                IVORY,
                [0.0, 0.19, 0.13],
            )
            .rot([90.0, 0.0, 0.0]),
        );
    }
    if f.t2 {
        v.push(part(
            Prop(Socket::OffhandL),
            Ring(0.018, 0.20, 360.0),
            if f.b { SAP } else { IVORY },
            [0.0, 0.0, 0.06],
        ));
    }
    FigureSpec { plan: p, parts: v }
}
pub fn plan(t: u8) -> Proportions {
    let mut p = body::plan(false, 0.92, t);
    p.torso.x *= 0.83;
    p.arm *= 0.95;
    p
}
