//! Touch scrolls the foremost viewport and hands excess motion only to its ancestors.

use super::input::Pointer;
use bevy::{
    input::mouse::{MouseScrollUnit, MouseWheel},
    prelude::*,
};

type Lists<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static Node,
        &'static ComputedNode,
        &'static UiGlobalTransform,
        &'static InheritedVisibility,
        &'static bevy::ui::ComputedStackIndex,
        Option<&'static bevy::ui::CalculatedClip>,
        &'static mut ScrollPosition,
    ),
>;
type Shields<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static ComputedNode,
        &'static UiGlobalTransform,
        &'static InheritedVisibility,
        &'static bevy::ui::ComputedStackIndex,
        Option<&'static bevy::ui::CalculatedClip>,
    ),
    With<crate::ui::UiAction>,
>;

pub fn scroll(
    mut events: MessageReader<MouseWheel>,
    pointer: Res<Pointer>,
    window: Single<&Window>,
    form: NonSend<crate::ui::textform::ActiveForm>,
    mut lists: Lists,
    shields: Shields,
    parents: Query<&ChildOf>,
) {
    if crate::ui::textform::is_open(&form) {
        events.clear();
        return;
    }
    let mut motions = Vec::new();
    if let Some((point, delta)) = pointer.drag {
        motions.push((
            point * window.scale_factor(),
            delta.y * window.scale_factor(),
        ));
    }
    if let Some(point) = window.cursor_position() {
        for event in events.read() {
            let dy = event.y
                * if event.unit == MouseScrollUnit::Line {
                    36.
                } else {
                    1.
                };
            motions.push((point * window.scale_factor(), dy * window.scale_factor()));
        }
    } else {
        events.clear();
    }
    for (point, delta) in motions {
        // A modal's shield cannot pass a drag into a background page.
        let shield = shields
            .iter()
            .filter(|(_, node, t, visible, _, clip)| {
                visible.get()
                    && node.contains_point(**t, point)
                    && clip.is_none_or(|c| c.contains_point(point))
            })
            .max_by_key(|(_, _, _, _, stack, _)| stack.0)
            .map(|(entity, _, _, _, stack, _)| (entity, stack.0));
        let target = lists
            .iter()
            .filter(|(entity, style, node, t, visible, stack, clip, _)| {
                style.overflow.y == Overflow::scroll_y().y
                    && visible.get()
                    && shield
                        .is_none_or(|(shield, z)| stack.0 >= z || owns(*entity, shield, &parents))
                    && node.content_size.y > node.size.y
                    && node.contains_point(**t, point)
                    && clip.is_none_or(|c| c.contains_point(point))
            })
            .max_by_key(|(_, _, _, _, _, stack, _, _)| stack.0)
            .map(|(entity, ..)| entity);
        chain(target, delta, &mut lists, &parents);
    }
}

fn chain(mut target: Option<Entity>, mut delta: f32, lists: &mut Lists, parents: &Query<&ChildOf>) {
    while let Some(entity) = target {
        if let Ok((_, style, node, _, _, _, _, mut position)) = lists.get_mut(entity) {
            if style.overflow.y == Overflow::scroll_y().y && node.inverse_scale_factor > 0. {
                let scale = node.inverse_scale_factor;
                let max = ((node.content_size.y - node.size.y) * scale).max(0.);
                let old = position.0.y;
                position.0.y = (old - delta * scale).clamp(0., max);
                delta -= (old - position.0.y) / scale;
            }
        }
        if delta.abs() < 0.1 {
            break;
        }
        target = parents.get(entity).ok().map(ChildOf::parent);
    }
}

// A list owns its buttons: dragging over one scrolls the list, while an unrelated modal shields it.
fn owns(ancestor: Entity, mut child: Entity, parents: &Query<&ChildOf>) -> bool {
    loop {
        if ancestor == child {
            return true;
        }
        let Ok(parent) = parents.get(child) else {
            return false;
        };
        child = parent.parent();
    }
}
