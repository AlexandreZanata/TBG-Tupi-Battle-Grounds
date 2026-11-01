//! Authored five-form low-poly figure; proportions and palette follow the shared rig-v2 contract.
use super::forms::*;
pub fn spec(t: u8, b: u8) -> FigureSpec {
    let p = plan(t);
    let f = Form::of(t, b);

    let mut v = humanoid(&p, IVORY, IVORY, NAVY, IVORY);
    v.retain(|q| !matches!(q.node, Body(Head)));
    v.push(part(
        Body(Head),
        Bevel(0.22, 0.25, 0.18, 0.04),
        NAVY,
        [0.0, 0.0, 0.0],
    ));
    for a in [0.0, 72.0, 144.0, 216.0, 288.0] {
        v.push(
            part(Body(Head), Cone(0.045, 0.13), GOLD, [0.0, 0.0, 0.105])
                .rot([0.0, 0.0, a])
                .scale([1.0, 1.0, 0.25]),
        );
    }
    let w = Prop(Socket::WeaponR);
    v.push(part(w, Cyl(0.028, 0.68), NAVY, [0.0, 0.0, 0.20]).rot([90.0, 0.0, 0.0]));
    v.push(part(
        w,
        Bevel(if f.a { 0.43 } else { 0.35 }, 0.27, 0.27, 0.06),
        IVORY,
        [0.0, 0.0, 0.57],
    ));
    for a in [0.0, 90.0] {
        v.push(part(w, Box(0.04, 0.36, 0.29), GOLD, [0.0, 0.0, 0.57]).rot([0.0, 0.0, a]));
    }
    if f.b {
        v.push(
            part(
                w,
                Ball(if f.t2 { 0.14 } else { 0.10 }),
                GOLD,
                [0.0, 0.0, 0.74],
            )
            .glow(),
        );
    }
    if f.a {
        v.push(part(
            Body(Chest),
            Bevel(0.24, 0.22, 0.09, 0.025),
            IVORY,
            [0.0, 0.21, 0.10],
        ));
    }
    if f.t2 {
        for s in [-1.0, 1.0] {
            v.push(part(w, Cone(0.07, 0.20), GOLD, [s * 0.23, 0.0, 0.57]).rot([
                0.0,
                0.0,
                s * -90.0,
            ]));
        }
    }
    team(&mut v, Body(Hips), 0.10);
    FigureSpec { plan: p, parts: v }
}
pub fn plan(t: u8) -> Proportions {
    let mut p = body::plan(false, 1.04, t);
    p.arm = 0.2;
    p
}
