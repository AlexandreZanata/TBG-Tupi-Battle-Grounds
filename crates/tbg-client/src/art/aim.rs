//! Pure math and state machine for directional aiming and impact pairing (D1, D3).
//!
//! Visual presentation only; invariant I4: targeting never alters simulation outcomes,
//! never queries nearest-unit proximity, and reads targets strictly from authoritative
//! events (Event::AttackStart { from, to }).

#![allow(dead_code)]

use bevy::prelude::*;
pub use tbg_core::state::Target as AimTarget;

/// Dead-zone to eliminate angular jitter on small oscillations (2.0° in radians).
pub const DEAD_ZONE: f32 = 0.034_906_585; // 2.0 * PI / 180.0

/// Maximum angular slew rate toward target (~720°/s in radians/sec).
pub const MAX_SLEW_RATE: f32 = 12.566_371; // 4.0 * PI

/// Duration in seconds to return from aim yaw to base idle yaw.
pub const RELEASE_DURATION: f32 = 0.150;

/// Expiry lifetime in seconds for unmatched impact pairing entries in the FIFO ring.
pub const IMPACT_EXPIRY_SECS: f64 = 1.0;

/// Base visual yaw for Seat A (own units, facing +X down lane, turned 25° toward camera).
pub const BASE_YAW_OWN: f32 = 65.0 * std::f32::consts::PI / 180.0;

/// Base visual yaw for Seat B (enemy units, facing -X down lane, turned 25° toward camera).
pub const BASE_YAW_ENEMY: f32 = -65.0 * std::f32::consts::PI / 180.0;

/// Computes the desired world yaw for an attacker to face a target point in the XZ plane.
///
/// Coordinate system in TBG:
/// - +X is right (down the lane from Seat A to Seat B)
/// - +Z is toward the match camera (forward from lane)
///
/// A figure at local yaw 0 faces +Z. Rotation by yaw θ yields forward vector (sin θ, 0, cos θ).
/// Therefore, the world yaw to point from `attacker` to `target` is:
/// θ = atan2(dx, dz) where dx = target.x - attacker.x, dz = target.z - attacker.z.
pub fn facing_yaw(attacker: Vec3, target: Vec3) -> f32 {
    let dx = target.x - attacker.x;
    let dz = target.z - attacker.z;
    if dx.abs() < 1e-6 && dz.abs() < 1e-6 {
        return 0.0;
    }
    dx.atan2(dz)
}

/// Normalizes an angle in radians to the range (-π, π].
pub fn normalize_angle(mut angle: f32) -> f32 {
    while angle > std::f32::consts::PI {
        angle -= std::f32::consts::TAU;
    }
    while angle <= -std::f32::consts::PI {
        angle += std::f32::consts::TAU;
    }
    angle
}

/// Computes the shortest angular difference (to - from) in (-π, π].
pub fn shortest_angle_diff(from: f32, to: f32) -> f32 {
    normalize_angle(to - from)
}

/// Advances `current` yaw toward `desired` yaw by at most `max_step` radians along the shortest arc.
///
/// If |diff| <= `dead_zone`, `current` is kept unchanged to eliminate micro-jitter.
pub fn turn_toward(current: f32, desired: f32, max_step: f32, dead_zone: f32) -> f32 {
    let diff = shortest_angle_diff(current, desired);
    if diff.abs() <= dead_zone {
        return current;
    }
    let step = diff.clamp(-max_step, max_step);
    normalize_angle(current + step)
}

/// Lifecycle state for an active directional aim.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AimPhase {
    /// Slewing toward the target at `MAX_SLEW_RATE`.
    Active,
    /// Holding aim facing the target until strike/release time.
    Hold,
    /// Returning smoothly to base yaw (+65° / -65°) over `RELEASE_DURATION`.
    Release,
}

