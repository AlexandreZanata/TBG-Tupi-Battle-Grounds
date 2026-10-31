//! Left-side game modes retain the existing authoritative actions and account gates.

use super::{chrome, kit, Account, Action, Lang, Panel, UiAction};
use bevy::prelude::*;

pub fn spawn(root: &mut ChildSpawnerCommands, account: &Account, assets: &AssetServer, lang: Lang) {
    // Both columns end at 67%, leaving the same 1% gap above their lower panels.
    let rect = [
        2.,
        17.,
        27.5,
        if cfg!(target_os = "linux") { 50. } else { 45. },
    ];
    let mut panel = root.spawn(chrome::panel(rect));
    #[cfg(target_os = "linux")]
    panel.insert(Node {
        row_gap: px(9),
        ..chrome::panel_node(rect)
    });
    #[cfg(not(target_os = "linux"))]
    panel.insert((
        BackgroundColor(chrome::WOOD),
        Node {
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(px(16)),
            row_gap: px(9),
            border: UiRect::all(px(3)),
            ..chrome::region(2., 17., 27.5, 45.)
        },
    ));
    panel.with_children(|p| {
        for (key, icon, action) in [
            (
                "ranked",
                "swords",
                if account.profile.is_some() {
                    UiAction::Ranked
                } else {
                    UiAction::CreateAccount
                },
            ),
            ("casual", "shield", UiAction::Quick),
            ("challenge", "friends", UiAction::CreateChallenge),
            ("room", "friends", UiAction::Disabled),
            ("practice", "bot", UiAction::Practice),
        ] {
            let ranked = key == "ranked";
            let title = if ranked {
                lang.key("lobby.mode.play_online")
            } else {
                lang.key(&format!("lobby.mode.{key}.title"))
            };
            let hint = if ranked && account.profile.is_none() {
                lang.key("lobby.mode.ranked_hint_guest")
            } else {
                lang.key(&format!("lobby.mode.{key}.hint"))
            };
            let mut card = p.spawn((
                Node {
                    width: percent(100),
                    flex_grow: 1.,
                    flex_basis: px(0),
                    min_height: px(0),
                    align_items: AlignItems::Center,
                    column_gap: px(13),
                    padding: UiRect::axes(px(14), px(5)),
                    border: UiRect::all(px(if cfg!(target_os = "linux") { 1. } else { 2. })),
                    border_radius: BorderRadius::all(px(if cfg!(target_os = "linux") {
                        4.
                    } else {
                        5.
                    })),
                    ..default()
                },
                chrome::skin(mode_color(ranked), true, ranked),
                action,
            ));
            if key == "room" {
                card.insert(Action::Open(Panel::Friends));
            }
            card.with_children(|r| {
                r.spawn(chrome::icon(
                    assets,
                    icon,
                    37.,
                    if ranked && !cfg!(target_os = "linux") {
                        Color::srgb_u8(200, 236, 255)
                    } else {
                        kit::GOLD
                    },
                ));
                r.spawn(Node {
                    flex_grow: 1.,
                    flex_basis: px(0),
                    row_gap: px(3),
                    ..chrome::column()
                })
                .with_children(|copy| {
                    copy.spawn(kit::heading(title, 21.))
                        .insert(TextColor(Color::srgb_u8(245, 240, 224)));
                    copy.spawn(kit::muted(hint, 13.));
                });
                r.spawn(kit::muted(">", 28.));
            });
        }
    });
}

fn mode_color(ranked: bool) -> Color {
    if cfg!(target_os = "linux") {
        chrome::METAL
    } else if ranked {
        Color::srgb_u8(18, 88, 148)
    } else {
        chrome::WOOD
    }
}
