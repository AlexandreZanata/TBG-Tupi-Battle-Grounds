//! Original cosmetic pushing around authoritative anchors in normal play and spectating.

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
        let r = unit_radius(look.kind, look.tier, look.kind.is_summon());
        Self {
            authoritative: false,
            pos: pos.with_z(0.0),
            body: crowd::Body {
                id: look.id,
                anchor_x: pos.x,
                radius: r,
                half_length: if look.kind == UnitKind::Bonehorn {
                    1.35
                } else {
                    r
                },
                facing: look.seat.dir() as f32,
                ghost: false,
            },
        }
    }
    pub fn from_view(u: &UnitView, _you: Seat, physical: bool) -> Self {
        let r = unit_radius(u.kind, u.tier, u.is_summon());
        Self {
            authoritative: physical,
            pos: Vec3::ZERO,
            body: crowd::Body {
                id: u.id,
                anchor_x: 0.0,
                radius: r,
                half_length: if u.kind == UnitKind::Bonehorn {
                    1.35 * (1.0 + f32::from(u.tier) * 0.08)
                } else {
                    r
                },
                facing: u.seat.dir() as f32,
                ghost: u.fx & (1 << 14) != 0,
            },
        }
    }
}

fn unit_radius(kind: UnitKind, tier: u8, summon: bool) -> f32 {
    crate::art::kits::completion::crowd_radius(kind, tier)
        .unwrap_or_else(|| radius(tbg_core::content::def(kind).archetype, summon))
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

pub(crate) fn live_body(
    base: crowd::Body,
    anchor_x: f32,
    ghost: bool,
    (kind, tier, branch): (UnitKind, u8, u8),
    form: &crate::art::FigureForm,
    yaw: f32,
    engaged: bool,
) -> crowd::Body {
    let mut body = crowd::Body {
        anchor_x,
        ghost,
        ..base
    };
    let scale = 1.0 + 0.08 * f32::from(tier.min(2));
    match kind {
        UnitKind::Bonehorn => {
            let depth = form.occupancy;
            body.radius = body.radius.max(form.body_radius);
            body.anchor_x += (depth.x + depth.y) * 0.5 * body.facing;
            body.half_length = (depth.y - depth.x) * 0.5;
        }
        UnitKind::Shatterback if engaged => {
            let lunge = crate::art::melee::strike_profile_for_unit(kind, tier, branch).lunge;
            body.include_envelope(
                form.occupancy.x,
                form.occupancy.y + lunge,
                form.body_radius,
                yaw,
            );
        }
        UnitKind::Shatterback => {
            let r = 0.56 * scale;
            body.radius = body.radius.max(r);
            body.half_length = body.half_length.max(r);
        }
        UnitKind::Colossus if engaged => {
            body.include_envelope(form.occupancy.x, form.occupancy.y, form.body_radius, yaw);
        }
        UnitKind::Colossus => {
            let r = 0.64 * scale;
            body.radius = body.radius.max(r);
            body.half_length = body.half_length.max(r);
        }
        UnitKind::SiegeWalker => {
            let far = if engaged { 0.58 } else { 0.42 };
            body.include_envelope(-0.36 * scale, far * scale, 0.44 * scale, yaw);
        }
        _ => {}
    }
    body
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
        let (tier, branch) = visual.form();
        let form = library.get(visual.kind(), tier, branch);
        let body = live_body(
            anchor.body,
            anchor.pos.x,
            is_ghost(visual, anchor),
            (visual.kind(), tier, branch),
            form,
            aim.map_or(visual.yaw, |aim| aim.current_yaw),
            aim.is_some(),
        );
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::art::{kits, native_form, overhead};

    fn make_live_body(kind: UnitKind, id: u32, x: f32, engaged: bool) -> crowd::Body {
        let mut meshes = Assets::<Mesh>::default();
        let mut form = native_form((kits::kit(kind).unwrap().figure)(0, 0), &mut meshes);
        (form.overhead, form.occupancy, form.body_radius) =
            overhead::height(&form, &meshes, kind, 0, 0);
        let r = unit_radius(kind, 0, false);
        let base = crowd::Body {
            id,
            anchor_x: x,
            radius: r,
            half_length: r,
            facing: 1.0,
            ghost: false,
        };
        live_body(
            base,
            x,
            false,
            (kind, 0, 0),
            &form,
            65f32.to_radians(),
            engaged,
        )
    }

    #[test]
    fn tower_exit_non_contact_does_not_displace_ally_while_contact_pushes() {
        let params = crowd::Params::default();
        for kind in [
            UnitKind::Colossus,
            UnitKind::SiegeWalker,
            UnitKind::GustDrummer,
            UnitKind::Shatterback,
            UnitKind::BasaltBulwark,
            UnitKind::TrenchFist,
            UnitKind::Trident,
        ] {
            let lead_alone = make_live_body(UnitKind::Shieldbearer, 1, 1.8, false);
            let mut solo_state = [crowd::Offset::default()];
            let mut solver = crowd::Solver::default();
            for _ in 0..30 {
                solver.solve(&[lead_alone], &mut solo_state, 1.0 / 60.0, &params);
            }

            // Spawn `kind` at the tower exit (x = 0.5) while the ally is ahead at x = 1.8 (no visual contact).
            let exiting = make_live_body(kind, 3, 0.5, false);
            let mut pair_non_contact = [solo_state[0], crowd::Offset::default()];
            let mut baseline = [solo_state[0]];
            for _ in 0..30 {
                solver.solve(&[lead_alone], &mut baseline, 1.0 / 60.0, &params);
                solver.solve(
                    &[lead_alone, exiting],
                    &mut pair_non_contact,
                    1.0 / 60.0,
                    &params,
                );
            }
            assert!(
                (pair_non_contact[0].dz - baseline[0].dz).abs() < 1e-5
                    && (pair_non_contact[0].dx - baseline[0].dx).abs() < 1e-5,
                "{kind:?} at tower exit (x=0.5) displaced non-contact ally at x=1.8: {:?} vs {:?}",
                pair_non_contact[0],
                baseline[0]
            );

            // When the two units actually make contact at the tower exit (x = 0.5 and x = 0.65), they push laterally.
            let lead_contact = make_live_body(UnitKind::Shieldbearer, 1, 0.65, false);
            let mut pair_contact = [solo_state[0], crowd::Offset::default()];
            for _ in 0..30 {
                solver.solve(
                    &[lead_contact, exiting],
                    &mut pair_contact,
                    1.0 / 60.0,
                    &params,
                );
            }
            assert!(
                (pair_contact[0].dz - baseline[0].dz).abs() > 0.08,
                "{kind:?} in real contact failed to push ally laterally: dz={}",
                pair_contact[0].dz
            );
        }
    }
}
