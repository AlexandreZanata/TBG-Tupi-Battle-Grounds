//! Opt-in native witness of authoritative width reservations and stationary root stability.
use bevy::prelude::*;

#[derive(Resource, Default)]
pub(super) struct Status {
    pub ready: bool,
    frames: u32,
    positions: Vec<(u32, Vec3)>,
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
    assert!(
        !state.deployments.is_empty(),
        "PK-01 witness needs real overflow"
    );
    let mut positions = Vec::new();
    for unit in &state.units {
        let Some(entity) = entities.0.get(&unit.id) else {
            return;
        };
        let Ok(root) = roots.get(*entity) else { return };
        let expected = Vec3::new(
            unit.x as f32 / 1000.,
            0.,
            tbg_core::occupancy::lateral(unit) as f32 / 1000.,
        );
        if root.translation.distance(expected) > 0.015 {
            status.frames = 0;
            return;
        }
        positions.push((unit.id, root.translation));
    }
    positions.sort_unstable_by_key(|(id, _)| *id);
    if positions != status.positions {
        status.positions = positions;
        status.frames = 0;
        return;
    }
    status.frames += 1;
    if status.frames >= 20 {
        status.ready = true;
        eprintln!("PK-01 native witness passed: active={}, queued={}, stationary_frames=20, root_error<0.015, no_cosmetic_repulsion=true", state.units.len(), state.deployments.len());
    }
}
