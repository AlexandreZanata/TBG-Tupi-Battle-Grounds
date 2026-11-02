//! UndertowHerald equipment and five authored shell Evolution forms.
use super::common::*;
pub fn spec(t: u8, b: u8) -> FigureSpec {
    let p = plan(UndertowHerald, t);
    let f = Form::of(t, b);
    let mut v = marine(UndertowHerald, &p, f);

    let n = Prop(Socket::WeaponR);
    let c = if f.b { PLUM } else { IVORY };
    v.push(part(n, Frustum(0.15, 0.035, 0.33), c, [0.0, 0.0, 0.12]).rot([75.0, 0.0, -12.0]));
    v.push(part(n, Ring(0.035, 0.11, 300.0), c, [0.0, 0.02, 0.15]).rot([90.0, 0.0, 0.0]));
    v.push(
        part(
            n,
            Ring(0.018, 0.15, 360.0),
            if f.a { GOLD } else { LEATHER },
            [0.0, 0.06, 0.28],
        )
        .rot([75.0, 0.0, -12.0]),
    );
    if f.b {
        for x in [-0.09, 0.09] {
            v.push(part(n, Prism(0.045, 0.17), PLUM, [x, 0.09, 0.18]).rot([20.0, 0.0, x * 120.0]));
        }
    }
    if f.a {
        v.push(part(n, Ring(0.012, 0.17, 360.0), IVORY, [0.0, 0.08, 0.29]).rot([75.0, 0.0, -12.0]));
    }
    finish(&mut v, f, n, [0.0, 0.09, 0.18]);
    FigureSpec { plan: p, parts: v }
}
