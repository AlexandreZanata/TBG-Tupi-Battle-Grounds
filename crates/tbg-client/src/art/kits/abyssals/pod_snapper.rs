//! Pod Snapper: bark anatomy, split forearm pod and seed/leaf Evolution silhouettes.
use super::common::*;
pub fn spec(t: u8, b: u8) -> FigureSpec {
    let p = plan(PodSnapper, t);
    let f = Form::of(t, b);
    let mut v = Vec::new();
    body::humanoid(
        &mut v,
        &p,
        &body::Look {
            skin: OAK,
            cloth: OAK,
            legs: DARK_LEATHER,
            boots: OAK,
        },
    );
    v.retain(|a| !matches!(a.node, Body(Head)));
    v.push(
        part(Body(Head), Prism(0.14, 0.23), OAK, [0.0, 0.0, 0.015])
            .scale([0.9, 1.0, 0.75])
            .rot([180.0, 0.0, 0.0]),
    );
    for s in [-1.0f32, 1.0] {
        v.push(part(Body(Head), Ball(0.023), AMBER, [s * 0.055, 0.01, 0.095]).glow());
    }
    for i in 0..if f.t2 {
        5
    } else if f.a {
        4
    } else {
        3
    } {
        v.push(
            part(
                Body(Head),
                Prism(0.07, 0.27),
                if f.a { MOSS } else { HOOD_GREEN },
                [-0.07 + i as f32 * 0.035, 0.16, 0.0],
            )
            .scale([1.0, 1.0, 0.25])
            .rot([20.0, 0.0, -35.0 + i as f32 * 18.0]),
        );
    }
    v.push(
        part(
            Body(Head),
            Ring(0.014, 0.075, 270.0),
            MOSS,
            [0.09, 0.28, 0.0],
        )
        .rot([90.0, 0.0, 0.0]),
    );
    let n = Prop(Socket::WeaponR);
    let c = if f.a { MOSS } else { HOOD_GREEN };
    for s in [-1.0f32, 1.0] {
        v.push(
            part(
                if s < 0.0 {
                    Prop(Socket::StringTop)
                } else {
                    Prop(Socket::StringBottom)
                },
                Ball(0.19),
                c,
                [s * 0.075, 0.045, 0.06],
            )
            .scale([0.55, 1.25, 0.70]),
        );
    }
    for y in [-0.09, 0.04, 0.17] {
        v.push(part(
            n,
            Ball(0.055),
            if f.t2 && f.a { AMBER } else { IVORY },
            [0.0, y, 0.15],
        ));
    }
    if f.b {
        v.push(part(n, Prism(0.10, 0.35), MOSS, [0.0, 0.05, -0.08]).scale([0.4, 1.0, 0.7]));
    }
    if f.t2 {
        v.push(
            part(
                Body(Head),
                Ball(0.055),
                if f.a { BLOSSOM } else { AMBER },
                [0.0, 0.30, 0.0],
            )
            .glow(),
        );
    }
    for shoulder in [UpperArmL, UpperArmR] {
        v.push(
            part(Body(shoulder), Prism(0.10, 0.13), MOSS, [0.0, 0.015, 0.0])
                .scale([1.0, 1.0, 0.3])
                .rot([30.0, 0.0, 30.0]),
        );
    }
    FigureSpec { plan: p, parts: v }
}
