//! Strike profile table, melee data, cosmetic lunge, and directional recoil math (D3).

use bevy::prelude::*;
use tbg_core::UnitKind;

/// Weapon classes defined in D3.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WeaponClass {
    Slash,
    Smash,
    Chop,
    Bash,
    Mech,
}

/// Visual impact category for sparks and screen punch.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImpactVfx {
    Spark,
    HeavySpark,
    Grit,
}

/// Strike profile parameters defining authored melee feel (D3).
#[allow(dead_code)]
#[derive(Clone, Copy, Debug)]
pub struct StrikeProfile {
    pub class: WeaponClass,
    /// Fraction of attack duration spent in windup before Strike (marker at 0.30s).
    pub windup_frac: f32,
    /// Weapon swing arc in degrees.
    pub arc_deg: f32,
    /// Torso rotation twist in degrees during windup and strike.
    pub torso_twist_deg: f32,
    /// Maximum cosmetic forward lunge distance in tiles (must be <= 0.25).
    pub lunge: f32,
    /// Hit-stop duration in milliseconds applied to the target animator (contract: 50 ms).
    pub hit_stop_ms: f32,
    /// Visual impact VFX category.
    pub impact: ImpactVfx,
}

/// Authoritative strike profile table assigned per weapon class.
pub const fn profile_for_class(class: WeaponClass) -> StrikeProfile {
    match class {
        WeaponClass::Slash => StrikeProfile {
            class,
            windup_frac: 0.60,
            arc_deg: 85.0,
            torso_twist_deg: 20.0,
            lunge: 0.20,
            hit_stop_ms: 50.0,
            impact: ImpactVfx::Spark,
        },
        WeaponClass::Smash => StrikeProfile {
            class,
            windup_frac: 0.60,
            arc_deg: 65.0,
            torso_twist_deg: 26.0,
            lunge: 0.22,
            hit_stop_ms: 50.0,
            impact: ImpactVfx::HeavySpark,
        },
        WeaponClass::Chop => StrikeProfile {
            class,
            windup_frac: 0.60,
            arc_deg: 90.0,
            torso_twist_deg: 28.0,
            lunge: 0.24,
            hit_stop_ms: 50.0,
            impact: ImpactVfx::HeavySpark,
        },
        WeaponClass::Bash => StrikeProfile {
            class,
            windup_frac: 0.50,
            arc_deg: 30.0,
            torso_twist_deg: 12.0,
            lunge: 0.25,
            hit_stop_ms: 50.0,
            impact: ImpactVfx::Grit,
        },
        WeaponClass::Mech => StrikeProfile {
            class,
            windup_frac: 0.55,
            arc_deg: 40.0,
            torso_twist_deg: 0.0,
            lunge: 0.20,
            hit_stop_ms: 50.0,
            impact: ImpactVfx::Spark,
        },
    }
}

/// Resolves the weapon class and profile for a specific unit form.
pub fn strike_profile_for_unit(kind: UnitKind, tier: u8, _branch: u8) -> StrikeProfile {
    let mut profile = match kind {
        UnitKind::Shieldbearer => profile_for_class(WeaponClass::Smash),
        UnitKind::BoneBrute => profile_for_class(WeaponClass::Chop),
        UnitKind::Sawbot => profile_for_class(WeaponClass::Mech),
        UnitKind::Rampart => profile_for_class(WeaponClass::Bash),
        _ => profile_for_class(WeaponClass::Slash),
    };

    // Scale profile by tier evolution (tier 1: x1.08, tier 2: x1.16)
    let scale = match tier {
        1 => 1.08,
        2 => 1.16,
        _ => 1.00,
    };
    profile.arc_deg *= scale;
    profile.lunge = (profile.lunge * scale).min(0.25);
    profile
}

/// Computes the forward cosmetic lunge distance along the aim line.
///
/// Moves forward from 0.0 to `profile.lunge` at `strike_time` (0.30s),
/// then smoothly springs back to 0.0 at `duration` (0.50s).
pub fn lunge_distance(elapsed: f32, strike_time: f32, duration: f32, max_lunge: f32) -> f32 {
    if elapsed <= 0.0 || elapsed >= duration {
        0.0
    } else if elapsed < strike_time {
        let p = (elapsed / strike_time).clamp(0.0, 1.0);
        let ease = p * p; // quadratic ease-in
        max_lunge * ease
    } else {
        let p = ((elapsed - strike_time) / (duration - strike_time)).clamp(0.0, 1.0);
        let ease = (1.0 - p) * (1.0 - p); // quadratic decay spring-back
        max_lunge * ease
    }
}

/// State of an active hit reaction recoil on a target unit.
#[derive(Component, Clone, Copy, Debug)]
pub struct HitRecoil {
    pub dir_xz: Vec2,
    pub elapsed: f32,
    pub duration: f32,
    pub max_dist: f32,
}

impl Default for HitRecoil {
    fn default() -> Self {
        Self {
            dir_xz: Vec2::ZERO,
            elapsed: 0.0,
            duration: 0.120, // 120 ms
            max_dist: 0.08,  // <= 0.08 tile
        }
    }
}

