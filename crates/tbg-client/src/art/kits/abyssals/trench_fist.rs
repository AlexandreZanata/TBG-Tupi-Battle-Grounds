//! TrenchFist equipment and five authored shell Evolution forms.
use super::common::*;
pub fn spec(t: u8, b: u8) -> FigureSpec {
    let p = plan(TrenchFist, t);
    let f = Form::of(t, b);
    let mut v = marine(TrenchFist, &p, f);

    let n = Prop(Socket::WeaponR);
    v.push(
        part(
            n,
            Rock(if f.a { 0.22 } else { 0.19 }, 731),
            if f.b { PLUM } else { IVORY },
            [0.0, -0.07, 0.04],
        )
        .scale([1.0, 1.2, 0.85]),
    );
    for i in 0..if f.b { 3 } else { 2 } {
        v.push(
            part(
                n,
                Prism(0.055, 0.16),
                if f.b { PLUM } else { PALE },
                [-0.07 + i as f32 * 0.07, 0.08, 0.02],
            )
            .rot([0.0, 0.0, -20.0]),
        );
    }
    v.push(
        part(
            Body(Chest),
            Bevel(0.07, 0.40, 0.03, 0.009),
            OILSKIN_YELLOW,
            [0.0, 0.14, 0.13],
        )
        .rot([0.0, 0.0, -35.0]),
    );
    finish(&mut v, f, n, [0.0, 0.12, 0.12]);
    FigureSpec { plan: p, parts: v }
}