/// Component attached to an attacking unit's root entity to drive directional aiming.
#[derive(Component, Clone, Debug)]
pub struct Aim {
    /// Authoritative target copied from `Event::AttackStart`.
    pub target: AimTarget,
    /// Match time when the attack started (seconds).
    pub started: f64,
    /// Windup hold duration before release/strike (e.g. 0.18s ranged, 0.30s melee).
    pub hold: f32,
    /// Idle base yaw to return to (+65° own, -65° enemy).
    pub base_yaw: f32,
    /// Current rendered yaw angle.
    pub current_yaw: f32,
    /// Current phase in the state machine.
    pub phase: AimPhase,
    /// Last known world position of target, preserved if target despawns mid-attack.
    pub last_target_point: Option<Vec3>,
    /// Whether the aim sequence has completed and can be removed.
    pub expired: bool,
}

impl Aim {
    /// Creates a new Aim state machine for an attacker.
    pub fn new(
        target: AimTarget,
        started: f64,
        hold: f32,
        base_yaw: f32,
        initial_yaw: f32,
    ) -> Self {
        Self {
            target,
            started,
            hold,
            base_yaw,
            current_yaw: initial_yaw,
            phase: AimPhase::Active,
            last_target_point: None,
            expired: false,
        }
    }

    /// Retargets mid-clip when a new AttackStart arrives before the previous clip finished.
    /// Preserves `current_yaw` to prevent snapping back to base yaw.
    pub fn retarget(&mut self, new_target: AimTarget, now: f64, new_hold: f32) {
        self.target = new_target;
        self.started = now;
        self.hold = new_hold;
        self.phase = AimPhase::Active;
        self.expired = false;
    }

    /// Updates the aim state machine for the elapsed time step `dt`.
    pub fn update(&mut self, now: f64, dt: f32, target_pos: Option<Vec3>, attacker_pos: Vec3) {
        if let Some(pos) = target_pos {
            self.last_target_point = Some(pos);
        }

        let pt = match target_pos.or(self.last_target_point) {
            Some(p) => p,
            None => {
                // No target point ever known; expire after hold + release
                if (now - self.started) as f32 >= self.hold + RELEASE_DURATION {
                    self.expired = true;
                }
                return;
            }
        };

        let elapsed = (now - self.started) as f32;

        if elapsed < self.hold {
            // Slew or hold toward target
            let desired = facing_yaw(attacker_pos, pt);
            let max_step = MAX_SLEW_RATE * dt;
            self.current_yaw = turn_toward(self.current_yaw, desired, max_step, DEAD_ZONE);

            let diff = shortest_angle_diff(self.current_yaw, desired);
            if diff.abs() <= DEAD_ZONE {
                self.phase = AimPhase::Hold;
            } else {
                self.phase = AimPhase::Active;
            }
        } else if elapsed < self.hold + RELEASE_DURATION {
            // Smoothly release back to base yaw over RELEASE_DURATION
            self.phase = AimPhase::Release;
            let remaining = (self.hold + RELEASE_DURATION - elapsed).max(1e-4);
            let diff = shortest_angle_diff(self.current_yaw, self.base_yaw);
            let step = diff * (dt / remaining).min(1.0);
            self.current_yaw = normalize_angle(self.current_yaw + step);
        } else {
            self.current_yaw = self.base_yaw;
            self.expired = true;
        }
    }
}

/// An unresolved impact event in flight between `AttackStart` and `Hit`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ImpactEntry {
    pub from: AimTarget,
    pub to: AimTarget,
    pub due: f64,
}

/// Small FIFO ring buffer per match tracking pending impacts for directional hit reactions.
///
/// In TBG, `Event::Hit { to, amount }` carries no attacker. The FIFO pairs each `Hit`
/// with the oldest pending `AttackStart { from, to }` for that target.
#[derive(Default, Clone, Debug)]
pub struct ImpactRing {
    entries: Vec<ImpactEntry>,
}

impl ImpactRing {
    pub fn new() -> Self {
        Self {
            entries: Vec::with_capacity(32),
        }
    }

    /// Records an attack initiation.
    pub fn push(&mut self, from: AimTarget, to: AimTarget, due: f64) {
        self.entries.push(ImpactEntry { from, to, due });
    }

