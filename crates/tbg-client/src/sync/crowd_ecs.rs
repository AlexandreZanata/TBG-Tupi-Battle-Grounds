//! Place live and spectator units using client-only crowd offsets.

use bevy::prelude::*;
use tbg_core::{Archetype, Seat, UnitView};

use crate::{config, crowd, visuals::UnitVisual};

#[derive(Component)]
pub(super) struct LaneAnchor {
    pub pos: Vec3,
    pub(super) body: crowd::Body,
}

impl LaneAnchor {
    pub fn from_view(u: &UnitView, you: Seat) -> Self {
        Self {
            pos: Vec3::ZERO,
            body: crowd::Body {
                id: u.id,
                anchor_x: 0.0,
                radius: radius(tbg_core::content::def(u.kind).archetype, u.is_summon()),
                facing: if u.seat == you { 1.0 } else { -1.0 },
                // The current view does not pack carrier state; see the phase report.
                ghost: false,
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
pub(super) struct CrowdState(crowd::Offset);

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
    mut units: Query<(&LaneAnchor, &mut CrowdState, &mut Transform, &UnitVisual)>,
) {
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
    for (anchor, state, _, visual) in &units {
        bodies.push(crowd::Body {
            anchor_x: anchor.pos.x,
            ghost: is_ghost(visual, anchor),
            ..anchor.body
        });
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
    for ((anchor, mut state, mut transform, _), offset) in units.iter_mut().zip(offsets.iter()) {
        state.0 = *offset;
        let delta = if enabled {
            Vec3::new(offset.dx, 0.0, offset.dz)
        } else {
            Vec3::new(0.0, 0.0, [-0.8, 0.0, 0.8][(anchor.body.id % 3) as usize])
        };
        transform.translation = anchor.pos + delta;
    }
}
