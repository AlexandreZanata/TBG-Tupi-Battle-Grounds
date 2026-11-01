//! Weaver's alternating four-leg groups, braced spinner cast and abdomen curl over fixed rig nodes.
mod model;
use super::{common::moves, tools, Anatomy, Attack, Gait, Kit, Moment};
use crate::{
    art::{
        anim::{Clip, Pose},
        rig::{
            Joint::*,
            Node::{Body, Prop, Secondary},
            Socket, Spring,
        },
    },
    vfx::{impacts::Burst, shots::ShotKind},
};
use bevy::prelude::*;
static ANATOMY: Anatomy = Anatomy {
    rest: model::rest,
    parent: model::parent,
    springs: false,
    planted_legs: false,
};
pub const KIT: Kit = Kit {
    anatomy: Some(&ANATOMY),
    figure: model::spec,
    pose,
    gait: Gait::LIGHT,
    attack: Attack::Shot {
        from: Prop(Socket::WeaponR),
        muzzle: |_, _| Vec3::new(0.0, 0.0, 0.14),
        shot: |_, _| ShotKind::WebSpit,
        flash: |_, _| None,
        loaded: None,
    },
    evolve: Burst::Bloom,
    durations: moves::DURATIONS,
};
fn pose(m: &Moment, p: &mut Pose) {
    // The eight low legs own their gait; humanoid pelvis/arm tracks would detach them.
    for (j, _, _) in model::LEGS {
        p[Body(j).index()] = (Vec3::ZERO, Vec3::ZERO);
    }
    p[Body(Hips).index()] = (Vec3::ZERO, Vec3::ZERO);
    p[Body(Chest).index()] = (Vec3::ZERO, Vec3::ZERO);
    p[Body(Head).index()] = (Vec3::ZERO, Vec3::ZERO);
    let wave = (m.clock * 1.6).sin();
    tools::shift(p, Chest, [0.0, 0.008 * wave, 0.0]);
    match m.clip {
        Clip::Walk => {
            let phase = m.walk.unwrap_or(0.0) * std::f32::consts::TAU;
            for (i, (j, s, _)) in model::LEGS.iter().enumerate() {
                let k = (phase
                    + if i % 2 == 0 {
                        0.0
                    } else {
                        std::f32::consts::PI
                    })
                .sin();
                tools::set(p, *j, [0.0, s * k * 18.0, 0.0]);
                tools::shift(p, *j, [0.0, k.max(0.0) * 0.045, 0.0]);
            }
        }
        Clip::AttackMelee | Clip::AttackRanged | Clip::Cast => cast(m.t, p),
        Clip::Hit => tools::shift(
            p,
            Hips,
            [0.0, -0.04 * tools::hold(m.t, [0.0, 0.05, 0.1, 0.25]), -0.02],
        ),
        Clip::Death => {
            let k = tools::window(m.t, 0.0, 0.7);
            tools::shift(p, Hips, [0.0, -0.18 * k, 0.0]);
            for (j, s, _) in model::LEGS {
                tools::set(p, j, [0.0, 0.0, -s * 55.0 * k]);
            }
        }
        Clip::Evolve => {
            tools::shift(
                p,
                Chest,
                [0.0, 0.08 * tools::hold(m.t, [0.0, 0.25, 0.4, 0.9]), 0.0],
            );
        }
        Clip::Spawn => tools::shift(
            p,
            Hips,
            [0.0, -0.20 * (1.0 - tools::window(m.t, 0.0, 0.5)), 0.0],
        ),
        Clip::Victory => tools::set(p, Head, [-12.0, 0.0, 0.0]),
        Clip::Idle => {}
    }
    if let Some(t) = m.special {
        cast(t * 0.62, p);
    }
    p[Secondary(Spring::Back).index()].0.x = wave * 2.0;
}
fn cast(t: f32, p: &mut Pose) {
    let k = tools::hold(t, [0.0, 0.19, 0.22, 0.48]);
    tools::set(p, Chest, [-15.0 * k, 0.0, 0.0]);
    tools::set(p, Head, [8.0 * k, 0.0, 0.0]);
    tools::shift(p, Head, [0.0, 0.02 * k, 0.025 * k]);
}
