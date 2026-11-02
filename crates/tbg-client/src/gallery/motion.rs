//! Opt-in continuous review of four champions and five forms through the normal Animator.

use super::{figures::spawn_one, polish};
use crate::{
    art::{
        anim::{Animator, Clip, CLIPS},
        FigureLibrary, FORMS,
    },
    config,
};
use bevy::prelude::*;
use tbg_core::UnitKind;

#[derive(Resource)]
pub(crate) struct Motion {
    pub units: Vec<UnitKind>,
    pub schedule: Vec<(Clip, f32)>,
    pub elapsed: f32,
    pub started: bool,
    pub frames: u32,
    shown: Option<usize>,
}

pub fn spawn(commands: &mut Commands, lib: &FigureLibrary) {
    let names = config::url_param("review_units").unwrap_or("Trident".into());
    let units: Vec<_> = names
        .split(',')
        .take(4)
        .filter_map(|name| {
            tbg_core::FULL_POOL
                .into_iter()
                .find(|kind| format!("{kind:?}") == name)
        })
        .collect();
    let mut schedule = Vec::new();
    for clip in CLIPS {
        let duration = units
            .iter()
            .flat_map(|kind| FORMS.map(|form| polish::duration(*kind, form, clip)))
            .fold(0.0_f32, f32::max);
        schedule.push((
            clip,
            if clip.looping() {
                duration * 2.0 + 0.2
            } else {
                duration + 0.25
            },
        ));
        // Exercise recovery and explicitly release looping Victory before returning to locomotion.
        schedule.push((Clip::Idle, 0.35));
    }
    for (row, &kind) in units.iter().enumerate() {
        for (column, form) in FORMS.into_iter().enumerate() {
            let (root, animator) = spawn_one(
                commands,
                lib,
                kind,
                form,
                Vec3::new(column as f32 * 5.6, -(row as f32) * 5.6, 0.0),
                &lib.material,
            );
            commands.entity(root).insert(animator);
        }
    }
    commands.insert_resource(Motion {
        units,
        schedule,
        elapsed: 0.0,
        started: false,
        frames: 0,
        shown: None,
    });
}

pub fn camera() -> (Vec3, f32, f32, Vec3) {
    (
        Vec3::new(11.2, -7.4, 0.0),
        14.5,
        11.0,
        Vec3::new(0.0, 0.7, 10.0),
    )
}

pub fn update(
    time: Res<Time>,
    motion: Option<ResMut<Motion>>,
    mut animators: Query<&mut Animator>,
) {
    let Some(mut motion) = motion else { return };
    motion.frames += 1;
    if !motion.started {
        return;
    }
    motion.elapsed += time.delta_secs();
    let mut start = 0.0;
    let selected = motion
        .schedule
        .iter()
        .enumerate()
        .find_map(|(index, &(clip, length))| {
            start += length;
            (motion.elapsed < start).then_some((index, clip))
        });
    let Some((index, clip)) = selected else {
        return;
    };
    if motion.shown == Some(index) {
        return;
    }
    motion.shown = Some(index);
    info!(
        "P70_MOTION step={index} clip={clip:?} elapsed={}",
        motion.elapsed
    );
    for mut animator in &mut animators {
        match clip {
            Clip::Idle | Clip::Walk => {
                animator.finish_action();
                animator.set_base(clip);
            }
            _ => animator.play(clip),
        }
    }
}

/// A nameless four-champion page uses the unchanged match camera and arena.
pub fn spawn_lineup(commands: &mut Commands, lib: &FigureLibrary) {
    let names = config::url_param("review_units").unwrap_or("Trident".into());
    for (column, name) in names.split(',').take(4).enumerate() {
        let Some(kind) = tbg_core::FULL_POOL
            .into_iter()
            .find(|k| format!("{k:?}") == name)
        else {
            continue;
        };
        let (root, mut animator) = spawn_one(
            commands,
            lib,
            kind,
            (0, 0),
            Vec3::new(2.0 + column as f32 * 4.0, 0.0, 0.0),
            &lib.material,
        );
        commands.entity(animator.figure).insert(
            Transform::from_rotation(Quat::from_rotation_y(std::f32::consts::FRAC_PI_2))
                .with_scale(Vec3::splat(animator.plan.scale)),
        );
        animator.present(Clip::Idle, 0.4);
        commands.entity(root).insert(animator);
    }
}
