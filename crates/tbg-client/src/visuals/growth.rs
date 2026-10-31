//! Seven shared sap sprouts show Essence; the nest glow follows its authoritative hatch timer.
use super::{UnitVisual, VisualAssets};
use crate::art::{anim::Animator, rig::Joint};
use bevy::prelude::*;
#[derive(Component)]
struct Sprouts([Entity; 7]);
pub(super) fn plugin(app: &mut App) {
    app.add_systems(Update, update);
}
fn update(
    mut commands: Commands,
    assets: Res<VisualAssets>,
    mut units: Query<(Entity, &UnitVisual, &Animator, Option<&Sprouts>)>,
) {
    for (e, v, a, sprouts) in &mut units {
        if v.kind() == tbg_core::UnitKind::Nester {
            for glow in &v.glow {
                commands
                    .entity(*glow)
                    .try_insert(Transform::from_scale(Vec3::splat(1.0 + 0.25 * a.growth)));
            }
        }
        if v.kind() != tbg_core::UnitKind::Leechroot {
            continue;
        }
        let n = usize::from((v.fx >> 11) & 7);
        if let Some(s) = sprouts {
            for (i, e) in s.0.iter().enumerate() {
                commands.entity(*e).try_insert(if i < n {
                    Visibility::Inherited
                } else {
                    Visibility::Hidden
                });
            }
            continue;
        }
        let ids = std::array::from_fn(|i| {
            let angle = i as f32 * std::f32::consts::TAU / 7.0;
            let id = commands
                .spawn((
                    Mesh3d(assets.tier_cube.clone()),
                    MeshMaterial3d(assets.heal.clone()),
                    Transform::from_xyz(angle.cos() * 0.18, 0.15 + angle.sin() * 0.12, 0.17)
                        .with_scale(Vec3::new(0.6, 1.7, 0.6)),
                    Visibility::Hidden,
                    bevy::light::NotShadowCaster,
                ))
                .id();
            commands
                .entity(a.joints[Joint::Chest.index()])
                .add_child(id);
            id
        });
        commands.entity(e).insert(Sprouts(ids));
    }
}
