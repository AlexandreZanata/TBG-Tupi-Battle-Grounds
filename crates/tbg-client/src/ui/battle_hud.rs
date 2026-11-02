//! Which battle presentation shows: the Bevy HUD everywhere since P72 (the
//! browser's HTML battle layer was removed); `DesktopBattle` stays false.

use bevy::prelude::*;

/// Whether the HTML battle layer is showing (desktop landscape).
#[derive(Resource, Default)]
pub(crate) struct DesktopBattle(pub bool);

pub fn plugin(app: &mut App) {
    app.init_resource::<DesktopBattle>()
        .add_systems(Update, hide_fallback);
}

fn hide_fallback(
    desktop: Res<DesktopBattle>,
    state: Res<State<crate::GameState>>,
    mut hud: Query<&mut Visibility, With<super::hud::HudRoot>>,
    mut stats: Query<
        &mut Visibility,
        (
            With<crate::overlay::StatsText>,
            Without<super::hud::HudRoot>,
        ),
    >,
) {
    for mut visibility in &mut hud {
        *visibility = if desktop.0 {
            Visibility::Hidden
        } else {
            Visibility::Inherited
        };
    }
    let debug = crate::config::url_param("measure").is_some();
    let native_menu = cfg!(tbg_native_ui) && *state.get() == crate::GameState::Menu;
    for mut visibility in &mut stats {
        *visibility = if native_menu
            || *state.get() == crate::GameState::Gallery
            || ((desktop.0 || cfg!(target_os = "android")) && !debug)
        {
            Visibility::Hidden
        } else {
            Visibility::Inherited
        };
    }
}
