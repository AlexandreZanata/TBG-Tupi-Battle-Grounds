//! Persistent champion effect indicators, replaced from the team snapshot on every reconnect.
use super::{Spawned, UnitVisual};
use crate::{app::Screen, vfx::VfxAssets};
use bevy::prelude::*;

#[derive(Component, Default)]
struct Indicators {
    glyphs: [Option<Entity>; 8],
    link: Option<Entity>,
}
pub(super) fn plugin(app: &mut App) {
    app.add_systems(PostUpdate, draw.run_if(in_state(Screen::Match)));
}
fn marker(
    c: &mut Commands,
    parent: Entity,
    slot: &mut Option<Entity>,
    mesh: &Handle<Mesh>,
    mat: &Handle<StandardMaterial>,
    pose: Option<Transform>,
) {
    let Some(pose) = pose else {
        if let Some(e) = *slot {
            c.entity(e).try_insert(Visibility::Hidden);
        }
        return;
    };
    let e = *slot.get_or_insert_with(|| {
        c.spawn((
            Mesh3d(mesh.clone()),
            MeshMaterial3d(mat.clone()),
            Visibility::Inherited,
            Transform::default(),
            ChildOf(parent),
            bevy::light::NotShadowCaster,
        ))
        .id()
    });
    c.entity(e).try_insert((pose, Visibility::Inherited));
}
fn draw(
    mut c: Commands,
    a: Res<VfxAssets>,
    map: Res<Spawned>,
    positions: Query<&Transform, With<UnitVisual>>,
    mut units: Query<(Entity, &Transform, &UnitVisual, Option<&mut Indicators>)>,
) {
    for (entity, root, u, marks) in &mut units {
        if super::champions::champion_of(u.kind).is_none() {
            continue;
        }
        let Some(mut marks) = marks else {
            c.entity(entity).insert(Indicators::default());
            continue;
        };
        let f = u.effect;
        let v = f.completion.unwrap_or_default();
        for (i, (visible, mesh, mat)) in [
            (v.echo_ticks > 0, &a.ring, &a.green),
            (v.regrowth_pulses > 0, &a.ball, &a.turquoise),
            (v.spark_ticks > 0, &a.chip, &a.gold),
            (v.interrupt_ticks > 0, &a.ring, &a.grey),
            (v.charged, &a.crescent, &a.turquoise),
            (f.poison > 0, &a.ball, &a.spore),
            (f.rooted || f.slowed, &a.ring, &a.violet),
            (f.shield > 0, &a.ring, &a.gold),
        ]
        .into_iter()
        .enumerate()
        {
            let t = Transform::from_xyz((i as f32 - 3.5) * 0.095, u.bar_height / root.scale.y, 0.0)
                .with_scale(Vec3::splat(if i == 1 { 0.04 } else { 0.07 }));
            marker(
                &mut c,
                entity,
                &mut marks.glyphs[i],
                mesh,
                mat,
                visible.then_some(t),
            );
        }
        let link = v
            .channel_target
            .or((f.bond != 0).then_some(f.bond))
            .and_then(|id| map.by_id.get(&id))
            .and_then(|e| positions.get(*e).ok())
            .map(|target| {
                let from = Vec3::Y * 0.67;
                let to = (target.translation - root.translation) / root.scale.x + Vec3::Y * 0.65;
                let delta = to - from;
                Transform::from_translation((from + to) * 0.5)
                    .with_rotation(Quat::from_rotation_arc(
                        Vec3::Y,
                        delta.normalize_or(Vec3::Y),
                    ))
                    .with_scale(Vec3::new(0.018, delta.length(), 0.018))
            });
        marker(&mut c, entity, &mut marks.link, &a.chip, &a.amber, link);
    }
}
