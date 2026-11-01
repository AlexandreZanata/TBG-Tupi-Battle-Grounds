//! Deterministic display offsets around authoritative one-dimensional lane anchors.

#[derive(Clone, Copy, Debug)]
pub struct Body {
    pub id: u32,
    pub anchor_x: f32,
    pub radius: f32,
    pub facing: f32,
    pub ghost: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Offset {
    pub dx: f32,
    pub dz: f32,
    pub vel_z: f32,
    pub home: f32,
    pub init: bool,
}

#[derive(Clone, Copy, Debug)]
pub struct Params {
    pub half_width: f32,
    pub max_dx: f32,
    pub max_back: f32,
    pub x_share: f32,
    pub home_k: f32,
    pub damping: f32,
    pub iterations: u32,
    pub rank_share: f32,
}

impl Default for Params {
    fn default() -> Self {
        Self {
            half_width: 1.1,
            max_dx: 0.15,
            max_back: 0.7,
            x_share: 0.5,
            home_k: 6.0,
            damping: 9.0,
            iterations: 3,
            rank_share: 0.8,
        }
    }
}

pub fn home_lane(id: u32) -> f32 {
    let h = id.wrapping_mul(0x9E37_79B9) >> 16;
    (h % 1000) as f32 / 1000.0 * 1.2 - 0.6
}

#[cfg(test)]
pub fn solve(bodies: &[Body], state: &mut [Offset], dt: f32, p: &Params) {
    Solver::default().solve(bodies, state, dt, p);
}

/// Scratch storage is retained across frames, including sort indices.
#[derive(Default)]
pub struct Solver {
    order: Vec<usize>,
}

impl Solver {
    pub fn solve(&mut self, bodies: &[Body], state: &mut [Offset], dt: f32, p: &Params) {
        let order = &mut self.order;
        assert_eq!(bodies.len(), state.len());
        let dt = dt.clamp(0.0, 1.0 / 30.0);
        for (b, o) in bodies.iter().zip(state.iter_mut()) {
            if !o.init {
                *o = Offset {
                    home: home_lane(b.id),
                    dz: home_lane(b.id) * p.half_width,
                    init: true,
                    ..*o
                };
            }
            let target = o.home * p.half_width;
            o.vel_z += (target - o.dz) * p.home_k * dt;
            o.vel_z *= (-p.damping * dt).exp();
            o.dz += o.vel_z * dt;
            o.dx *= (-p.home_k * dt).exp();
        }

        order.clear();
        order.extend(0..bodies.len());
        let max_r = bodies.iter().map(|b| b.radius).fold(0.0, f32::max);
        for _ in 0..p.iterations {
            let px = |s: &[Offset], i: usize| bodies[i].anchor_x + s[i].dx;
            order.sort_unstable_by(|&a, &b| {
                px(state, a)
                    .total_cmp(&px(state, b))
                    .then(bodies[a].id.cmp(&bodies[b].id))
            });
            for oi in 0..order.len() {
                let i = order[oi];
                for &j in &order[oi + 1..] {
                    if px(state, j) - px(state, i) > bodies[i].radius + max_r {
                        break; // ninguém mais à frente pode encostar em i
                    }
                    if bodies[i].ghost || bodies[j].ghost {
                        continue;
                    }
                    separate(bodies, state, i, j, p);
                }
            }
        }
        for (b, o) in bodies.iter().zip(state.iter_mut()) {
            let ahead = (o.dx * b.facing).clamp(-p.max_back, p.max_dx);
            o.dx = ahead * b.facing;
            o.dz = o.dz.clamp(-p.half_width, p.half_width);
        }
    }
}

fn separate(bodies: &[Body], state: &mut [Offset], i: usize, j: usize, p: &Params) {
    let (bi, bj) = (bodies[i], bodies[j]);
    let ex = (bj.anchor_x + state[j].dx) - (bi.anchor_x + state[i].dx);
    let ez = state[j].dz - state[i].dz;
    let min = bi.radius + bj.radius;
    let d2 = ex * ex + ez * ez;
    if d2 >= min * min {
        return;
    }
    let d = d2.sqrt();
    let (nx, nz) = if d > 1e-4 {
        (ex / d, ez / d)
    } else if bi.id < bj.id {
        (0.0, 1.0)
    } else {
        (0.0, -1.0)
    };
    let push = (min - d) * 0.5;
    let (mi, mj) = (bi.radius * bi.radius, bj.radius * bj.radius);
    let (wi, wj) = (mj / (mi + mj), mi / (mi + mj));
    let side = if nz > 0.0 || (nz == 0.0 && bi.id < bj.id) {
        1.0
    } else {
        -1.0
    };
    let lat = side * nz.abs().max(0.6);
    state[i].dx -= nx * push * p.x_share * 2.0 * wi;
    state[j].dx += nx * push * p.x_share * 2.0 * wj;
    state[i].dz -= lat * push * 2.0 * wi;
    state[j].dz += lat * push * 2.0 * wj;
    if nx.abs() < 0.3 {
        let back = if bi.id > bj.id { i } else { j };
        state[back].dx -= bodies[back].facing * push * p.rank_share;
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn large_frame_delta_is_bounded() {
        let b = [body(1, 4.0), body(2, 4.0)];
        let mut a = vec![Offset::default(); 2];
        let mut c = a.clone();
        solve(&b, &mut a, 2.0, &Params::default());
        solve(&b, &mut c, 1.0 / 30.0, &Params::default());
        assert_eq!(a, c);
    }

    #[test]
    fn opposing_fronts_respect_authoritative_bounds() {
        let b: Vec<_> = (1..=20)
            .map(|id| Body {
                facing: if id % 2 == 0 { 1.0 } else { -1.0 },
                anchor_x: if id % 2 == 0 { 4.0 } else { 4.8 },
                ..body(id, 0.0)
            })
            .collect();
        let s = run(&b, 240);
        let p = Params::default();
        for (b, s) in b.iter().zip(s) {
            assert!((-p.max_back..=p.max_dx).contains(&(s.dx * b.facing)));
        }
    }

    use super::*;

    fn run(bodies: &[Body], frames: u32) -> Vec<Offset> {
        let mut s = vec![Offset::default(); bodies.len()];
        for _ in 0..frames {
            solve(bodies, &mut s, 1.0 / 60.0, &Params::default());
        }
        s
    }

    fn body(id: u32, x: f32) -> Body {
        Body {
            id,
            anchor_x: x,
            radius: 0.4,
            facing: 1.0,
            ghost: false,
        }
    }

    fn gap(b: &[Body], s: &[Offset], i: usize, j: usize) -> f32 {
        let dx = (b[j].anchor_x + s[j].dx) - (b[i].anchor_x + s[i].dx);
        let dz = s[j].dz - s[i].dz;
        (dx * dx + dz * dz).sqrt()
    }

    #[test]
    fn stacked_units_spread_apart() {
        let b: Vec<Body> = (1..=5).map(|id| body(id * 3, 4.0)).collect();
        let s = run(&b, 240);
        for i in 0..b.len() {
            for j in i + 1..b.len() {
                assert!(
                    gap(&b, &s, i, j) > 0.8 * 0.9,
                    "{i},{j}: {}",
                    gap(&b, &s, i, j)
                );
            }
        }
    }

    #[test]
    fn drawing_never_strays_from_server_x() {
        let b: Vec<Body> = (1..=12).map(|id| body(id, 4.0)).collect();
        let s = run(&b, 240);
        let p = Params::default();
        assert!(s
            .iter()
            .all(|o| o.dx <= p.max_dx + 1e-6 && o.dx >= -p.max_back - 1e-6));
    }

    #[test]
    fn ghost_may_overlap() {
        let mut b = vec![body(1, 4.0), body(4, 4.0)];
        b[1].ghost = true;
        b[0].radius = 0.4;
        let s = run(&b, 240);
        assert!(s[1].dx.abs() < 1e-3, "ghost must not be pushed on x");
    }

    #[test]
    fn overtaking_walks_around_instead_of_through() {
        let mut s = vec![Offset::default(); 2];
        let mut worst = f32::MAX;
        for f in 0..300 {
            let t = f as f32 / 60.0;
            let b = [body(4, 2.0 + 0.8 * t), body(7, 0.5 + 2.4 * t)];
            solve(&b, &mut s, 1.0 / 60.0, &Params::default());
            worst = worst.min(gap(&b, &s, 0, 1));
        }
        assert!(worst > 0.8 * 0.85, "minimum gap {worst}");
    }

    #[test]
    fn deterministic_for_spectators() {
        let b: Vec<Body> = (1..=8).map(|id| body(id, 4.0 + id as f32 * 0.05)).collect();
        assert_eq!(run(&b, 120), run(&b, 120));
    }
}
