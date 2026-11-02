//! Regression: rendered decoration stays outside the widened paved BattleLine in every Arena.

use super::{camp, ground, life, props, theme::Theme, themes};
use crate::art::mesh;
use bevy::{ecs::system::RunSystemOnce, mesh::VertexAttributeValues, prelude::*};
use std::time::Duration;

// Owner's ARENA-CLEAR-01: protect the full five-tile road and its existing border.
const PAVED_HALF_WIDTH: f32 = 2.67;
const PAVED_MIN_X: f32 = -1.1;
const PAVED_MAX_X: f32 = 17.1;

fn intrudes(mesh: &Mesh, transform: Transform) -> bool {
    let Some(VertexAttributeValues::Float32x3(points)) = mesh.attribute(Mesh::ATTRIBUTE_POSITION)
    else {
        panic!("decoration must expose its rendered vertices");
    };
    let points: Vec<Vec3> = match mesh.indices() {
        Some(indices) => indices.iter().map(|i| Vec3::from(points[i])).collect(),
        None => points.iter().copied().map(Vec3::from).collect(),
    };
    points.chunks_exact(3).any(|triangle| {
        let world: Vec<_> = triangle
            .iter()
            .map(|p| transform.transform_point(*p))
            .collect();
        let min = world
            .iter()
            .copied()
            .fold(Vec3::splat(f32::INFINITY), Vec3::min);
        let max = world
            .iter()
            .copied()
            .fold(Vec3::splat(f32::NEG_INFINITY), Vec3::max);
        // Below-ground contact patches cannot obstruct the road. Triangle bounds
        // conservatively include tilted branches/overhangs, not just prop origins.
        max.y > 0.02
            && max.x > PAVED_MIN_X + 0.001
            && min.x < PAVED_MAX_X - 0.001
            && max.z > -PAVED_HALF_WIDTH + 0.001
            && min.z < PAVED_HALF_WIDTH - 0.001
    })
}

#[test]
fn arena_clear01_all_eight_arenas_keep_static_dressing_outside_the_wide_road() {
    assert_eq!(ground::LANE_WIDTH, 5.0, "keep the owner's widened road");
    let mut failures = Vec::new();
    for arena in tbg_core::Arena::ALL {
        let theme = Theme::from(arena);
        let mut kit = themes::build(theme);
        if theme == Theme::Forest {
            kit.props = props::scatter();
        }
        for (layer, parts) in [("props", kit.props), ("glow", kit.glow)] {
            for (i, (shape, _, transform)) in parts.into_iter().enumerate() {
                if intrudes(&mesh::build(shape), transform) {
                    failures.push(format!(
                        "{theme:?} {layer}[{i}] {shape:?} at {:?}",
                        transform.translation
                    ));
                }
            }
        }
    }
    assert!(
        failures.is_empty(),
        "road decoration intrusions:\n{}",
        failures.join("\n")
    );
}

#[test]
fn arena_clear01_fire_stays_on_its_braziers_and_animated_dressing_clears_the_road() {
    for arena in tbg_core::Arena::ALL {
        let theme = Theme::from(arena);
        let mut world = World::new();
        world.init_resource::<Assets<Mesh>>();
        world.init_resource::<Assets<StandardMaterial>>();
        world.init_resource::<Time>();
        world
            .run_system_once(
                move |mut commands: Commands,
                      mut meshes: ResMut<Assets<Mesh>>,
                      mut mats: ResMut<Assets<StandardMaterial>>| {
                    let material = mats.add(StandardMaterial::default());
                    life::spawn(&mut commands, &mut meshes, &mut mats, &material, theme);
                    themes::spawn_motion(&mut commands, &mut meshes, &material, theme);
                    if theme == Theme::Forest {
                        super::scenery::spawn(&mut commands, &mut meshes, &material);
                    }
                },
            )
            .unwrap();
        let fire: Vec<_> = world
            .query_filtered::<&Transform, With<life::Flame>>()
            .iter(&world)
            .map(|t| (t.translation.x, t.translation.z))
            .collect();
        assert_eq!(fire.len(), camp::BRAZIERS.len());
        for at in camp::BRAZIERS {
            assert!(
                fire.contains(&at),
                "{theme:?}: a flame left its pedestal at {at:?}"
            );
        }
        for _ in 0..160 {
            world
                .resource_mut::<Time>()
                .advance_by(Duration::from_millis(37));
            world.run_system_once(life::animate).unwrap();
            world.run_system_once(themes::animate).unwrap();
            world.run_system_once(super::maritime::animate).unwrap();
            let roots: Vec<_> = world
                .query_filtered::<(&Mesh3d, &Transform), Without<super::towers::TowerPiece>>()
                .iter(&world)
                .map(|(mesh, transform)| (mesh.0.clone(), *transform))
                .collect();
            let meshes = world.resource::<Assets<Mesh>>();
            for (handle, transform) in roots {
                assert!(
                    !intrudes(meshes.get(&handle).unwrap(), transform),
                    "{theme:?}: animated decoration enters the road at {:?}",
                    transform.translation
                );
            }
        }
    }
}
