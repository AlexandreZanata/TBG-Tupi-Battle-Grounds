//! Authored five-form low-poly figure; proportions and palette follow the shared rig-v2 contract.
use super::forms::*;
pub fn spec(t: u8, b: u8) -> FigureSpec {
    let p = plan(t);
    let f = Form::of(t, b);

    let mut v = humanoid(&p, OGRE_SKIN, MOSS, CHARCOAL, OGRE_SKIN);
    v.push(part(Body(Head), Ball(0.20), CHARCOAL, [0.0, -0.02, 0.02]).scale([1.3, 0.7, 0.85]));
    v.push(part(
        Body(Head),
        Box(0.33, 0.035, 0.045),
        IVORY,
        [0.0, -0.07, 0.17],
    ));
    for s in [-1.0, 1.0] {
        v.push(part(
            Body(Head),
            Ball(0.075),
            OGRE_SKIN,
            [s * 0.17, 0.09, 0.07],
        ));
        v.push(part(Body(Head), Ball(0.025), TURQUOISE, [s * 0.18, 0.10, 0.13]).glow());
    }
    bowl(
        &mut v,
        Prop(Socket::OffhandL),
        0.16,
        ROCK,
        if f.b { TURQUOISE } else { SAP },
    );
    spoon(
        &mut v,
        Prop(Socket::WeaponR),
        if f.a { 0.44 } else { 0.35 },
        OAK,
    );
    v.push(part(
        Body(Hips),
        Box(0.21, 0.25, 0.025),
        LINEN,
        [0.0, -0.09, 0.14],
    ));
    if f.a {
        for s in [-1.0, 1.0] {
            v.push(part(
                Body(UpperArmL),
                Ball(0.065),
                IVORY,
                [s * 0.06, -0.04, 0.0],
            ));
        }
    }
    if f.b {
        v.push(
            part(
                Body(Chest),
                Ring(0.018, 0.12, 300.0),
                TURQUOISE,
                [0.0, 0.15, 0.14],
            )
            .rot([90.0, 0.0, 0.0]),
        );
    }
    if f.t2 {
        v.push(part(
            Prop(Socket::OffhandL),
            Ring(0.018, 0.19, 360.0),
            IVORY,
            [0.0, 0.0, 0.06],
        ));
    }
    FigureSpec { plan: p, parts: v }
}
pub fn plan(t: u8) -> Proportions {
    let mut p = body::plan(false, 0.93, t);
    p.torso.x *= 1.1;
    p.leg *= 0.85;
    p
}
