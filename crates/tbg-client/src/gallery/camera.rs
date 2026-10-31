//! Responsive framing for palette boards, lineups, unit forms, and contact sheets.

use super::settings::Gallery;
use crate::{art::anim::CLIPS, camera::ArenaCamera};
use bevy::prelude::*;

#[allow(clippy::too_many_arguments)] // Bevy keeps content, mesh cache and camera resources explicit.
pub fn frame(
    options: Res<Gallery>,
    lib: Res<crate::art::FigureLibrary>,
    meshes: Res<Assets<Mesh>>,
    mut cached: Local<Option<(tbg_core::UnitKind, (f32, f32))>>,
    window: Single<&Window>,
    camera: Single<(&mut Transform, &mut DistanceFog), With<ArenaCamera>>,
) {
    if options.stress {
        return;
    }
    let aspect = window.width() / window.height();
    let (center, width, height, offset) = if options.portrait {
        let kind = options.unit.unwrap_or(tbg_core::UnitKind::Archer);
        let (radius, h) = cached
            .get_or_insert_with(|| {
                (
                    kind,
                    crate::art::celebration::bounds::envelope(
                        lib.get(kind, 0, 0),
                        &meshes,
                        kind,
                        0,
                        0,
                    ),
                )
            })
            .1;
        (
            Vec3::new(options.portrait_x, h * 0.45, 0.),
            radius * options.portrait_zoom,
            h * 0.57 * options.portrait_zoom,
            Vec3::new(0., 0.6, 10.),
        )
    } else {
        target(&options, aspect < 1.0)
    };
    let distance = (width / (0.315 * aspect)).max(height / 0.315);
    let offset = if options.silhouette { Vec3::Z } else { offset };
    let (mut t, mut fog) = camera.into_inner();
    *t = Transform::from_translation(center + offset.normalize() * distance)
        .looking_at(center, Vec3::Y);
    fog.falloff = FogFalloff::Linear {
        start: distance + 15.0,
        end: distance + 70.0,
    };
}
fn target(options: &Gallery, portrait: bool) -> (Vec3, f32, f32, Vec3) {
    if options.portrait {
        if matches!(
            options.unit,
            Some(
                tbg_core::UnitKind::Microdrill
                    | tbg_core::UnitKind::Duodrill
                    | tbg_core::UnitKind::Megadrill
            )
        ) {
            // Capture-only framing for low tiny summons; gameplay scale is untouched.
            return (
                Vec3::new(options.portrait_x, 0.10, 0.0),
                0.42 * options.portrait_zoom,
                0.35 * options.portrait_zoom,
                Vec3::new(1.0, 1.0, 3.0),
            );
        }
        let large = matches!(
            options.unit,
            Some(tbg_core::UnitKind::BoneBrute | tbg_core::UnitKind::SiegeWalker)
        );
        return (
            Vec3::new(options.portrait_x, if large { 1.05 } else { 0.78 }, 0.0),
            (if large { 1.6 } else { 1.05 }) * options.portrait_zoom,
            (if large { 1.35 } else { 1.05 }) * options.portrait_zoom,
            Vec3::new(0.0, 1.2, 10.0),
        );
    }
    match options.mode.as_str() {
        "palette" => (
            Vec3::new(8.0, 0.5, 1.8),
            5.8,
            5.8,
            Vec3::new(0.0, 18.0, 14.0),
        ),
        // Four rows of five forms, 3.2 apart in x and 3.6 back in z.
        "strongholds" => (
            Vec3::new(8.0, 1.3, -5.0),
            10.8,
            7.4,
            Vec3::new(0.0, 1.4, 1.0),
        ),
        "blockout" if portrait && !options.silhouette => (
            Vec3::new(8.0, 0.6, 0.0),
            3.0,
            4.5,
            Vec3::new(0.0, 14.0, 10.0),
        ),
        "blockout" => (Vec3::new(8.0, 0.7, 0.0), 9.0, 3.0, Vec3::new(0.0, 1.0, 1.0)),
        "aim" => (
            Vec3::new(8.0, 0.7, 0.0),
            12.0,
            7.0,
            Vec3::new(0.0, 9.0, 11.0),
        ),
        _ if options.range => {
            let gap = options.unit.map_or(4.0, super::range::reach);
            (
                Vec3::new(super::range::FIRST_X + gap * 0.5, 0.5, 0.3),
                gap + 3.0,
                3.4,
                Vec3::new(0.0, 1.25, 1.0),
            )
        }
        _ if options.frames => sheet(options, portrait),
        _ if options.silhouette => (Vec3::new(8.0, 0.7, -2.8), 9.0, 3.0, Vec3::Z),
        _ if options.unit.is_some() => unit(portrait),
        _ => (
            Vec3::new(8.0, 0.6, 0.0),
            10.0,
            5.5,
            Vec3::new(0.0, 1.0, 1.0),
        ),
    }
}
fn unit(portrait: bool) -> (Vec3, f32, f32, Vec3) {
    if portrait {
        (
            Vec3::new(1.0, 0.7, -2.2),
            2.3,
            4.0,
            Vec3::new(0.0, 10.0, 14.0),
        )
    } else {
        (
            Vec3::new(4.0, 0.7, 0.0),
            5.2,
            2.2,
            Vec3::new(0.0, 2.0, 10.0),
        )
    }
}
fn sheet(options: &Gallery, portrait: bool) -> (Vec3, f32, f32, Vec3) {
    if options.lineup {
        // Five forms per row; idle, walk and attack rows (`Layout::Lineup`).
        return (
            Vec3::new(4.2, -1.45, 0.0),
            5.0,
            4.5,
            Vec3::new(0.0, 1.2, 10.0),
        );
    }
    let clips = if options.clip.is_some() {
        1
    } else {
        CLIPS.len()
    };
    let cols = if portrait { 1 } else { 2 };
    let rows = clips.div_ceil(cols);
    let width = (cols.min(clips) * 4 - 1) as f32 * 1.6;
    let height = (rows - 1) as f32 * 2.0;
    (
        Vec3::new(width * 0.5, 0.65 - height * 0.5, 0.0),
        width * 0.5 + 1.0,
        height * 0.5 + 1.6,
        Vec3::new(0.0, 1.2, 10.0),
    )
}
