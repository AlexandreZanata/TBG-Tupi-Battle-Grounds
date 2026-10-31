//! Persistent creator and origin links required by the owner's visible-attribution license.

use super::{kit, UiAction};
use crate::i18n::Lang;
use bevy::prelude::*;

pub const AUTHOR: &str = "Alexandre Zanata (@AlexandreZanata)";
pub const CREATOR_URL: &str = "https://github.com/AlexandreZanata";
pub const REPOSITORY_URL: &str = "https://github.com/AlexandreZanata/TBG-Tupi-Battle-Grounds";

#[derive(Component)]
pub(in crate::ui) struct Credit;

pub fn plugin(app: &mut App) {
    app.add_systems(Startup, spawn)
        .add_systems(Update, retranslate.run_if(resource_changed::<Lang>));
}

fn retranslate(mut commands: Commands, roots: Query<Entity, With<Credit>>, lang: Res<Lang>) {
    if !crate::config::bevy_ui() {
        return;
    }
    for e in &roots {
        commands.entity(e).despawn();
    }
    build(&mut commands, *lang);
}

fn spawn(mut commands: Commands, lang: Res<Lang>) {
    #[cfg(target_arch = "wasm32")]
    if let Some(element) = web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.get_element_by_id("tbg-attribution"))
    {
        let _ = element.set_attribute("hidden", "");
        if !crate::config::bevy_ui() {
            let _ = element.remove_attribute("hidden");
        }
    }
    if crate::config::bevy_ui() {
        build(&mut commands, *lang);
    }
}

fn link(text: impl Into<String>, action: UiAction) -> impl Bundle {
    (
        Node {
            padding: UiRect::axes(px(4), px(2)),
            ..default()
        },
        action,
        children![kit::heading(text, 12.)],
    )
}

fn build(commands: &mut Commands, lang: Lang) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: px(0),
                left: px(0),
                width: percent(100),
                min_height: px(24),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                column_gap: px(6),
                ..default()
            },
            BackgroundColor(Color::srgba(0.025, 0.04, 0.05, 0.96)),
            GlobalZIndex(120),
            Credit,
        ))
        .with_children(|r| {
            r.spawn(kit::muted(
                format!("Tupi Battle Grounds · {}", lang.key("native.created_by")),
                12.,
            ));
            r.spawn(link(AUTHOR, UiAction::OpenCreator));
            r.spawn(kit::muted("·", 12.));
            r.spawn(link(
                lang.key("native.original_repository"),
                UiAction::OpenRepository,
            ));
        });
}
