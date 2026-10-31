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
    if options.portrait {
        let (root, mut animator) = spawn_one(commands, lib, unit, (0, 0), Vec3::ZERO, &material);
        animator.preview = Some((Clip::Idle, 0.4));
        commands.entity(root).insert(animator);
        return;
    }
    if options.lineup {
        super::lineup::spawn(
            commands,
            lib,
            unit,
            &material,
            window.height() > window.width(),
        );
    } else if options.frames {
        frames(
            commands,
            lib,
            unit,
            clips,
            &material,
            (options.tier, window.height() > window.width()),
        );
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
    super::title::spawn(commands, options, unit);
}
fn frames(
    commands: &mut Commands,
    lib: &FigureLibrary,
    unit: UnitKind,
    clips: Vec<Clip>,
    material: &Handle<StandardMaterial>,
    (tier, portrait): (u8, bool),
) {
    for (i, clip) in clips.into_iter().enumerate() {
        for quarter in 0..4 {
            let layout = Layout::Frame(i, quarter);
            let (root, mut animator) = spawn_one(
                commands,
                lib,
                unit,
                (tier, 0),
                layout.position(portrait),
                material,
            );
            animator.preview = Some((clip, clip.duration() * quarter as f32 / 4.0));
            let stage = commands
                .spawn((
                    Mesh3d(lib.stage.clone()),
                    MeshMaterial3d(lib.material.clone()),
                    Transform::from_xyz(0.0, -0.08, 0.0),
                ))
                .id();
            commands
                .entity(root)
                .add_child(stage)
                .insert((animator, layout));
        }
    }
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
pub(super) fn spawn_one(
    commands: &mut Commands,
    lib: &FigureLibrary,
    kind: UnitKind,
    (tier, branch): (u8, u8),
    pos: Vec3,
    material: &Handle<StandardMaterial>,
) -> (Entity, Animator) {
    let form = lib.get(kind, tier, branch);
    info!("gallery {kind:?}: {} triangles", form.triangles);
    let root = commands
        .spawn((Transform::from_translation(pos), Visibility::default()))
        .id();
    let figure = commands
        .spawn((
            Transform::from_rotation(Quat::from_rotation_y(yaw(kind).to_radians()))
                .with_scale(Vec3::splat(form.plan.scale)),
            Visibility::default(),
        ))
        .id();
    let joints = figure::spawn_joints(commands, figure, kind, &form.plan, &form.meshes, material);
    if *material == lib.material {
        figure::spawn_glow(commands, &joints, &form.glow, &lib.glow_material);
    }
    commands.entity(root).add_child(figure);
    let mut animator = Animator::new(joints, form.plan, figure, kind, root.index_u32());
    animator.native = form.native;
    animator.form = (tier, branch);
    // Review figures walk in place.
    animator.treadmill = true;
    (root, animator)
}

/// The default Archer turns side-on to shoot, so review it at 45° to
/// see the chest and the arrow line together; everything else keeps 25°.
fn yaw(kind: UnitKind) -> f32 {
    if kind == UnitKind::Archer {
        45.0
    } else {
        25.0
    }
}
