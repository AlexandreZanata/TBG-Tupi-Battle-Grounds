//! System driving unit figure yaw toward active aim targets (D1, D3).

use bevy::prelude::*;

use super::UnitVisual;
use crate::art::aim::{Aim, AimTarget};
use crate::sync::{world_x, MatchInfo, UnitEntities};

/// Updates directional aiming for live match units:
/// - Units with `Aim` compute yaw toward target, transitioning Active -> Hold -> Release.
/// - Figure child entity is rotated to `aim.current_yaw`.
/// - Units without `Aim` maintain base yaw (`v.yaw`).
pub fn update_aim(
    mut commands: Commands,
    time: Res<Time>,
    map: Option<Res<UnitEntities>>,
    match_info: Option<Res<MatchInfo>>,
    mut units: Query<(Entity, &UnitVisual, Option<&mut Aim>)>,
    mut transforms: Query<&mut Transform>,
) {
    let now = time.elapsed_secs_f64();
    let dt = time.delta_secs();
    let you = match_info.and_then(|m| m.you).unwrap_or(tbg_core::Seat::A);

    for (e, v, mut aim_opt) in &mut units {
        let attacker_pos = transforms
            .get(e)
            .map(|t| t.translation)
            .unwrap_or(Vec3::ZERO);

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

            if let Ok(mut fig_t) = transforms.get_mut(v.figure) {
                fig_t.rotation = Quat::from_rotation_y(aim.current_yaw);
            }

            if aim.expired {
                commands.entity(e).remove::<Aim>();
            }
        } else if let Ok(mut fig_t) = transforms.get_mut(v.figure) {
            fig_t.rotation = Quat::from_rotation_y(v.yaw);
        }
    }
}
