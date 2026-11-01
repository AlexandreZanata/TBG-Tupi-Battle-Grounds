//! Authored five-form low-poly figure; proportions and palette follow the shared rig-v2 contract.
use super::forms::*;
pub fn spec(t: u8, b: u8) -> FigureSpec {
    let p = plan(t);
    let f = Form::of(t, b);

    let mut v = humanoid(&p, CHARCOAL, GUNMETAL, CHARCOAL, DARK);
    v.push(part(Body(Head), Ball(0.13), GUNMETAL, [0.0, -0.035, 0.14]).scale([1.0, 0.65, 1.0]));
    for i in -2..=2 {
        v.push(
            part(
                Secondary(Spring::Back),
                Cone(0.065, if f.a { 0.65 } else { 0.47 }),
                if f.b { TURQUOISE } else { IVORY },
                [i as f32 * 0.075, 0.27, -0.13],
            )
            .rot([-15.0, 0.0, -i as f32 * 14.0]),
        );
    }
    needle(
        &mut v,
        Secondary(Spring::HeadGear),
        if f.a { 0.57 } else { 0.43 },
        IVORY,
    );
    v.push(part(
        Body(Chest),
        Box(0.23, 0.035, 0.025),
        TURQUOISE,
        [0.0, 0.23, 0.13],
    ));
    if f.b {
        for s in [-1.0, 1.0] {
            v.push(part(
                Body(ForearmR),
                Ring(0.012, 0.08, 270.0),
                TURQUOISE,
                [s * 0.02, -0.06, 0.01],
            ));
        }
    }
    if f.t2 {
        v.push(
            part(
                Body(Head),
                Cone(0.05, 0.21),
                if f.a { IVORY } else { TURQUOISE },
                [0.0, 0.19, 0.0],
            )
            .rot([-15.0, 0.0, 0.0]),
        );
    }
    team(&mut v, Body(Hips), 0.10);
    FigureSpec { plan: p, parts: v }
}
pub fn plan(t: u8) -> Proportions {
    let p = body::plan(false, 1.0, t);
    p
}
