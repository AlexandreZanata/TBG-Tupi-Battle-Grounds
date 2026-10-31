//! Microdrill's tick shell, Duodrill's paired housings and Megadrill's broad three-bladed siege body.
use super::super::{
    common::{body, moves},
    tools, Anatomy, Attack, Gait, Kit, Moment,
};
use crate::{
    art::{
        anim::{Clip, Pose},
        figure::{part, FigureSpec},
        mesh::Shape::*,
        palette::*,
        rig::{
            self,
            Joint::*,
            Node,
            Node::{Body, Prop, Secondary},
            Proportions, Socket, Spring,
        },
    },
    vfx::impacts::{Burst, Impact},
};
use bevy::prelude::*;
fn parent(n: Node) -> Option<Node> {
    match n {
        Body(Chest) | Body(Head) => Some(Body(Hips)),
        Prop(Socket::WeaponR | Socket::OffhandL | Socket::Back) => Some(Body(Chest)),
        Secondary(_) => Some(Body(Hips)),
        _ => n.parent(),
    }
}
fn rest(rank: u8, n: Node, p: &Proportions) -> Vec3 {
    match n {
        Body(Hips) => Vec3::new(0.0, 0.15, 0.0),
        Body(Chest) => Vec3::ZERO,
        Body(Head) => Vec3::new(0.0, 0.05, 0.0),
        Prop(socket @ (Socket::WeaponR | Socket::OffhandL | Socket::Back)) => {
            let index = match socket {
                Socket::WeaponR => 0,
                Socket::OffhandL => 1,
                _ => 2,
            };
            Vec3::new(centers(rank).get(index).copied().unwrap_or(0.0), 0.0, 0.10)
        }
        Secondary(_) => Vec3::ZERO,
        _ => rig::rest(n, p),
    }
}
fn centers(rank: u8) -> &'static [f32] {
    match rank {
        1 => &[0.0],
        2 => &[-0.10, 0.10],
        _ => &[-0.18, 0.0, 0.18],
    }
}
static MICRO_ANATOMY: Anatomy = Anatomy {
    rest: |n, p| rest(1, n, p),
    parent,
    springs: false,
    planted_legs: false,
};
static DUO_ANATOMY: Anatomy = Anatomy {
    rest: |n, p| rest(2, n, p),
    parent,
    springs: false,
    planted_legs: false,
};
static MEGA_ANATOMY: Anatomy = Anatomy {
    rest: |n, p| rest(3, n, p),
    parent,
    springs: false,
    planted_legs: false,
};
fn spec(rank: u8, t: u8, b: u8) -> FigureSpec {
    let p = body::plan(false, 0.50, 0);
    let width = 0.10 + rank as f32 * 0.07;
    let mut v = Vec::new();
    let color = if b == 1 && t > 0 { CHITIN } else { IRON };
    for (index, &x) in centers(rank).iter().enumerate() {
        let head = Prop([Socket::WeaponR, Socket::OffhandL, Socket::Back][index]);
        v.push(part(
            Body(Chest),
            Box(0.16, 0.14, 0.24),
            color,
            [x, 0.0, 0.0],
        ));
        v.push(
            part(
                head,
                Cone(0.055, if rank == 3 { 0.26 } else { 0.18 }),
                BRASS,
                [0.0, 0.0, if rank == 3 { 0.19 } else { 0.15 }],
            )
            .rot([90.0, 0.0, 0.0]),
        );
        for z in [0.08, 0.14] {
            v.push(
                part(head, Ring(0.008, 0.045, 360.0), IRON, [0.0, 0.0, z]).rot([90.0, 0.0, 0.0]),
            );
        }
    }
    v.push(part(
        Body(Chest),
        Box(width * 2.0, 0.035, 0.12),
        ALLY,
        [0.0, 0.085, -0.02],
    ));
    v.push(part(
        Body(Chest),
        Box(width * 2.0, 0.065, 0.12),
        IRON,
        [0.0, -0.02, -0.045],
    ));
    for s in [-1.0, 1.0] {
        for z in [-0.10, 0.07] {
            // Overlapping struts keep each animated leg group visibly joined to its housing.
            v.push(part(
                Secondary(if s < 0.0 {
                    Spring::Tabard
                } else {
                    Spring::HeadGear
                }),
                Box(0.18, 0.025, 0.025),
                IRON,
                [s * width * 0.65, -0.065, z],
            ));
            v.push(
                part(
                    Secondary(if s < 0.0 {
                        Spring::Tabard
                    } else {
                        Spring::HeadGear
                    }),
                    Box(0.13, 0.025, 0.025),
                    CHITIN,
                    [s * (width + 0.025), -0.07, z],
                )
                .rot([0.0, s * 25.0, s * 30.0]),
            );
        }
        if rank == 3 {
            v.push(part(
                Body(Chest),
                Box(0.06, 0.10, 0.22),
                BRASS,
                [s * (width + 0.02), 0.02, -0.02],
            ));
        }
    }
    if t == 2 && b == 0 {
        v.push(part(
            Body(Chest),
            Box(width, 0.045, 0.05),
            IVORY,
            [0.0, 0.1225, -0.04],
        ));
    }
    FigureSpec { plan: p, parts: v }
}
fn pose(rank: u8, m: &Moment, p: &mut Pose) {
    for j in [Hips, Chest, Head] {
        p[Body(j).index()] = (Vec3::ZERO, Vec3::ZERO);
    }
    let k = m.walk.map_or(0.0, |t| (t * std::f32::consts::TAU).sin());
    for (s, n) in [(-1.0, Spring::Tabard), (1.0, Spring::HeadGear)] {
        p[Secondary(n).index()].0.y = s * k * 12.0;
    }
    for socket in [Socket::WeaponR, Socket::OffhandL, Socket::Back] {
        p[Prop(socket).index()].0.z = (m.clock * (540.0 / rank as f32)).rem_euclid(360.0);
    }
    if matches!(m.clip, Clip::AttackMelee | Clip::Cast | Clip::AttackRanged) {
        let k = tools::hold(m.t, [0.0, 0.22, 0.30, 0.52]);
        tools::shift(p, Chest, [0.0, -0.02 * k, 0.07 * k]);
    }
    if m.clip == Clip::Death {
        let k = tools::window(m.t, 0.0, 0.6);
        tools::set(p, Hips, [0.0, 0.0, 45.0 * k]);
        tools::shift(p, Hips, [0.0, -0.10 * k, 0.0]);
    }
}

