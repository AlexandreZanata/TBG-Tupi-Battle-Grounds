//! Linux header account card with the original player crest and clear login actions.

use super::{block, chrome, kit, Account, Lang, UiAction};
use bevy::prelude::*;

pub(super) fn spawn(
    root: &mut ChildSpawnerCommands,
    account: &Account,
    assets: &AssetServer,
    lang: Lang,
) {
    root.spawn((
        Node {
            width: percent(16),
            flex_grow: 0.,
            flex_basis: Val::Auto,
            padding: UiRect::all(px(4)),
            ..block()
        },
        BackgroundColor(chrome::METAL),
        BorderColor::all(chrome::EDGE),
    ))
    .with_children(|cell| {
        cell.spawn((
            Node {
                width: percent(100),
                height: percent(100),
                min_width: px(0),
                align_items: AlignItems::Center,
                column_gap: px(10),
                padding: UiRect::axes(px(9), px(5)),
                border: UiRect::all(px(1)),
                border_radius: BorderRadius::all(px(6)),
                overflow: Overflow::clip(),
                ..default()
            },
            BackgroundColor(kit::INK),
            BackgroundGradient::from(LinearGradient::to_bottom(vec![
                ColorStop::auto(Color::srgb_u8(34, 45, 48)),
                ColorStop::auto(Color::srgb_u8(9, 20, 27)),
            ])),
            BorderColor::all(kit::GOLD),
            BoxShadow(vec![ShadowStyle {
                color: kit::GOLD.with_alpha(0.16),
                blur_radius: px(5),
                ..default()
            }]),
        ))
        .with_children(|card| {
            avatar(card, assets);
            card.spawn(Node {
                flex_grow: 1.,
                flex_basis: px(0),
                row_gap: px(1),
                ..chrome::column()
            })
            .with_children(|copy| {
                name(copy, account, assets, lang);
                if let Some(profile) = &account.profile {
                    copy.spawn(Node {
                        align_items: AlignItems::Center,
                        column_gap: px(8),
                        min_height: px(22),
                        ..default()
                    })
                    .with_children(|rating| {
                        rating.spawn(chrome::icon(assets, "trophy", 14., kit::GOLD));
                        rating.spawn(kit::heading(profile.rating.to_string(), 14.));
                    });
                } else {
                    for (key, icon, action) in [
                        ("create", "account-add", UiAction::CreateAccount),
                        ("key", "account-key", UiAction::HaveKey),
                    ] {
                        login_action(copy, assets, lang, key, icon, action);
                    }
                }
            });
        });
    });
}

fn avatar(parent: &mut ChildSpawnerCommands, assets: &AssetServer) {
    parent
        .spawn((
            Node {
                width: px(52),
                height: px(52),
                flex_shrink: 0.,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                border: UiRect::all(px(1)),
                border_radius: BorderRadius::MAX,
                ..default()
            },
            BackgroundColor(Color::srgb_u8(7, 16, 23)),
            BorderColor::all(kit::GOLD.with_alpha(0.6)),
        ))
        .with_children(|portrait| {
            portrait.spawn(kit::image(assets, "lobby/crest-v2.webp", 36., 48.));
        });
}

fn name(parent: &mut ChildSpawnerCommands, account: &Account, assets: &AssetServer, lang: Lang) {
    parent
        .spawn((
            Node {
                width: percent(100),
                height: px(24),
                flex_shrink: 0.,
                align_items: AlignItems::Center,
                column_gap: px(8),
                border: UiRect::bottom(px(1)),
                overflow: Overflow::clip(),
                ..default()
            },
            BorderColor::all(kit::GOLD.with_alpha(0.3)),
        ))
        .with_children(|title| {
            title.spawn(chrome::icon(assets, "account-user", 14., kit::GOLD));
            title.spawn((
                kit::heading(
                    account
                        .profile
                        .as_ref()
                        .map_or_else(|| lang.key("lobby.guest.name"), |p| p.name.clone()),
                    18.,
                ),
                TextLayout::no_wrap(),
            ));
        });
}

fn login_action(
    parent: &mut ChildSpawnerCommands,
    assets: &AssetServer,
    lang: Lang,
    key: &str,
    icon: &str,
    action: UiAction,
) {
    parent
        .spawn((
            Node {
                width: percent(100),
                height: px(18),
                flex_shrink: 0.,
                align_items: AlignItems::Center,
                column_gap: px(8),
                border_radius: BorderRadius::all(px(3)),
                ..default()
            },
            chrome::skin(Color::NONE, true, false),
            action,
        ))
        .with_children(|row| {
            row.spawn(chrome::icon(assets, icon, 14., kit::GOLD));
            row.spawn((
                kit::muted(lang.key(&format!("lobby.account.{key}")), 11.),
                TextLayout::no_wrap(),
            ));
        });
}
