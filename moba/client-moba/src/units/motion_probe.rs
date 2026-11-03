//! Opt-in native motion evidence for camera input and shared champion clips.
use super::UnitVisual;
use crate::{
    art::anim::Animator,
    camera::{CameraDrag, Rig},
    link::Feed,
};
use bevy::prelude::*;
use std::{
    fs::File,
    io::{BufWriter, Write},
};

pub fn plugin(app: &mut App) {
    let Some(path) = crate::config::url_param("motion-log") else {
        return;
    };
    match File::create(path) {
        Ok(file) => {
            app.insert_resource(Probe(BufWriter::new(file)))
                .add_systems(
                    PostUpdate,
                    record.run_if(in_state(crate::app::Screen::Match)),
                );
        }
        Err(error) => warn!("Motion log unavailable: {error}"),
    }
}
#[derive(Resource)]
struct Probe(BufWriter<File>);
fn record(
    time: Res<Time>,
    rig: Res<Rig>,
    drag: Res<CameraDrag>,
    window: Single<&Window>,
    feed: Res<Feed>,
    mut probe: ResMut<Probe>,
    units: Query<(&UnitVisual, &Transform, Option<&Animator>)>,
) {
    let t = time.elapsed_secs();
    let cursor = window.cursor_position().unwrap_or(Vec2::splat(-1.0));
    let _ = writeln!(
        probe.0,
        "camera,{t:.6},{},{:.3},{:.3},{},{:.6},{:.6},{:.6},{}",
        window.focused,
        cursor.x,
        cursor.y,
        drag.active,
        rig.focus.x,
        rig.focus.y,
        rig.distance,
        feed.curr.as_ref().map_or(0, |w| w.tick)
    );
    for (v, transform, animator) in &units {
        let Some(a) = animator else { continue };
        let _ = writeln!(
            probe.0,
            "unit,{t:.6},{},{:.6},{:.6},{:.6},{:?},{:?},{},{:.6}",
            v.id,
            transform.translation.x,
            transform.translation.z,
            v.yaw,
            v.activity,
            a.clip_time().0,
            v.target,
            v.curr.distance(v.prev)
        );
    }
    let _ = probe.0.flush();
}
