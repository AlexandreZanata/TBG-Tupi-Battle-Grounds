//! Cached raw figures for review, independent of live-match animation cues and HP bars.

use super::{layout::Layout, settings::Gallery, Cycle};
use crate::art::{
    anim::{Animator, Clip, CLIPS},
    figure, FigureLibrary, FORMS,
};
use bevy::prelude::*;
use tbg_core::{UnitKind, UNIT_KINDS};

pub fn spawn(
    commands: &mut Commands,
    lib: &FigureLibrary,
    options: &Gallery,
    mut mats: ResMut<Assets<StandardMaterial>>,
    window: &Window,
) {
    let material = if options.silhouette {
        mats.add(StandardMaterial {
            base_color: Color::BLACK,
            unlit: true,
            ..default()
        })
    } else {
        lib.material.clone()
    };
    let unit = options.unit.unwrap_or(UnitKind::Archer);
    let clips = options
        .clip
        .map_or_else(|| CLIPS.to_vec(), |clip| vec![clip]);
    if options.frames {
        for (i, clip) in clips.into_iter().enumerate() {
            for quarter in 0..4 {
                let cols = if window.width() > window.height() {
                    2
                } else {
                    1
                };
                let pos = Vec3::new(
                    (i % cols * 4 + quarter) as f32 * 1.6,
                    0.0,
                    -((i / cols) as f32) * 2.0,
                );
                let mut a = spawn_one(commands, lib, unit, (0, 0), pos, &material);
                a.1.preview = Some((clip, clip.duration() * quarter as f32 / 4.0));
                commands
                    .entity(a.0)
                    .insert((a.1, Layout::Frame(i, quarter)));
            }
        }
    } else if options.silhouette || options.unit.is_none() {
        roster(commands, lib, options, &material);
    } else {
        for (i, form) in FORMS.into_iter().enumerate() {
            let cols = if window.width() > window.height() {
                5
            } else {
                2
            };
            let pos = Vec3::new((i % cols) as f32 * 2.0, 0.0, -((i / cols) as f32) * 2.2);
            let (root, mut a) = spawn_one(commands, lib, unit, form, pos, &material);
            a.rate = options.slow;
            commands.entity(root).insert((
                a,
                Layout::Form(i),
                Cycle {
                    offset: 0,
                    shown: None,
                },
            ));
        }
    }
    title(commands, options, unit);
}
fn title(commands: &mut Commands, options: &Gallery, unit: UnitKind) {
    let title = if options.frames {
        format!(
            "{unit:?} | CONTACT SHEET | 0 / 25 / 50 / 75 %\n{}",
            options.clip.map_or_else(
                || "Idle / Walk | Melee / Ranged | Cast / Hit | Death / Spawn | Evolve / Victory"
                    .into(),
                |c| format!("{c:?}")
            )
        )
    } else {
        format!("P15 GALLERY | {} | {}x\nBase / A1 / A2 / B1 / B2 | Idle, Walk, attacks, cast, hit, death, spawn, evolve, victory", options.mode, options.slow)
    };
    commands.spawn((
        Text::new(title),
        TextFont {
            font_size: FontSize::Px(16.0),
            ..default()
        },
        TextColor(crate::art::palette::INK),
        Node {
            position_type: PositionType::Absolute,
            top: px(12),
            left: px(12),
            max_width: percent(94),
            ..default()
        },
    ));
}
fn roster(
    commands: &mut Commands,
    lib: &FigureLibrary,
    options: &Gallery,
    material: &Handle<StandardMaterial>,
) {
    for (col, kind) in UNIT_KINDS.into_iter().enumerate() {
        let forms = if options.silhouette {
            &FORMS[..1]
        } else {
            &FORMS[..]
        };
        for (row, &form) in forms.iter().enumerate() {
            let pos = Vec3::new(0.6 + col as f32 * 2.1, 0.0, -2.8 + row as f32 * 1.4);
            let (root, mut a) = spawn_one(commands, lib, kind, form, pos, material);
            a.rate = options.slow;
            if options.silhouette {
                a.preview = Some((Clip::Idle, 0.0));
            }
            commands.entity(root).insert((
                a,
                Cycle {
                    offset: col + row,
                    shown: None,
                },
            ));
        }
    }
}
fn spawn_one(
    commands: &mut Commands,
    lib: &FigureLibrary,
    kind: UnitKind,
    form: (u8, u8),
    pos: Vec3,
    material: &Handle<StandardMaterial>,
) -> (Entity, Animator) {
    let form = lib.get(kind, form.0, form.1);
    info!("gallery {kind:?}: {} triangles", form.triangles);
    let root = commands
        .spawn((Transform::from_translation(pos), Visibility::default()))
        .id();
    let figure = commands
        .spawn((
            Transform::from_rotation(Quat::from_rotation_y(25f32.to_radians()))
                .with_scale(Vec3::splat(form.plan.scale)),
            Visibility::default(),
        ))
        .id();
    let joints = figure::spawn_joints(commands, figure, &form.plan, &form.meshes, material);
    commands.entity(root).add_child(figure);
    (
        root,
        Animator::new(joints, form.plan, figure, kind, root.index_u32()),
    )
}