pub const MICRO: Kit = Kit {
    anatomy: Some(&MICRO_ANATOMY),
    figure: |t, b| spec(1, t, b),
    pose: |m, p| pose(1, m, p),
    gait: Gait::LIGHT,
    attack: Attack::Melee {
        from: Prop(Socket::WeaponR),
        tip: |_, _| Vec3::new(0.0, 0.0, 0.24),
        impact: |_, _| Impact::Sparks,
    },
    evolve: Burst::Forge,
    durations: moves::DURATIONS,
};

pub const DUO: Kit = Kit {
    anatomy: Some(&DUO_ANATOMY),
    figure: |t, b| spec(2, t, b),
    pose: |m, p| pose(2, m, p),
    gait: Gait::LIGHT,
    attack: Attack::Melee {
        from: Prop(Socket::WeaponR),
        tip: |_, _| Vec3::new(0.0, 0.0, 0.24),
        impact: |_, _| Impact::Sparks,
    },
    evolve: Burst::Forge,
    durations: moves::DURATIONS,
};

pub const MEGA: Kit = Kit {
    anatomy: Some(&MEGA_ANATOMY),
    figure: |t, b| spec(3, t, b),
    pose: |m, p| pose(3, m, p),
    gait: Gait::LIGHT,
    attack: Attack::Melee {
        from: Prop(Socket::WeaponR),
        tip: |_, _| Vec3::new(0.0, 0.0, 0.32),
        impact: |_, _| Impact::Sparks,
    },
    evolve: Burst::Forge,
    durations: moves::DURATIONS,
};
