//! Authored five-form low-poly figure; proportions and palette follow the shared rig-v2 contract.
use super::forms::*;
pub fn spec(t: u8, b: u8) -> FigureSpec {
    let p = plan(t);
    let f = Form::of(t, b);

    let mut v = insect(&p, f, 0.17);
    antennae(&mut v, 0.12, false);
    v.push(part(Body(Chest), Ball(0.18), MOSS, [0.0, 0.22, -0.02]).scale([1.05, 0.80, 1.10]));
    v.push(
        part(
            Body(Hips),
            Ball(if f.a { 0.21 } else { 0.17 }),
            if f.b { SAP } else { AMBER },
            [0.0, 0.02, -0.18],
        )
        .scale([0.85, 0.95, 1.2])
        .glow(),
    );
    for s in [-1.0, 1.0] {
        v.push(
            part(
                Body(Hips),
                Box(0.13, 0.25, 0.07),
                CHITIN,
                [s * 0.12, 0.05, -0.21],
            )
            .rot([0.0, 0.0, s * -20.0]),
        );
        v.push(
            part(
                Prop(if s > 0.0 {
                    Socket::OffhandL
                } else {
                    Socket::WeaponR
                }),
                Cone(0.05, 0.16),
                IVORY,
                [0.0, 0.0, 0.10],
            )
            .rot([90.0, 0.0, 0.0]),
        );
    }
    if f.a {
        v.push(
            part(Body(Chest), Cone(0.07, 0.21), IVORY, [0.0, 0.32, -0.06]).rot([-28.0, 0.0, 0.0]),
        );
    }
    if f.t2 {
        v.push(
            part(
                Body(Hips),
                Ring(0.018, 0.21, 300.0),
                if f.b { SAP } else { IVORY },
                [0.0, 0.03, -0.18],
            )
            .rot([90.0, 0.0, 0.0]),
        );
    }
    FigureSpec { plan: p, parts: v }
}
pub fn plan(t: u8) -> Proportions {
    let mut p = body::plan(false, 0.85, t);
    p.torso.x *= 0.83;
    p.arm *= 0.95;
    p
}
