//! Landscape lobby retains desktop art and navigation with thumb-sized controls.

use crate::{
    i18n::Lang,
    ui::{
        kit,
        shell::{Action, Panel},
        UiAction,
    },
};
use bevy::prelude::*;

pub fn header(
    root: &mut ChildSpawnerCommands,
    assets: &AssetServer,
    lang: Lang,
    account: &crate::ui::account::Account,
) {
    root.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: px(12),
            right: px(12),
            top: px(4),
            height: px(64),
            align_items: AlignItems::Center,
            column_gap: px(10),
            border: UiRect::bottom(px(1)),
            ..default()
        },
        BackgroundColor(kit::PANEL),
        BorderColor::all(kit::GOLD),
    ))
    .with_children(|bar| {
        bar.spawn((
            ImageNode::new(assets.load("lobby/wordmark.webp")),
            Node {
                width: px(140),
                height: px(62),
                ..default()
            },
            UiAction::Disabled,
            Action::Open(Panel::Lobby),
        ));
        for (key, panel) in [
            ("lobby.feature.troops.title", Panel::Troops),
            ("android.arenas", Panel::Arenas),
            ("android.live", Panel::Live),
            ("lobby.settings.title", Panel::Settings),
        ] {
            bar.spawn(crate::ui::shell::button(
                lang.key(key),
                Action::Open(panel),
                74.,
            ));
        }
        bar.spawn(Node {
            flex_grow: 1.,
            ..default()
        });
        if account.profile.is_none() {
            bar.spawn(kit::button(
                lang.key("android.create"),
                UiAction::CreateAccount,
                86.,
            ));
        }
        bar.spawn(kit::button(
            lang.key("android.sign_in"),
            UiAction::HaveKey,
            86.,
        ));
    });
}

pub fn home(root: &mut ChildSpawnerCommands, assets: &AssetServer, lang: Lang) {
    root.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: px(16),
            top: px(88),
            width: px(220),
            bottom: px(22),
            flex_direction: FlexDirection::Column,
            row_gap: px(8),
            overflow: Overflow::scroll_y(),
            padding: UiRect::all(px(14)),
            border: UiRect::all(px(1)),
            ..default()
        },
        BackgroundColor(kit::PANEL),
        BorderColor::all(kit::GOLD),
    ))
    .with_children(|play| {
        play.spawn(kit::heading(lang.key("android.play"), 24.));
        play.spawn(kit::button(
            lang.key("lobby.mode.practice.title"),
            UiAction::PracticeSetup,
            190.,
        ));
        play.spawn(kit::button(
            lang.key("lobby.mode.casual.title"),
            UiAction::Quick,
            190.,
        ));
        play.spawn(crate::ui::shell::button(
            lang.key("lobby.friend.title"),
            Action::Open(Panel::Friends),
            190.,
        ));
        play.spawn(kit::muted(lang.key("android.offline"), 12.));
    });
    root.spawn(Node {
        position_type: PositionType::Absolute,
        right: px(16),
        bottom: px(26),
        width: percent(65),
        flex_direction: FlexDirection::Column,
        row_gap: px(8),
        ..default()
    })
    .with_children(|collection| {
        collection.spawn(kit::heading(lang.key("android.collection"), 17.));
        collection
            .spawn(Node {
                column_gap: px(10),
                ..default()
            })
            .with_children(|row| {
                for (key, panel) in [
                    ("arcanas", Some(Panel::Arcanas)),
                    ("strongholds", Some(Panel::Strongholds)),
                    ("troops", Some(Panel::Troops)),
                    ("ranking", None),
                ] {
                    let mut tile = row.spawn((
                        Node {
                            flex_grow: 1.,
                            flex_basis: px(0),
                            min_width: px(48),
                            height: px(112),
                            flex_direction: FlexDirection::Column,
                            border: UiRect::all(px(1)),
                            ..default()
                        },
                        BackgroundColor(kit::PANEL),
                        BorderColor::all(kit::GOLD),
                        if panel.is_some() {
                            UiAction::Disabled
                        } else {
                            UiAction::OpenRanking
                        },
                    ));
                    if let Some(panel) = panel {
                        tile.insert(Action::Open(panel));
                    }
                    tile.with_children(|tile| {
                        tile.spawn((
                            ImageNode::new(assets.load(format!("lobby/{key}-v2.webp"))),
                            Node {
                                width: percent(100),
                                height: px(74),
                                ..default()
                            },
                        ));
                        tile.spawn((
                            kit::heading(lang.key(&format!("lobby.feature.{key}.title")), 14.),
                            Node {
                                margin: UiRect::all(px(6)),
                                ..default()
                            },
                        ));
                    });
                }
            });
    });
    root.spawn(Node {
        position_type: PositionType::Absolute,
        right: px(20),
        top: px(84),
        width: percent(34),
        flex_direction: FlexDirection::Column,
        row_gap: px(8),
        ..default()
    })
    .with_children(|news| {
        news.spawn(kit::heading(lang.key("lobby.news.title"), 18.));
        news.spawn(crate::ui::shell::button(
            lang.key("patches.title"),
            Action::Open(Panel::PatchNotes),
            160.,
        ));
    });
}