    /// Prunes stale entries older than `IMPACT_EXPIRY_SECS` (1.0 s).
    pub fn prune_stale(&mut self, now: f64) {
        self.entries.retain(|e| (now - e.due) <= IMPACT_EXPIRY_SECS);
    }

    /// Pops the oldest pending attacker directed at `to`.
    /// Stale entries are pruned before matching.
    pub fn pop_matching(&mut self, to: AimTarget, now: f64) -> Option<AimTarget> {
        self.prune_stale(now);
        if let Some(pos) = self.entries.iter().position(|e| e.to == to) {
            Some(self.entries.remove(pos).from)
        } else {
            None
        }
    }

    /// Returns the number of currently tracked pending impacts.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns true if there are no pending impacts.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;
    use tbg_core::Seat;

    const EPSILON: f32 = 1e-4;

    #[test]
    fn test_aim_facing_front_up_down() {
        // Contract check (I4): Attacker at origin (5.0, 0.0, 0.0).
        let attacker = Vec3::new(5.0, 0.0, 0.0);

        // Target front (+X down lane): dx = 3.0, dz = 0.0 => +90° (+π/2)
        let target_front = Vec3::new(8.0, 0.0, 0.0);
        let yaw_front = facing_yaw(attacker, target_front);
        assert!(
            (yaw_front - PI / 2.0).abs() < EPSILON,
            "front yaw was {yaw_front}"
        );

        // Target down (+Z towards camera): dx = 0.0, dz = 2.0 => 0.0°
        let target_down = Vec3::new(5.0, 0.0, 2.0);
        let yaw_down = facing_yaw(attacker, target_down);
        assert!(yaw_down.abs() < EPSILON, "down yaw was {yaw_down}");

        // Target up (-Z away from camera): dx = 0.0, dz = -2.0 => 180° (π)
        let target_up = Vec3::new(5.0, 0.0, -2.0);
        let yaw_up = facing_yaw(attacker, target_up);
        assert!((yaw_up.abs() - PI).abs() < EPSILON, "up yaw was {yaw_up}");
    }

    #[test]
    fn test_aim_facing_diagonal() {
        // Contract check (I4): 45° diagonals.
        let attacker = Vec3::new(4.0, 0.0, 0.0);

        // Forward-camera diagonal (+X, +Z): dx = 2.0, dz = 2.0 => +45° (+π/4)
        let target_diag_pos = Vec3::new(6.0, 0.0, 2.0);
        let yaw_pos = facing_yaw(attacker, target_diag_pos);
        assert!(
            (yaw_pos - PI / 4.0).abs() < EPSILON,
            "diag pos yaw was {yaw_pos}"
        );

        // Forward-away diagonal (+X, -Z): dx = 2.0, dz = -2.0 => +135° (+3π/4)
        let target_diag_neg = Vec3::new(6.0, 0.0, -2.0);
        let yaw_neg = facing_yaw(attacker, target_diag_neg);
        assert!(
            (yaw_neg - 3.0 * PI / 4.0).abs() < EPSILON,
            "diag neg yaw was {yaw_neg}"
        );
    }

    #[test]
    fn test_aim_facing_near_far_and_behind() {
        // Contract check (I4): Distance scaling invariance.
        let attacker = Vec3::new(2.0, 0.0, 0.0);

        // Near target (0.4 tile)
        let target_near = Vec3::new(2.4, 0.0, 0.0);
        assert!((facing_yaw(attacker, target_near) - PI / 2.0).abs() < EPSILON);

        // Far target (6.0 tiles)
        let target_far = Vec3::new(8.0, 0.0, 0.0);
        assert!((facing_yaw(attacker, target_far) - PI / 2.0).abs() < EPSILON);

        // Target behind (-X): dx = -3.0, dz = 0.0 => -90° (-π/2)
        let target_behind = Vec3::new(-1.0, 0.0, 0.0);
        assert!((facing_yaw(attacker, target_behind) - (-PI / 2.0)).abs() < EPSILON);
    }

