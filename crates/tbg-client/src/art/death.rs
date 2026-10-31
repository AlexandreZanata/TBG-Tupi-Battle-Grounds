//! Faction death styles (Humans fall, Undead collapse into the ground,
//! Machines spark and power down, Plants wilt and sink).

use bevy::prelude::*;
use tbg_core::Faction;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DeathProfile {
    pub faction: Faction,
    pub duration: f32,
    pub sink_depth: f32,
    pub wilt_scale_y: f32,
    pub pitch_deg: f32,
    pub jitter_amplitude: f32,
}

pub fn profile_for_faction(faction: Faction) -> DeathProfile {
    match faction {
        Faction::Humans => DeathProfile {
            faction,
            duration: 0.60,
            sink_depth: -0.15,
            wilt_scale_y: 1.0,
            pitch_deg: -80.0,
            jitter_amplitude: 0.0,
        },
        Faction::Undead => DeathProfile {
            faction,
            duration: 0.80,
            sink_depth: -0.65,
            wilt_scale_y: 0.80,
            pitch_deg: -20.0,
            jitter_amplitude: 0.0,
        },
        Faction::Machines => DeathProfile {
            faction,
            duration: 0.60,
            sink_depth: -0.10,
            wilt_scale_y: 0.95,
            pitch_deg: -35.0,
            jitter_amplitude: 0.04,
        },
        Faction::Plants => DeathProfile {
            faction,
            duration: 0.75,
            sink_depth: -0.40,
            wilt_scale_y: 0.30,
            pitch_deg: -10.0,
            jitter_amplitude: 0.0,
        },
    }
}

pub fn evaluate(faction: Faction, dead_for: f32, base_scale: f32, base_yaw: f32) -> Transform {
    let p = profile_for_faction(faction);
    let s = (dead_for / p.duration).clamp(0.0, 1.0);
    let e = s * (2.0 - s);

    let (translation, rotation, scale_mult) = match faction {
        Faction::Humans => {
            let rot = Quat::from_rotation_y(base_yaw)
                * Quat::from_rotation_x(p.pitch_deg.to_radians() * e);
            let pos = Vec3::new(0.0, p.sink_depth * e, -0.30 * e);
            (pos, rot, Vec3::ONE)
        }
        Faction::Undead => {
            let rot = Quat::from_rotation_y(base_yaw)
                * Quat::from_rotation_x(p.pitch_deg.to_radians() * e);
            let pos = Vec3::new(0.0, p.sink_depth * e, 0.0);
            let sc = Vec3::new(
                1.0 + 0.10 * e,
                1.0 - (1.0 - p.wilt_scale_y) * e,
                1.0 + 0.10 * e,
            );
            (pos, rot, sc)
        }
        Faction::Machines => {
            let jitter = if dead_for < 0.25 {
                (dead_for * 60.0).sin() * p.jitter_amplitude * (1.0 - dead_for / 0.25)
            } else {
                0.0
            };
            let rot = Quat::from_rotation_y(base_yaw)
                * Quat::from_rotation_x(p.pitch_deg.to_radians() * e);
            let pos = Vec3::new(jitter, p.sink_depth * e, 0.0);
            (pos, rot, Vec3::ONE)
        }
        Faction::Plants => {
            let rot = Quat::from_rotation_y(base_yaw)
                * Quat::from_rotation_x(p.pitch_deg.to_radians() * e);
            let pos = Vec3::new(0.0, p.sink_depth * e, 0.0);
            let sc = Vec3::new(
                1.0 + 0.15 * e,
                1.0 - (1.0 - p.wilt_scale_y) * e,
                1.0 + 0.15 * e,
            );
            (pos, rot, sc)
        }
    };

    Transform {
        translation,
        rotation,
        scale: Vec3::splat(base_scale) * scale_mult,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_death_profiles_exist_for_all_factions() {
        for f in [
            Faction::Humans,
            Faction::Undead,
            Faction::Machines,
            Faction::Plants,
        ] {
            let p = profile_for_faction(f);
            assert_eq!(p.faction, f);
            assert!(p.duration >= 0.50);
            let initial = evaluate(f, 0.0, 1.0, 0.0);
            assert_eq!(initial.translation, Vec3::ZERO);
            assert_eq!(initial.scale, Vec3::ONE);

            let fallen = evaluate(f, p.duration, 1.0, 0.0);
            match f {
                Faction::Humans => assert!(fallen.translation.y < 0.0),
                Faction::Undead => assert!(fallen.translation.y <= p.sink_depth),
                Faction::Machines => assert!(fallen.rotation != Quat::IDENTITY),
                Faction::Plants => assert!(fallen.scale.y < 0.5),
            }
        }
    }
}
