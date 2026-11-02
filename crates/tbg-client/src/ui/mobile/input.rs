//! One owned pointer distinguishes tap, inspection, scrolling and cancellation (BR-AN05).

use bevy::{input::touch::TouchPhase, prelude::*};
#[derive(Resource, Default)]
pub struct Pointer {
    owner: Option<u64>,
    active: std::collections::BTreeSet<u64>,
    start: Vec2,
    last: Vec2,
    started: f64,
    cancelled: bool,
    dragging: bool,
    pub taps: Vec<(Vec2, Vec2, bool)>,
    pub drag: Option<(Vec2, Vec2)>,
    pub touch_frame: bool,
    pub consumed: bool,
}
impl Pointer {
    pub fn event(&mut self, id: u64, phase: TouchPhase, point: Vec2, now: f64) {
        if phase == TouchPhase::Started {
            self.active.insert(id);
        }
        match phase {
            TouchPhase::Started if self.owner.is_none() && self.active.len() == 1 => {
                self.owner = Some(id);
                self.start = point;
                self.last = point;
                self.started = now;
                self.cancelled = false;
                self.dragging = false;
            }
            TouchPhase::Started => {
                self.cancelled = true;
                self.dragging = false;
                self.drag = None;
            }
            TouchPhase::Moved if self.owner == Some(id) => {
                let was_dragging = self.dragging;
                if !self.cancelled && point.distance(self.start) > 12. {
                    self.cancelled = true;
                    self.dragging = true;
                }
                if self.dragging {
                    // Android can deliver several moves between rendered frames.
                    // Preserve their full distance instead of only the final event.
                    let previous = self.drag.map_or(Vec2::ZERO, |(_, delta)| delta);
                    let from = if was_dragging { self.last } else { self.start };
                    self.drag = Some((point, previous + point - from));
                }
                self.last = point;
            }
            TouchPhase::Ended if self.owner == Some(id) => {
                if !self.cancelled && point.distance(self.start) <= 12. {
                    self.taps
                        .push((self.start, point, now - self.started >= 0.5));
                }
                self.owner = None;
            }
            TouchPhase::Canceled => {
                self.cancelled = true;
                self.dragging = false;
                self.drag = None;
                if self.owner == Some(id) {
                    self.owner = None;
                }
            }
            _ => {}
        }
        if matches!(phase, TouchPhase::Ended | TouchPhase::Canceled) {
            self.active.remove(&id);
        }
    }
}
pub fn collect(
    mut pointer: ResMut<Pointer>,
    mut events: MessageReader<TouchInput>,
    time: Res<Time<Real>>,
    window: Single<&Window>,
) {
    pointer.taps.clear();
    pointer.drag = None;
    pointer.touch_frame = false;
    pointer.consumed = false;
    if !window.focused {
        pointer.owner = None;
        pointer.active.clear();
        pointer.cancelled = true;
    }
    for event in events.read() {
        pointer.touch_frame = true;
        pointer.event(
            event.id,
            event.phase,
            event.position,
            time.elapsed_secs_f64(),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn deliberate_repeated_taps_are_never_cooled_down() {
        // BR-AN05: each deliberate tap produces exactly one action, including quick repeats.
        let mut p = Pointer::default();
        for n in 0..2 {
            p.event(n, TouchPhase::Started, Vec2::ZERO, 0.);
            p.event(n, TouchPhase::Ended, Vec2::ZERO, 0.05);
            assert!(p.taps.last().is_some_and(|tap| !tap.2));
            assert!(p.owner.is_none());
        }
        assert_eq!(p.taps.len(), 2);
    }
    #[test]
    fn inspection_is_distinct_from_purchase_release() {
        // BR-AN05: inspection cannot also buy on release.
        let mut p = Pointer::default();
        p.event(1, TouchPhase::Started, Vec2::ZERO, 0.);
        p.event(1, TouchPhase::Ended, Vec2::ZERO, 0.7);
        assert!(p.taps.last().is_some_and(|tap| tap.2));
    }
    #[test]
    fn scrolling_and_cancel_do_not_activate() {
        // BR-AN05: scrolling, cancellation and wrong pointer releases buy nothing.
        for phase in [TouchPhase::Moved, TouchPhase::Canceled] {
            let mut p = Pointer::default();
            p.event(1, TouchPhase::Started, Vec2::ZERO, 0.);
            p.event(1, phase, Vec2::new(0., 32.), 0.1);
            p.event(1, TouchPhase::Ended, Vec2::ZERO, 0.2);
            assert!(p.taps.is_empty());
        }
    }
    #[test]
    fn small_tap_motion_never_scrolls_and_a_second_pointer_cancels_drag() {
        // BR-AN05: pointer jitter must not move a control before its tap is released.
        let mut p = Pointer::default();
        p.event(1, TouchPhase::Started, Vec2::ZERO, 0.);
        p.event(1, TouchPhase::Moved, Vec2::new(2., 2.), 0.1);
        assert!(p.drag.is_none());
        p.event(1, TouchPhase::Moved, Vec2::new(0., 30.), 0.2);
        assert!(p.drag.is_some());
        p.drag = None;
        p.event(2, TouchPhase::Started, Vec2::ZERO, 0.3);
        p.event(2, TouchPhase::Ended, Vec2::ZERO, 0.4);
        p.event(1, TouchPhase::Moved, Vec2::new(0., 60.), 0.5);
        p.event(1, TouchPhase::Ended, Vec2::ZERO, 0.6);
        assert!(p.drag.is_none());
        assert!(p.taps.is_empty());
    }

    #[test]
    fn another_finger_cannot_release_the_owned_action() {
        let mut p = Pointer::default();
        p.event(1, TouchPhase::Started, Vec2::ZERO, 0.);
        p.event(2, TouchPhase::Started, Vec2::ZERO, 0.);
        p.event(2, TouchPhase::Ended, Vec2::ZERO, 0.1);
        p.event(1, TouchPhase::Ended, Vec2::ZERO, 0.2);
        assert!(p.taps.is_empty());
    }

    #[test]
    fn coalesced_moves_preserve_the_scroll_distance_and_cancel_together() {
        // BR-AN05: scrolling must not depend on the renderer's frame rate.
        let mut p = Pointer::default();
        p.event(1, TouchPhase::Started, Vec2::new(100., 300.), 0.);
        for y in [295., 280., 250., 200.] {
            p.event(1, TouchPhase::Moved, Vec2::new(100., y), 0.1);
        }
        assert_eq!(p.drag, Some((Vec2::new(100., 200.), Vec2::new(0., -100.))));
        p.event(1, TouchPhase::Ended, Vec2::new(100., 200.), 0.2);
        assert!(p.taps.is_empty());
        p.event(2, TouchPhase::Started, Vec2::ZERO, 0.3);
        p.event(3, TouchPhase::Started, Vec2::ZERO, 0.3);
        assert!(p.drag.is_none());
    }
}

/// Enlarge the effective control, preserving the node's transform and draw-order hit testing.
pub fn hit(node: &ComputedNode, transform: UiGlobalTransform, point: Vec2, density: f32) -> bool {
    if node.size.min_element() <= 0. {
        return false;
    }
    let axes = Vec2::new(
        transform.transform_vector2(Vec2::X).length(),
        transform.transform_vector2(Vec2::Y).length(),
    );
    let mut target = *node;
    target.size = target
        .size
        .max(Vec2::splat(48. * density) / axes.max(Vec2::splat(0.001)));
    target.contains_point(transform, point)
}
