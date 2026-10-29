//! System driving unit figure yaw toward active aim targets (D1, D3).

use bevy::prelude::*;

use super::UnitVisual;
use crate::art::aim::{Aim, AimTarget};
use crate::art::melee::{lunge_distance, strike_profile_for_unit, HitRecoil};
use crate::sync::{world_x, MatchInfo, UnitEntities};

/// Updates directional aiming and cosmetic offsets for live match units:
/// - Units with `Aim` compute yaw toward target, transitioning Active -> Hold -> Release.
/// - Figure child entity is rotated to `aim.current_yaw`.
/// - Melee attackers apply forward cosmetic lunge along the aim line.
/// - Units with `HitRecoil` apply directional recoil away from the attacker.
/// - Units without `Aim` maintain base yaw (`v.yaw`) and neutral translation.
pub fn update_aim(
    mut commands: Commands,
    time: Res<Time>,
    map: Option<Res<UnitEntities>>,
    match_info: Option<Res<MatchInfo>>,
    mut units: Query<(
        Entity,
        &UnitVisual,
        Option<&mut Aim>,
        Option<&mut HitRecoil>,
    )>,
    mut transforms: Query<&mut Transform>,
) {
    let now = time.elapsed_secs_f64();
    let dt = time.delta_secs();
    let you = match_info.and_then(|m| m.you).unwrap_or(tbg_core::Seat::A);

    for (e, v, mut aim_opt, mut recoil_opt) in &mut units {
        let attacker_pos = transforms
            .get(e)
            .map(|t| t.translation)
            .unwrap_or(Vec3::ZERO);

        let mut recoil_offset = Vec3::ZERO;
        if let Some(ref mut recoil) = recoil_opt {
            recoil.update(dt);
            recoil_offset = recoil.offset_vec3();
            if recoil.elapsed >= recoil.duration {
                commands.entity(e).remove::<HitRecoil>();
            }
        }

        if let Some(ref mut aim) = aim_opt {
            let target_pos = match aim.target {
                AimTarget::Unit(target_id) => map
                    .as_ref()
                    .and_then(|m| m.0.get(&target_id))
                    .and_then(|&target_e| transforms.get(target_e).ok())
                    .map(|t| t.translation),
                AimTarget::Stronghold(seat) => {
                    let seat_x = tbg_core::state::stronghold_x(seat);
                    let wx = world_x(seat_x, you);
                    // Hit point on Stronghold face directed toward attacker
                    let hit_x = if seat == you { wx + 0.8 } else { wx - 0.8 };
                    Some(Vec3::new(hit_x, 0.6, attacker_pos.z))
                }
            };

            aim.update(now, dt, target_pos, attacker_pos);

            let profile = strike_profile_for_unit(v.form.0, v.form.1, v.form.2);
            let elapsed = (now - aim.started) as f32;
            let lunge_dist = lunge_distance(elapsed, 0.30, 0.50, profile.lunge);
            let lunge_offset = Vec3::new(
                aim.current_yaw.sin() * lunge_dist,
                0.0,
                aim.current_yaw.cos() * lunge_dist,
            );

            if let Ok(mut fig_t) = transforms.get_mut(v.figure) {
                fig_t.rotation = Quat::from_rotation_y(aim.current_yaw);
                fig_t.translation = lunge_offset + recoil_offset;
            }

            if aim.expired {
                commands.entity(e).remove::<Aim>();
            }
        } else if let Ok(mut fig_t) = transforms.get_mut(v.figure) {
            fig_t.rotation = Quat::from_rotation_y(v.yaw);
            fig_t.translation = recoil_offset;
        }
    }
}
