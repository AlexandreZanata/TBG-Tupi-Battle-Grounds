//! Hide Mender: five native forms with its own equipment and silhouette.
use super::common::*;
pub fn plan(t: u8) -> Proportions {
    super::common::plan(t, 1.08, 0.38, 0.41)
}
pub fn spec(t: u8, b: u8) -> FigureSpec {
    let p = plan(t);
    let f = Form::of(t, b);
    let mut v = ogre(&p, OGRE_SKIN, cloth(f, LEATHER));

    for x in [-0.06, 0.06] {
        v.push(
            part(
                Body(Head),
                Ring(0.012, 0.044, 360.0),
                BRASS,
                [x, 0.02, 0.148],
            )
            .rot([90.0, 0.0, 0.0]),
        );
    }
    v.push(part(
        Body(Chest),
        Bevel(0.43, 0.065, 0.28, 0.015),
        if f.a {
            LINEN
        } else if f.b {
            DARK_LEATHER
        } else {
            LEATHER
        },
        [0.0, 0.27, 0.0],
    ));
    for x in [-0.13, 0.0, 0.13] {
        v.push(part(
            Body(Chest),
            Box(0.012, 0.04, 0.02),
            LINEN,
            [x, 0.25, 0.151],
        ));
    }
    v.push(part(
        Secondary(Spring::Back),
        Cyl(
            if f.b { 0.14 } else { 0.10 },
            if f.t2 { 0.44 } else { 0.30 },
        ),
        AMBER,
        [0.0, 0.12, -0.08],
    ));
    for y in [-0.04, 0.28] {
        v.push(part(
            Secondary(Spring::Back),
            Cyl(0.15, 0.025),
            OAK,
            [0.0, y, -0.08],
        ));
    }
    v.push(
        part(
            Prop(Socket::WeaponR),
            Cone(0.018, if f.a { 0.43 } else { 0.32 }),
            IVORY,
            [0.0, 0.0, 0.16],
        )
        .rot([90.0, 0.0, 0.0]),
    );
    v.push(
        part(
            Prop(Socket::WeaponR),
            Ring(0.012, 0.04, 360.0),
            BRASS,
            [0.0, 0.0, -0.015],
        )
        .rot([90.0, 0.0, 0.0]),
    );
    v.push(part(
        Body(Hips),
        Bevel(0.13, 0.18, 0.12, 0.02),
        LEATHER,
        [0.20, -0.05, 0.0],
    ));
    tier(&mut v, f, [0.0, 0.30, -0.14], Ring(0.018, 0.10, 360.0));
    FigureSpec { plan: p, parts: v }
}
