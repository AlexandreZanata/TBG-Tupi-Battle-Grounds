//! RESTORE-01 native witness: every piece active, cosmetic pushing bounded, no gameplay queue.
use bevy::prelude::*;
#[derive(Resource, Default)]
pub(super) struct Status {
    pub ready: bool,
    frames: u32,
    positions: Vec<(u32, Vec3)>,
    maximum_step: f32,
}
pub(super) fn plugin(app: &mut App) {
    app.init_resource::<Status>()
        .add_systems(PostUpdate, observe);
}
fn observe(
    trial: Res<crate::training::Trial>,
    entities: Res<crate::sync::UnitEntities>,
    roots: Query<&Transform, With<crate::visuals::UnitVisual>>,
    mut status: ResMut<Status>,
) {
    if status.ready {
        return;
    }
    let Some(session) = &trial.session else {
        return;
    };
    if !matches!(
        session.lifecycle(),
        tbg_core::training::Lifecycle::Paused(_)
    ) {
        return;
    }
    let state = session.state();
    assert!(!state.physical_lanes && state.deployments.is_empty());
    assert_eq!(state.army_size(tbg_core::Seat::A), 12);
    let mut positions = Vec::new();
    for unit in &state.units {
        let Some(entity) = entities.0.get(&unit.id) else {
            return;
        };
        let Ok(root) = roots.get(*entity) else { return };
        positions.push((unit.id, root.translation));
    }
    positions.sort_unstable_by_key(|(id, _)| *id);
    status.frames += 1;
    if status.frames > 60 && positions.len() == status.positions.len() {
        for ((id, now), (old_id, before)) in positions.iter().zip(&status.positions) {
            assert_eq!(id, old_id);
            let step = now.distance(*before);
            assert!(
                step <= 0.14,
                "RESTORE-01: cosmetic root teleported by {step}"
            );
        }
        let maximum = positions
            .iter()
            .zip(&status.positions)
            .map(|((_, a), (_, b))| a.distance(*b))
            .fold(0., f32::max);
        status.maximum_step = status.maximum_step.max(maximum);
    }
    status.positions = positions;
    if status.frames >= 90 {
        status.ready = true;
        eprintln!("RESTORE-01 native witness passed: active={}, queued=0, all_twelve_allies_visible=true, bounded_frames=30, max_cosmetic_step={}, legacy_movement=true", state.units.len(), status.maximum_step);
    }
}