impl HitRecoil {
    /// Triggers or adds a directional recoil away from the attacker.
    pub fn trigger(&mut self, attacker_pos: Vec3, target_pos: Vec3) {
        let dx = target_pos.x - attacker_pos.x;
        let dz = target_pos.z - attacker_pos.z;
        let diff = Vec2::new(dx, dz);
        let dir = if diff.length_squared() > 1e-4 {
            diff.normalize()
        } else {
            Vec2::X // fallback along lane
        };

        if self.elapsed >= self.duration {
            self.dir_xz = dir;
            self.elapsed = 0.0;
        } else {
            // Additive recoil, clamped to 0.12 max
            let current_vec = self.offset_vec2();
            let new_vec = current_vec + dir * self.max_dist;
            self.dir_xz = new_vec.normalize_or_zero();
            self.max_dist = new_vec.length().min(0.12);
            self.elapsed = 0.0;
        }
    }

    /// Current recoil offset in 2D (X, Z).
    pub fn offset_vec2(&self) -> Vec2 {
        if self.elapsed >= self.duration || self.duration <= 0.0 {
            Vec2::ZERO
        } else {
            let p = (self.elapsed / self.duration).clamp(0.0, 1.0);
            let ease = (1.0 - p) * (1.0 - p);
            self.dir_xz * (self.max_dist * ease)
        }
    }

    /// Current recoil offset in 3D (X, 0, Z).
    pub fn offset_vec3(&self) -> Vec3 {
        let off = self.offset_vec2();
        Vec3::new(off.x, 0.0, off.y)
    }

    /// Advances recoil time.
    pub fn update(&mut self, dt: f32) {
        self.elapsed += dt;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strike_profile_table_values() {
        // Assert each weapon class has authored parameters and lunge <= 0.25
        for class in [
            WeaponClass::Slash,
            WeaponClass::Smash,
            WeaponClass::Chop,
            WeaponClass::Bash,
            WeaponClass::Mech,
        ] {
            let p = profile_for_class(class);
            assert!(p.lunge <= 0.25, "Lunge {0} exceeded 0.25 tile", p.lunge);
            assert_eq!(p.hit_stop_ms, 50.0, "Hit stop must be 50 ms");
            assert!(p.windup_frac > 0.0 && p.windup_frac < 1.0);
            assert!(p.arc_deg > 0.0);
            assert!(p.torso_twist_deg >= 0.0);
        }
    }

    #[test]
    fn test_unit_weapon_mapping() {
        let shieldbearer = strike_profile_for_unit(UnitKind::Shieldbearer, 0, 0);
        assert_eq!(shieldbearer.class, WeaponClass::Smash);
        assert_eq!(shieldbearer.impact, ImpactVfx::HeavySpark);

        let bone_brute = strike_profile_for_unit(UnitKind::BoneBrute, 0, 0);
        assert_eq!(bone_brute.class, WeaponClass::Chop);
        assert_eq!(bone_brute.impact, ImpactVfx::HeavySpark);

        let sawbot = strike_profile_for_unit(UnitKind::Sawbot, 0, 0);
        assert_eq!(sawbot.class, WeaponClass::Mech);

        let rampart = strike_profile_for_unit(UnitKind::Rampart, 0, 0);
        assert_eq!(rampart.class, WeaponClass::Bash);
    }

    #[test]
    fn test_tier_evolution_scales_within_budget() {
        let t0 = strike_profile_for_unit(UnitKind::Shieldbearer, 0, 0);
        let t2 = strike_profile_for_unit(UnitKind::Shieldbearer, 2, 0);
        assert!(t2.arc_deg > t0.arc_deg, "Tier 2 should have wider arc");
        assert!(t2.lunge <= 0.25, "Tier 2 lunge must not exceed 0.25 tile");
    }

    #[test]
    fn test_lunge_timing_and_spring_back() {
        let strike_time = 0.30;
        let duration = 0.50;
        let max_lunge = 0.22;

        // At t=0, lunge is 0
        assert_eq!(lunge_distance(0.0, strike_time, duration, max_lunge), 0.0);

        // At t=0.15 (half windup), lunge is partial
        let mid = lunge_distance(0.15, strike_time, duration, max_lunge);
        assert!(mid > 0.0 && mid < max_lunge);

        // At t=0.30 (strike contact), lunge reaches peak
        let peak = lunge_distance(strike_time, strike_time, duration, max_lunge);
        assert!((peak - max_lunge).abs() < 1e-4);

        // At t=0.40 (recovery), lunge springs back
        let rec = lunge_distance(0.40, strike_time, duration, max_lunge);
        assert!(rec < peak && rec > 0.0);

        // At t=0.50 (end of clip), lunge is back to 0
        assert_eq!(
            lunge_distance(duration, strike_time, duration, max_lunge),
            0.0
        );
    }

    #[test]
    fn test_hit_recoil_direction_and_duration() {
        let mut recoil = HitRecoil::default();
        let attacker = Vec3::new(4.0, 0.0, 1.0);
        let target = Vec3::new(5.0, 0.0, 2.0);

        recoil.trigger(attacker, target);
        let offset = recoil.offset_vec3();

        // Direction must be away from attacker (dx > 0, dz > 0)
        assert!(offset.x > 0.0);
        assert!(offset.z > 0.0);
        assert!(offset.length() <= 0.081);

        // After 120 ms, recoil returns to 0
        recoil.update(0.125);
        let final_offset = recoil.offset_vec3();
        assert_eq!(final_offset, Vec3::ZERO);
    }
}
