//! Follow authoritative live/spectator roots; retain cosmetic offsets only in stress tools.

use bevy::prelude::*;
use tbg_core::{Archetype, Seat, UnitKind, UnitView};

use crate::{art::aim::Aim, config, crowd, visuals::UnitVisual};

#[derive(Component)]
pub(crate) struct LaneAnchor {
    pub authoritative: bool,
    pub pos: Vec3,
    pub(super) body: crowd::Body,
}

impl LaneAnchor {
    pub fn for_stress(look: &crate::visuals::UnitLook, pos: Vec3) -> Self {
        Self {
            authoritative: false,
            pos: pos.with_z(0.0),
            body: crowd::Body {
                id: look.id,
                anchor_x: pos.x,
                radius: crate::art::kits::completion::crowd_radius(look.kind, look.tier)
                    .unwrap_or_else(|| {
                        radius(
                            tbg_core::content::def(look.kind).archetype,
                            look.kind.is_summon(),
                        )
                    }),
                half_length: if look.kind == UnitKind::Bonehorn {
                    1.35
                } else {
                    radius(
                        tbg_core::content::def(look.kind).archetype,
                        look.kind.is_summon(),
                    )
                },
                facing: if look.own { 1.0 } else { -1.0 },
                ghost: false,
            },
        }
    }
    pub fn from_view(u: &UnitView, you: Seat, physical: bool) -> Self {
        Self {
            authoritative: physical,
            pos: Vec3::ZERO,
            body: crowd::Body {
                id: u.id,
                anchor_x: 0.0,
                radius: crate::art::kits::completion::crowd_radius(u.kind, u.tier).unwrap_or_else(
                    || radius(tbg_core::content::def(u.kind).archetype, u.is_summon()),
                ),
                half_length: if u.kind == UnitKind::Bonehorn {
                    1.35 * (1.0 + f32::from(u.tier) * 0.08)
                } else {
                    radius(tbg_core::content::def(u.kind).archetype, u.is_summon())
                },
                facing: if u.seat == you { 1.0 } else { -1.0 },
                ghost: u.fx & (1 << 14) != 0,
            },
        }
    }
}

fn radius(archetype: Archetype, summon: bool) -> f32 {
    if summon {
        return 0.28;
    }
    match archetype {
        Archetype::Tank => 0.50,
        Archetype::Dps | Archetype::Control => 0.40,
        Archetype::Ranged | Archetype::Support => 0.35,
    }
}

fn is_ghost(visual: &UnitVisual, anchor: &LaneAnchor) -> bool {
    visual.dying || anchor.body.ghost
}

#[derive(Component, Default)]
pub(crate) struct CrowdState(crowd::Offset);

#[derive(Default)]
pub(super) struct Scratch {
    solver: crowd::Solver,
    bodies: Vec<crowd::Body>,
    offsets: Vec<crowd::Offset>,
    enabled: Option<bool>,
}

pub(super) fn update(
    time: Res<Time>,
    mut scratch: Local<Scratch>,
    library: Res<crate::art::FigureLibrary>,
    mut units: Query<(
        &LaneAnchor,
        &mut CrowdState,
        &mut Transform,
        &UnitVisual,
        Option<&Aim>,
    )>,
) {
    // PK-01: live/spectator poses follow the shared authoritative timeline, without a second repulsion solver.
    if units.iter().all(|(a, _, _, _, _)| a.authoritative) {
        for (anchor, mut state, mut transform, _, _) in &mut units {
            state.0 = default();
            transform.translation = anchor.pos;
        }
        return;
    }
    let enabled = *scratch
        .enabled
        .get_or_insert_with(|| config::url_param("crowd").as_deref() != Some("0"));
    let Scratch {
        solver,
        bodies,
        offsets,
        ..
    } = &mut *scratch;
    bodies.clear();
    offsets.clear();
    for (anchor, state, _, visual, aim) in &units {
        let mut body = crowd::Body {
            anchor_x: anchor.pos.x,
            ghost: is_ghost(visual, anchor),
            ..anchor.body
        };
        if visual.kind() == UnitKind::Bonehorn {
            let (tier, branch) = visual.form();
            let form = library.get(visual.kind(), tier, branch);
            let depth = form.occupancy;
            body.radius = body.radius.max(form.body_radius);
            body.anchor_x += (depth.x + depth.y) * 0.5 * body.facing;
            body.half_length = (depth.y - depth.x) * 0.5;
        }
        if visual.kind() == UnitKind::Shatterback
            || matches!(visual.kind(), UnitKind::SiegeWalker | UnitKind::Trident)
            || crate::art::kits::ogres::contains(visual.kind())
            || crate::art::kits::abyssals::contains(visual.kind())
        {
            let (tier, branch) = visual.form();
            let form = library.get(visual.kind(), tier, branch);
            let lunge = if visual.kind() == UnitKind::Shatterback {
                crate::art::melee::strike_profile_for_unit(visual.kind(), tier, branch).lunge
            } else {
                0.0
            };
            body.include_envelope(
                form.occupancy.x,
                form.occupancy.y + lunge,
                form.body_radius,
                aim.map_or(visual.yaw, |aim| aim.current_yaw),
            );
        }
        bodies.push(body);
        offsets.push(state.0);
    }
    if enabled {
        solver.solve(
            bodies,
            offsets,
            time.delta_secs(),
            &crowd::Params::default(),
        );
    }
    for ((anchor, mut state, mut transform, _, _), offset) in units.iter_mut().zip(offsets.iter()) {
        state.0 = *offset;
        let delta = if enabled {
            Vec3::new(offset.dx, 0.0, offset.dz)
        } else {
            Vec3::new(0.0, 0.0, [-0.8, 0.0, 0.8][(anchor.body.id % 3) as usize])
        };
        transform.translation = anchor.pos + delta;
    }
}