    #[test]
    fn test_aim_mirrored_seat_b() {
        // Contract check (I4): Seat A attacks from 4.0 toward 7.0 (dx = +3.0, dz = 1.6).
        // Seat B attacks from 12.0 toward 9.0 (dx = -3.0, dz = 1.6).
        let attacker_a = Vec3::new(4.0, 0.0, 0.0);
        let target_a = Vec3::new(7.0, 0.0, 1.6);
        let yaw_a = facing_yaw(attacker_a, target_a);

        let attacker_b = Vec3::new(12.0, 0.0, 0.0);
        let target_b = Vec3::new(9.0, 0.0, 1.6);
        let yaw_b = facing_yaw(attacker_b, target_b);

        // Mirrored seat gives mirrored sign across the X=0 axis:
        assert!(
            (yaw_a + yaw_b).abs() < EPSILON,
            "Seat B yaw {yaw_b} should be negative of Seat A {yaw_a}"
        );
        assert!((BASE_YAW_ENEMY + BASE_YAW_OWN).abs() < EPSILON);
    }

    #[test]
    fn test_shortest_arc_across_pi() {
        // From +170° (+2.967 rad) to -170° (-2.967 rad).
        // Shortest step is +20° (+0.349 rad) passing through ±π, NOT -340°.
        let from = 170.0 * PI / 180.0;
        let to = -170.0 * PI / 180.0;
        let diff = shortest_angle_diff(from, to);
        let expected_diff = 20.0 * PI / 180.0;
        assert!(
            (diff - expected_diff).abs() < EPSILON,
            "diff was {diff}, expected {expected_diff}"
        );

        // Slewing by small step advances across boundary without reverse spinning:
        let stepped = turn_toward(from, to, 0.1, 0.0);
        let expected_stepped = normalize_angle(from + 0.1);
        assert!((stepped - expected_stepped).abs() < EPSILON);
    }

    #[test]
    fn test_deadzone_prevents_jitter() {
        // Target jitters by 1.5° (0.026 rad < DEAD_ZONE 0.035 rad):
        let current = 1.0;
        let desired = current + 0.025; // < DEAD_ZONE
        let result = turn_toward(current, desired, 0.5, DEAD_ZONE);
        assert_eq!(result, current, "Dead-zone must eliminate jitter when < 2°");

        // Target moves by 3.0° (0.052 rad > DEAD_ZONE):
        let desired_move = current + 0.052;
        let result_move = turn_toward(current, desired_move, 0.5, DEAD_ZONE);
        assert!(
            result_move > current,
            "Angles exceeding dead-zone must turn toward target"
        );
    }

    #[test]
    fn test_aim_state_machine_hold_and_release() {
        let target = AimTarget::Unit(10);
        let base_yaw = BASE_YAW_OWN;
        let attacker_pos = Vec3::new(2.0, 0.0, 0.0);
        let target_pos = Vec3::new(6.0, 0.0, 0.0); // directly down lane (+π/2)

        let mut aim = Aim::new(target, 0.0, 0.30, base_yaw, base_yaw);
        assert_eq!(aim.phase, AimPhase::Active);

        // Step 1: dt = 0.10s (elapsed 0.10 < 0.30) -> slew toward target
        aim.update(0.10, 0.10, Some(target_pos), attacker_pos);
        assert!(aim.current_yaw > base_yaw);

        // Step 2: dt = 0.10s (elapsed 0.20 < 0.30) -> reaches desired yaw (+π/2), enters Hold
        aim.update(0.20, 0.10, Some(target_pos), attacker_pos);
        assert!((aim.current_yaw - PI / 2.0).abs() < DEAD_ZONE);
        assert_eq!(aim.phase, AimPhase::Hold);

        // Step 3: dt = 0.15s (elapsed 0.35 >= hold 0.30, < hold + 0.150) -> Release phase
        aim.update(0.35, 0.15, Some(target_pos), attacker_pos);
        assert_eq!(aim.phase, AimPhase::Release);
        assert!(!aim.expired);

        // Step 4: dt = 0.15s (elapsed 0.50 >= 0.45) -> returns to base_yaw and expires
        aim.update(0.50, 0.15, Some(target_pos), attacker_pos);
        assert!((aim.current_yaw - base_yaw).abs() < EPSILON);
        assert!(aim.expired);
    }

