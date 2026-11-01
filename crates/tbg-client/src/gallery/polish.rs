//! Reusable five-form pose sheets with explicit times and deterministic P63 presentation.

use super::{
    figures::spawn_one,
    settings::{Gallery, View},
};
use crate::art::{
    anim::{Animator, Clip, CLIPS},
    kits, FigureLibrary, FORMS,
};
use bevy::prelude::*;
use serde_json::{json, Value};
use tbg_core::UnitKind;

pub const PAGE_COUNT: usize = CLIPS.len();
pub(super) const SAMPLE_COLUMNS: usize = 6;
const COLUMN_GAP: f32 = 3.5;
const ROW_GAP: f32 = 3.4;

/// Capture advances `page` only after its screenshot finishes; update then settles the new sheet.
#[derive(Resource)]
pub struct Review {
    pub page: usize,
    pub settled_frames: u32,
    applied_page: Option<usize>,
    unit: UnitKind,
    view: View,
    own: bool,
    forms: Vec<Value>,
    schedules: Vec<(f32, Vec<f32>)>,
}

impl Review {
    pub fn metadata(&self) -> Value {
        let (center, half_width, half_height, offset) = camera();
        json!({
            "unit": format!("{:?}", self.unit),
            "clip": CLIPS.get(self.page).map(|clip| format!("{clip:?}")),
            "clip_index": self.page,
            "view": self.view.label(),
            "team": if self.own { "ally" } else { "enemy" },
            "yaw_degrees": self.view.yaw(),
            "forms": self.forms,
            "evidence_kind": "event-free-pose-sheet",
            "presentation": "Animator::present",
            "secondary_motion": "rest-equilibrium; temporal capture required",
            "aim": null,
            "layout": "columns Base/A1/A2/B1/B2; rows ascending sample time",
            "camera_preset": "p70-fixed-sheet-v1",
            "camera": {
                "center": center.to_array(),
                "half_width": half_width,
                "half_height": half_height,
                "direction_offset": offset.to_array(),
                "distance": "max(half_width / (0.315 * viewport_aspect), half_height / 0.315)"
            },
            "applicability": "sampling is not evidence that every clip is used in combat"
        })
    }
}

#[derive(Component)]
pub(super) struct Sample {
    form_index: usize,
    row: usize,
}

pub(super) fn spawn(commands: &mut Commands, lib: &FigureLibrary, options: &Gallery) {
    let unit = options.unit.unwrap_or(UnitKind::Archer);
    let view = options.view.unwrap_or(View::ThreeQuarter);
    let own = crate::config::url_param("review_team").as_deref() != Some("enemy");
    for (column, form) in FORMS.into_iter().enumerate() {
        for row in 0..SAMPLE_COLUMNS {
            let (root, mut animator) = spawn_one(
                commands,
                lib,
                unit,
                form,
                Vec3::new(column as f32 * COLUMN_GAP, -(row as f32) * ROW_GAP, 0.0),
                &lib.material,
            );
            commands.entity(animator.figure).insert(
                Transform::from_rotation(Quat::from_rotation_y(view.yaw().to_radians()))
                    .with_scale(Vec3::splat(animator.plan.scale)),
            );
            // Static sheets compare equilibrium cloth; live previews retain physical secondary motion.
            for joint in animator.joints {
                commands
                    .entity(joint)
                    .remove::<crate::art::springs::SpringJoint>();
            }
            if !own {
                for (joint, mesh) in animator
                    .joints
                    .iter()
                    .zip(lib.get(unit, form.0, form.1).meshes(false))
                {
                    if let Some(mesh) = mesh {
                        commands.entity(*joint).insert(Mesh3d(mesh.clone()));
                    }
                }
            }
            animator.present(Clip::Idle, 0.0);
            commands.entity(root).insert((
                animator,
                Sample {
                    form_index: column,
                    row,
                },
            ));
        }
    }
    commands.insert_resource(Review {
        page: 0,
        settled_frames: 0,
        applied_page: None,
        unit,
        view,
        own,
        forms: Vec::new(),
        schedules: Vec::new(),
    });
}

/// Every rendered page reuses the same five forms, camera, aim and presentation clocks.
pub(super) fn update(
    review: Option<ResMut<Review>>,
    mut figures: Query<(&Sample, &mut Animator, &mut Visibility)>,
) {
    let Some(mut review) = review else { return };
    let Some(&clip) = CLIPS.get(review.page) else {
        return;
    };
    let changed = review.applied_page != Some(review.page);
    if changed {
        review.schedules = FORMS
            .into_iter()
            .map(|form| {
                let length = duration(review.unit, form, clip);
                (length, sample_times(clip, length))
            })
            .collect();
        review.applied_page = Some(review.page);
        review.settled_frames = 0;
        review.forms = FORMS.into_iter().zip(&review.schedules).map(|((tier, branch), (length, times))| {
            json!({"tier": tier, "branch": branch, "duration_s": length, "samples_s": times})
        }).collect();
        eprintln!("P70_SAMPLE {}", review.metadata());
    }
    for (sample, mut animator, mut visibility) in &mut figures {
        if let Some(&t) = review.schedules[sample.form_index].1.get(sample.row) {
            animator.present(clip, t);
            *visibility = Visibility::Inherited;
        } else {
            *visibility = Visibility::Hidden;
        }
    }
    review.settled_frames = review.settled_frames.saturating_add(1);
}

/// Same framing for all champions, forms, orientations and source revisions.
pub(super) fn camera() -> (Vec3, f32, f32, Vec3) {
    (
        Vec3::new(7.0, -7.0, 0.0),
        9.2,
        10.6,
        Vec3::new(0.0, 0.7, 10.0),
    )
}

pub(super) fn duration(unit: UnitKind, form: (u8, u8), clip: Clip) -> f32 {
    if clip == Clip::Victory {
        return crate::art::celebration::celebration_for(unit, form.0, form.1).duration;
    }
    kits::kit(unit).map_or_else(|| clip.duration(), |kit| kit.duration(clip))
}

pub(super) fn sample_times(clip: Clip, duration: f32) -> Vec<f32> {
    let mut times = vec![0.0, duration * 0.25, duration * 0.5, duration * 0.75];
    if matches!(clip, Clip::AttackMelee | Clip::AttackRanged | Clip::Cast) {
        times.extend(clip.events().iter().map(|(time, _)| *time));
    }
    if !clip.looping() {
        times.push(duration);
    }
    times.sort_by(f32::total_cmp);
    times.dedup_by(|left, right| (*left - *right).abs() < 1e-5);
    times
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn p70_pose_sheets_include_event_and_final_without_duplicate_samples() {
        // ART-DIRECTION §4: contact/release times stay fixed when recovery durations differ.
        for (clip, length, event) in [
            (Clip::AttackMelee, 0.62, 0.30),
            (Clip::AttackRanged, 0.72, 0.18),
            (Clip::Cast, 0.88, 0.22),
        ] {
            let times = sample_times(clip, length);
            assert!(times.contains(&event) && times.contains(&length));
            assert!(times.len() <= SAMPLE_COLUMNS);
            assert!(times.windows(2).all(|pair| pair[0] < pair[1]));
        }
        assert_eq!(sample_times(Clip::Death, 0.6).last(), Some(&0.6));
        assert_eq!(sample_times(Clip::Victory, 2.0), vec![0.0, 0.5, 1.0, 1.5]);
    }
}
