//! Barrel Keeper: a two-hand horizontal cask, brewing apron and distinct remedy Evolutions.
use super::common::*;

pub const CASK: &[[f32; 2]] = &[
    [0.145, -0.23],
    [0.185, -0.18],
    [0.205, -0.08],
    [0.205, 0.08],
    [0.185, 0.18],
    [0.145, 0.23],
];
pub fn plan(t: u8) -> Proportions {
    super::common::plan(t, 1.16, 0.39, 0.47)
}
pub fn parent(n: crate::art::rig::Node) -> Option<crate::art::rig::Node> {
    if matches!(n, Prop(Socket::WeaponR)) {
        Some(Body(Chest))
    } else {
        n.parent()
    }
}
pub fn rest(n: crate::art::rig::Node, p: &Proportions) -> Vec3 {
    if matches!(n, Prop(Socket::WeaponR)) {
        Vec3::new(0.0, 0.12, 0.29)
    } else {
        crate::art::rig::rest(n, p)
    }
}
pub fn spec(t: u8, b: u8) -> FigureSpec {
    let p = plan(t);
    let f = Form::of(t, b);
    let mut v = ogre(&p, OGRE_SKIN, if f.a { LINEN } else { DARK_LEATHER });
    v.push(
        part(
            Body(Head),
            Frustum(0.125, 0.145, 0.065),
            LINEN,
            [0.0, 0.11, 0.0],
        )
        .scale([1.0, 1.0, 0.90]),
    );
    v.push(
        part(
            Secondary(Spring::HeadGear),
            Prism(0.035, 0.24),
            LINEN,
            [0.10, 0.10, -0.15],
        )
        .scale([1.0, 1.0, 0.3])
        .rot([-65.0, 20.0, 12.0]),
    );
    v.push(part(
        Body(Chest),
        Bevel(0.29, 0.28, 0.035, 0.01),
        if f.a { LINEN } else { LEATHER },
        [0.0, 0.16, 0.14],
    ));
    v.push(
        part(
            Body(Hips),
            Frustum(0.16, 0.19, 0.25),
            if f.a { LINEN } else { LEATHER },
            [0.0, -0.13, 0.05],
        )
        .scale([1.0, 1.0, 0.57]),
    );
    let n = Prop(Socket::WeaponR);
    v.push(
        part(
            n,
            Lathe(CASK),
            if f.a { OAK } else { DARK_LEATHER },
            [0.0, 0.0, 0.0],
        )
        .rot([0.0, 0.0, 90.0]),
    );
    for x in [-0.17, 0.17] {
        v.push(part(n, Ring(0.017, 0.188, 360.0), IRON, [x, 0.0, 0.0]).rot([0.0, 0.0, 90.0]));
        v.push(
            part(n, Cyl(0.145, 0.018), OAK, [x.signum() * 0.232, 0.0, 0.0]).rot([0.0, 0.0, 90.0]),
        );
    }
    v.push(part(n, Cyl(0.057, 0.016), BRASS, [0.0, 0.0, 0.206]).rot([90.0, 0.0, 0.0]));
    v.push(part(n, Prism(0.026, 0.06), HOOD_GREEN, [0.0, 0.0, 0.218]).scale([1.0, 1.0, 0.25]));
    if f.a || f.b {
        v.push(part(Body(Hips), Ball(0.06), AMBER, [-0.20, -0.05, 0.0]).scale([0.8, 1.25, 0.8]));
        for s in [-1.0f32, 1.0] {
            v.push(part(
                Body(UpperArmL),
                Bevel(0.15, 0.07, 0.16, 0.018),
                LEATHER,
                [0.0, -0.035 + s * 0.018, 0.0],
            ));
        }
    }
    if f.t2 {
        v.push(
            part(n, Ball(0.047), AMBER, [-0.05, 0.205, 0.0])
                .scale([0.8, 1.3, 0.8])
                .glow(),
        );
        if f.b {
            v.push(part(n, Cyl(0.018, 0.09), BRASS, [0.04, -0.20, 0.10]).rot([45.0, 0.0, 0.0]));
        }
    }
    FigureSpec { plan: p, parts: v }
}