    #[test]
    fn test_retarget_keeps_current_yaw() {
        let target_a = AimTarget::Unit(1);
        let target_b = AimTarget::Unit(2);
        let base_yaw = BASE_YAW_OWN;

        let mut aim = Aim::new(target_a, 0.0, 0.30, base_yaw, base_yaw);
        aim.current_yaw = 1.45; // in middle of swing

        // Retarget arrives at t = 0.15s:
        aim.retarget(target_b, 0.15, 0.30);
        assert_eq!(aim.target, target_b);
        assert_eq!(aim.started, 0.15);
        assert_eq!(
            aim.current_yaw, 1.45,
            "Retarget must never snap back to base yaw"
        );
        assert_eq!(aim.phase, AimPhase::Active);
    }

    #[test]
    fn test_target_removed_freezes_last_point() {
        let target = AimTarget::Unit(5);
        let attacker_pos = Vec3::new(1.0, 0.0, 0.0);
        let target_pos = Vec3::new(4.0, 0.0, 2.0);

        let mut aim = Aim::new(target, 0.0, 0.30, BASE_YAW_OWN, BASE_YAW_OWN);

        // First frame: target is alive at target_pos
        aim.update(0.05, 0.05, Some(target_pos), attacker_pos);
        assert_eq!(aim.last_target_point, Some(target_pos));

        // Second frame: target despawned (target_pos is None)
        aim.update(0.10, 0.05, None, attacker_pos);
        // Desired yaw calculation still uses frozen last point:
        let desired = facing_yaw(attacker_pos, target_pos);
        assert!((aim.current_yaw - desired).abs() < 0.5);
        assert_eq!(aim.last_target_point, Some(target_pos));
    }

    #[test]
    fn test_impact_ring_fifo_pairing_and_expiry() {
        let mut ring = ImpactRing::new();

        let attacker_1 = AimTarget::Unit(101);
        let attacker_2 = AimTarget::Unit(102);
        let target_shared = AimTarget::Unit(200);
        let target_other = AimTarget::Unit(201);

        // Two attackers target the same unit at t = 1.00s and t = 1.10s
        ring.push(attacker_1, target_shared, 1.00);
        ring.push(attacker_2, target_shared, 1.10);
        assert_eq!(ring.len(), 2);

        // First Hit on target_shared pops the oldest (attacker_1)
        let popped_1 = ring.pop_matching(target_shared, 1.30);
        assert_eq!(popped_1, Some(attacker_1));

        // Second Hit on target_shared pops the second oldest (attacker_2)
        let popped_2 = ring.pop_matching(target_shared, 1.35);
        assert_eq!(popped_2, Some(attacker_2));

        // Third Hit on target_shared finds nothing (neutral reaction)
        let popped_3 = ring.pop_matching(target_shared, 1.40);
        assert_eq!(popped_3, None);

        // Push an entry at t = 2.00s for target_other
        ring.push(attacker_1, target_other, 2.00);

        // Query at t = 3.50s (1.50s later > 1.0s expiry)
        let expired_pop = ring.pop_matching(target_other, 3.50);
        assert_eq!(expired_pop, None, "Entries older than 1.0s must expire");
        assert!(ring.is_empty());
    }

    #[test]
    fn test_stronghold_target_compatibility() {
        let mut ring = ImpactRing::new();
        let attacker = AimTarget::Unit(50);
        let stronghold_target = AimTarget::Stronghold(Seat::B);

        ring.push(attacker, stronghold_target, 0.5);
        let match_result = ring.pop_matching(stronghold_target, 0.8);
        assert_eq!(match_result, Some(attacker));
    }
}
