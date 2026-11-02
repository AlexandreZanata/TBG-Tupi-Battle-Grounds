//! Legacy-entry native witness: every piece active with no gameplay queue.
use bevy::prelude::*;
#[derive(Resource, Default)]
pub(super) struct Status {
    pub ready: bool,
    frames: u32,
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
    if status.frames >= 90 {
        status.ready = true;
        eprintln!("Legacy-entry native witness passed: active={}, queued=0, all_twelve_allies_spawned=true, legacy_movement=true", state.units.len());
    }
}
