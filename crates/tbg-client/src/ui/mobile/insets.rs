//! Inset the controls while the Android battle surface draws the arena to every edge.

use super::layout::Metrics;
use bevy::prelude::*;

#[derive(Component)]
pub struct SafeFrame;

pub fn apply(
    mut commands: Commands,
    window: Single<&Window>,
    metrics: Res<Metrics>,
    mut frame: Query<(Entity, &mut Node), With<SafeFrame>>,
    roots: Query<
        Entity,
        (
            With<Node>,
            Without<ChildOf>,
            Without<SafeFrame>,
            Without<FullBleed>,
        ),
    >,
) {
    let safe = metrics.safe;
    let density = if metrics.density > 0. {
        metrics.density
    } else {
        window.scale_factor()
    };
    let node = Node {
        position_type: PositionType::Absolute,
        left: px(safe.x),
        top: px(safe.y),
        width: px((window.physical_width() as f32 / density - safe.x - safe.z).max(0.)),
        height: px((window.physical_height() as f32 / density - safe.y - safe.w).max(0.)),
        overflow: Overflow::clip(),
        ..default()
    };
    let entity = if let Ok((entity, mut current)) = frame.single_mut() {
        if *current != node {
            *current = node;
        }
        entity
    } else {
        commands.spawn((node, SafeFrame)).id()
    };
    for root in &roots {
        commands.entity(entity).add_child(root);
    }
}
