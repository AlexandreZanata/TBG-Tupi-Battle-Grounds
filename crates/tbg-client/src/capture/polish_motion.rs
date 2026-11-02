//! Native recording handshake and provenance for continuous, event-free gallery motion.

use crate::{camera::ArenaCamera, config, gallery::motion::Motion};
use bevy::prelude::*;
use serde_json::json;
use std::{fs::OpenOptions, io::Write, path::Path};

pub fn plugin(app: &mut App) {
    app.add_systems(Update, record.after(crate::gallery::motion::update));
}
fn write(path: &Path, value: serde_json::Value) -> std::io::Result<()> {
    let mut file = OpenOptions::new().write(true).create_new(true).open(path)?;
    writeln!(file, "{value}")
}
fn record(
    motion: Option<ResMut<Motion>>,
    time: Res<Time>,
    window: Single<&Window>,
    camera: Single<&Transform, With<ArenaCamera>>,
    mut stage: Local<u8>,
    mut started: Local<f64>,
    mut exit: MessageWriter<AppExit>,
) {
    let Some(mut motion) = motion else { return };
    let Some(directory) = config::url_param("capture_motion") else {
        return;
    };
    let path = Path::new(&directory);
    let result = match *stage {
        0 if motion.frames >= 30 => {
            *stage = 1;
            write(
                &path.join("ready.json"),
                json!({
                    "units": motion.units.iter().map(|k| format!("{k:?}")).collect::<Vec<_>>(),
                    "forms": ["Base","A1","A2","B1","B2"],
                    "layout": "one champion per row; five form columns; row/column pitch 5.6",
                    "viewport": [window.width() as u32, window.height() as u32], "dpi": window.scale_factor(),
                    "camera": {"translation": camera.translation.to_array(), "rotation": camera.rotation.to_array()},
                    "schedule": motion.schedule.iter().map(|(clip,d)| json!({"clip":format!("{clip:?}"),"seconds":d})).collect::<Vec<_>>(),
                    "clock": "normal Animator at rate 1; no effects, target or simulation",
                    "limitation": "Diagnostic includes unused clip slots. Consult applicability matrix. Victory action is not a Result-screen capture."
                }),
            )
        }
        1 if path.join("recording.start").exists() => {
            *stage = 2;
            *started = time.elapsed_secs_f64();
            motion.started = true;
            write(
                &path.join("started.json"),
                json!({"renderer_elapsed":*started}),
            )
        }
        2 if motion.elapsed >= motion.schedule.iter().map(|(_, d)| d).sum::<f32>() => {
            *stage = 3;
            write(
                &path.join("finished.json"),
                json!({"seconds":motion.elapsed,"frames":motion.frames}),
            )
        }
        3 => {
            exit.write(AppExit::Success);
            Ok(())
        }
        _ => Ok(()),
    };
    if let Err(error) = result {
        error!("P70 motion capture: {error}");
        exit.write(AppExit::error());
    }
}
