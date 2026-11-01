//! Authored five-form low-poly figure; proportions and palette follow the shared rig-v2 contract.
use super::forms::*;
pub fn spec(t: u8, b: u8) -> FigureSpec {
    let p = plan(t);
    let f = Form::of(t, b);

    let mut v = humanoid(&p, SKIN, IVORY, NAVY, GOLD);
    body::tunic(&mut v, &p, IVORY, GOLD, NAVY);
    v.push(part(Body(Head), Ball(0.15), NAVY, [0.0, 0.055, -0.08]).scale([1.05, 0.75, 0.70]));
    let w = Prop(Socket::WeaponR);
    v.push(
        part(
            w,
            Ring(0.024, if f.a { 0.19 } else { 0.16 }, 360.0),
            GOLD,
            [0.0, 0.0, 0.0],
        )
        .rot([90.0, 0.0, 0.0]),
    );
    v.push(part(w, Ball(0.09), GOLD, [0.0, 0.0, 0.0]).glow());
    for i in -2..=2 {
        v.push(
            part(
                w,
                Box(0.016, 0.11, 0.016),
                IVORY,
                [i as f32 * 0.045, -0.035, 0.0],
            )
            .rot([0.0, 0.0, -i as f32 * 20.0]),
        );
    }
    if f.b {
        v.push(part(w, Ring(0.012, 0.22, 270.0), IVORY, [0.0, 0.0, 0.0]).rot([90.0, 0.0, 0.0]));
    }
    if f.a {
        v.push(part(
            Body(Chest),
            Box(0.20, 0.055, 0.025),
            GOLD,
            [0.0, 0.28, 0.13],
        ));
    }
    if f.t2 {
        v.push(part(
            Body(Head),
            Ring(0.014, 0.135, 210.0),
            GOLD,
            [0.0, 0.01, 0.0],
        ));
    }
    let grip = if f.a { 0.17 } else { 0.14 };
    for part in &mut v {
        if matches!(part.node, Prop(Socket::WeaponR)) {
            part.at.x += grip;
        }
    }
    team(&mut v, Body(Hips), 0.10);
    FigureSpec { plan: p, parts: v }
}
pub fn plan(t: u8) -> Proportions {
    let mut p = body::plan(false, 1.0, t);
    p.arm = 0.19;
    p
}
