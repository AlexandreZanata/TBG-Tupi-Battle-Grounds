//! Rivet Ballista: five native forms with its own equipment and silhouette.
use super::common::*;
pub fn plan(t: u8) -> Proportions {
    super::common::plan(t, 1.05, 0.50, 0.32)
}
pub fn spec(t: u8, b: u8) -> FigureSpec {
    let p = plan(t);
    let f = Form::of(t, b);
    let mut v = Vec::new();
    body::humanoid(
        &mut v,
        &p,
        &body::Look {
            skin: IRON,
            cloth: cloth(f, BRASS),
            legs: GUNMETAL,
            boots: BRASS,
        },
    );

    v.push(part(
        Body(Head),
        Bevel(0.22, 0.15, 0.28, 0.03),
        if f.b { GUNMETAL } else { IVORY },
        [0.0, 0.0, 0.04],
    ));
    v.push(
        part(
            Body(Head),
            Box(0.13, 0.025, 0.025),
            ORANGE,
            [0.0, 0.035, 0.19],
        )
        .glow(),
    );
    v.push(part(
        Prop(Socket::WeaponR),
        Bevel(0.10, 0.10, if f.a { 0.52 } else { 0.38 }, 0.015),
        if f.a { IVORY } else { GUNMETAL },
        [0.0, 0.0, 0.15],
    ));
    v.push(part(
        Prop(Socket::WeaponR),
        Box(0.27, 0.045, 0.07),
        BRASS,
        [0.0, 0.0, 0.25],
    ));
    for x in if f.b {
        vec![-0.07, 0.0, 0.07]
    } else {
        vec![0.0]
    } {
        v.push(part(
            Secondary(Spring::Back),
            Box(0.055, if f.t2 { 0.48 } else { 0.34 }, 0.07),
            if f.a { STEEL } else { BRASS },
            [x, 0.24, 0.0],
        ));
    }
    for n in [Body(ShinL), Body(ShinR)] {
        v.push(part(
            n,
            Box(0.035, 0.23, 0.06),
            GUNMETAL,
            [0.0, -0.10, 0.04],
        ));
    }
    tier(&mut v, f, [0.0, 0.33, 0.12], Cone(0.055, 0.12));
    FigureSpec { plan: p, parts: v }
}
