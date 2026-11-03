//! `--capture PATH`: saves one screenshot a few seconds into a match, then exits.
//! Used for visual review of the Linux reference build.

use bevy::prelude::*;

use crate::app::Screen;
use crate::config::Launch;

/// Seconds in the match before the shot, so portraits, units and HUD settle.
const DELAY: f32 = 4.0;

pub fn plugin(app: &mut App) {
    app.add_systems(Update, shoot.run_if(in_state(Screen::Match)));
}

fn shoot(
    mut commands: Commands,
    launch: Res<Launch>,
    time: Res<Time>,
    mut started: Local<Option<f32>>,
    mut state: Local<u8>,
    mut exit: MessageWriter<AppExit>,
) {
    #[cfg(not(target_arch = "wasm32"))]
    {
        use bevy::render::view::screenshot::{save_to_disk, Screenshot};
        let Some(path) = launch.capture.clone() else {
            return;
        };
        let now = time.elapsed_secs();
        let start = *started.get_or_insert(now);
        match *state {
            0 if now - start >= DELAY => {
                commands
                    .spawn(Screenshot::primary_window())
                    .observe(save_to_disk(path));
                *state = 1;
            }
            // Give the asynchronous readback a moment to reach the disk.
            1 if now - start >= DELAY + 1.5 => {
                exit.write(AppExit::Success);
                *state = 2;
            }
            _ => {}
        }
    }
    #[cfg(target_arch = "wasm32")]
    let _ = (
        &mut commands,
        &launch,
        &time,
        &mut started,
        &mut state,
        &mut exit,
    );
}
