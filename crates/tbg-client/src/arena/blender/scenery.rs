//! Batches each Arena's Blender scenery and keeps the foundry gears synchronized.

use super::Model;
use crate::arena::{lighting, theme::Theme, ArenaScene};
use bevy::{light::NotShadowCaster, prelude::*};

#[derive(Component)]
pub(in crate::arena) struct Gear {
    rest: Quat,
    speed: f32,
}

pub(in crate::arena) fn spawn(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    theme: Theme,
    model: Model,
) {
    let terrain = materials.add(StandardMaterial {
        cull_mode: None,
        perceptual_roughness: 0.92,
        ..default()
    });
    let unlit = materials.add(StandardMaterial {
        cull_mode: None,
        unlit: true,
        ..default()
    });
    for batch in model.batches {
        let glowing = matches!(batch.name.as_str(), "water" | "light");
        let rest = batch.track[0].1;
        let mut entity = commands.spawn((
            ArenaScene,
            Name::new(batch.name.clone()),
            Mesh3d(meshes.add(batch.mesh)),
            MeshMaterial3d(if glowing {
                unlit.clone()
            } else {
                terrain.clone()
            }),
            rest,
        ));
        if glowing {
            entity.insert(NotShadowCaster);
        }
        if batch.name.starts_with("gear") {
            let speed = 7.0_f32.to_radians();
            entity.insert(Gear {
                rest: rest.rotation,
                speed: if batch.name == "gear0" {
                    speed
                } else {
                    -1.5 * speed
                },
            });
        }
    }
    lighting::spawn(commands, theme);
}

pub(in crate::arena) fn animate(time: Res<Time>, mut gears: Query<(&Gear, &mut Transform)>) {
    let seconds = if crate::config::low_motion() {
        0.0
    } else {
        time.elapsed_secs_wrapped()
    };
    for (gear, mut transform) in &mut gears {
        transform.rotation = gear.rest * Quat::from_rotation_z(-gear.speed * seconds);
    }
}
