//! Eight gray v2 bodies and a front-facing silhouette row for the P15 review.

use super::layout::Layout;
use crate::{
    art::{anim::Animator, figure, FigureLibrary},
    config,
};
use bevy::prelude::*;
use tbg_core::UNIT_KINDS;

pub fn spawn(
    commands: &mut Commands,
    lib: &FigureLibrary,
    mut mats: ResMut<Assets<StandardMaterial>>,
    portrait: bool,
) {
    let silhouette = config::url_param("silhouette").is_some();
    let material = if silhouette {
        mats.add(StandardMaterial {
            base_color: Color::BLACK,
            unlit: true,
            ..default()
        })
    } else {
        lib.material.clone()
    };
    for (i, kind) in UNIT_KINDS.into_iter().enumerate() {
        let form = lib.blockout(kind);
        let root = commands
            .spawn((
                Transform::from_translation(if portrait && !silhouette {
                    Vec3::new(6.5 + (i % 2) as f32 * 3.0, 0.0, -3.0 + (i / 2) as f32 * 2.0)
                } else {
                    Vec3::new(0.6 + i as f32 * 2.1, 0.0, 0.0)
                }),
                Visibility::default(),
            ))
            .id();
        let facing = if silhouette { 0.0 } else { 30f32.to_radians() };
        let figure = commands
            .spawn((
                Transform::from_rotation(Quat::from_rotation_y(facing))
                    .with_scale(Vec3::splat(form.plan.scale)),
                Visibility::default(),
            ))
            .id();
        let joints =
            figure::spawn_joints(commands, figure, kind, &form.plan, &form.meshes, &material);
        let mut animator = Animator::new(joints, form.plan, figure, kind, i as u32);
        animator.native = true;
        if silhouette {
            animator.preview = Some((crate::art::anim::Clip::Idle, 0.0));
        }
        commands.entity(root).add_child(figure).insert(animator);
        if !silhouette {
            commands.entity(root).insert(Layout::Blockout(i));
        }
    }
    commands.spawn((
        Text::new("P15 BLOCKOUTS | rig v2\nCrown / Barrow / Ironworks / Wildwood"),
        TextFont {
            font_size: FontSize::Px(16.0),
            ..default()
        },
        TextColor(crate::art::palette::INK),
        Node {
            position_type: PositionType::Absolute,
            left: px(12),
            top: px(12),
            max_width: percent(94),
            ..default()
        },
    ));
}
