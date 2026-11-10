//! Plays rigid Blender precinct tracks from the authoritative auxiliary lifecycle.

use super::{animate, AuxPrecinctRoot, AuxiliarySceneState, SlotAnimState};
use crate::arena::blender;
use bevy::prelude::*;
use tbg_core::auxiliary::{AuxiliaryKind, AuxiliaryPhase};

const SAMPLE_RATE: f32 = 8.0;

#[derive(Component)]
pub(super) struct Track {
    delivery: bool,
    keyframes: Vec<(u16, Transform)>,
}

pub(super) fn populate(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    material: &Handle<StandardMaterial>,
    root: Entity,
    slot: AuxPrecinctRoot,
    data: &[u8],
) -> bool {
    let Some(model) = blender::Model::read(data) else {
        error!("Invalid Blender precinct pack: {:?}", slot);
        return false;
    };
    let mut at = Transform::from_translation(model.origin);
    at.scale.x = super::placement::mirror(slot.seat);
    commands.entity(root).insert(at);
    for batch in model.batches {
        commands.spawn((
            Mesh3d(meshes.add(batch.mesh)),
            MeshMaterial3d(material.clone()),
            batch.track[0].1,
            Track {
                delivery: batch.name.starts_with("delivery"),
                keyframes: batch.track,
            },
            ChildOf(root),
        ));
    }
    true
}

fn sample(keyframes: &[(u16, Transform)], seconds: f32) -> Transform {
    let tick = seconds * SAMPLE_RATE;
    let next = keyframes.partition_point(|(frame, _)| f32::from(*frame) <= tick);
    let (a, from) = keyframes[next.saturating_sub(1)];
    let Some(&(b, to)) = keyframes.get(next) else {
        return from;
    };
    let alpha = (tick - f32::from(a)) / f32::from(b - a);
    Transform {
        translation: from.translation.lerp(to.translation, alpha),
        rotation: from.rotation.slerp(to.rotation, alpha).normalize(),
        scale: from.scale.lerp(to.scale, alpha),
    }
}

fn miner_time(anim: &SlotAnimState, clock: f32, reduced: bool) -> f32 {
    if matches!(anim.phase, AuxiliaryPhase::Preparing { .. }) {
        return 12.0 * animate::build_progress(anim, 0);
    }
    let (reveal, work, period) = match anim.tier {
        2 => (17.0, 18.1, 1.1),
        3 => (22.0, 23.8, 0.9),
        _ => (12.0, 13.375, 1.35),
    };
    if anim.tier > 1 && anim.tier_since < 1.05 && !reduced {
        return reveal + anim.tier_since;
    }
    if anim.phase == AuxiliaryPhase::Completed || reduced {
        return work;
    }
    work + clock % period
}

fn merchant_time(anim: &SlotAnimState, clock: f32, reduced: bool, delivery_track: bool) -> f32 {
    let build = animate::build_progress(anim, 1);
    if build < 1.0 {
        return build * 3.25;
    }
    let (reveal, idle, delivery) = match anim.tier {
        2 => (10.0, 11.2, 13.0),
        3 => (19.0, 20.2, 23.1),
        _ => (3.25, 3.6, 5.2),
    };
    if delivery_track {
        if !reduced && anim.delivery_timer > 0.0 {
            return delivery + animate::DELIVERY_SECS - anim.delivery_timer;
        }
        return idle;
    }
    if anim.tier > 1 && anim.tier_since < 1.0 && !reduced {
        return reveal + anim.tier_since;
    }
    idle + if reduced { 0.0 } else { clock % 0.8 }
}

pub(super) fn animate(
    state: Res<AuxiliarySceneState>,
    roots: Query<&AuxPrecinctRoot>,
    mut tracks: Query<(&Track, &ChildOf, &mut Transform)>,
) {
    let reduced = crate::config::low_motion();
    for (track, parent, mut transform) in &mut tracks {
        let Ok(root) = roots.get(parent.parent()) else {
            continue;
        };
        let anim = &state.slot_anim[root.seat.index()][usize::from(root.kind.slot())];
        if anim.phase == AuxiliaryPhase::Available {
            continue;
        }
        let clock = state.animation_time;
        let seconds = match root.kind {
            AuxiliaryKind::Miner => miner_time(anim, clock, reduced),
            AuxiliaryKind::Merchant => merchant_time(anim, clock, reduced, track.delivery),
        };
        transform.set_if_neq(sample(&track.keyframes, seconds));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn construction_is_driven_by_miner_preparation_and_merchant_delivery_is_event_driven() {
        let miner = SlotAnimState {
            phase: AuxiliaryPhase::Preparing {
                ticks_left: tbg_core::auxiliary::AUXILIARY_RULE.miner_prep_ticks / 2,
            },
            ..default()
        };
        assert_eq!(miner_time(&miner, 900.0, false), 6.0);
        let mut merchant = SlotAnimState {
            since_hire: 10.0,
            tier_since: 10.0,
            tier: 3,
            ..default()
        };
        assert_eq!(merchant_time(&merchant, 5.2, false, true), 20.2);
        merchant.delivery_timer = animate::DELIVERY_SECS;
        assert_eq!(merchant_time(&merchant, 0.0, false, true), 23.1);
        assert_eq!(merchant_time(&merchant, 0.0, true, true), 20.2);
    }
}
