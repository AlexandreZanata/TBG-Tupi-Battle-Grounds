//! Landscape home preserves Linux modes, collection artwork and live/news entry points.

use crate::{
    i18n::Lang,
    ui::{
        account::Account,
        kit,
        shell::{data::Data, mobile_chrome as chrome, Action, Panel},
        UiAction,
    },
};
use bevy::prelude::*;

pub fn spawn(
    root: &mut ChildSpawnerCommands,
    assets: &AssetServer,
    lang: Lang,
    account: &Account,
    data: &Data,
) {
    root.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: px(8),
            top: px(64),
            bottom: px(8),
            width: percent(30),
            min_width: px(156),
            max_width: px(250),
            flex_direction: FlexDirection::Column,
            row_gap: px(6),
            padding: UiRect::all(px(6)),
            overflow: Overflow::scroll_y(),
            border: UiRect::all(px(1)),
            ..default()
        },
        BackgroundColor(chrome::METAL),
        BorderColor::all(chrome::EDGE),
    ))
    .with_children(|modes| {
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
            ("practice", "bot", UiAction::PracticeSetup),
        ] {
            let title = if key == "ranked" {
                lang.key("lobby.mode.play_online")
            } else {
                lang.key(&format!("lobby.mode.{key}.title"))
            };
            let hint = if key == "ranked" && account.profile.is_none() {
                lang.key("lobby.mode.ranked_hint_guest")
            } else {
                lang.key(&format!("lobby.mode.{key}.hint"))
            };
            let mut mode = modes.spawn((
                Node {
                    width: percent(100),
                    min_width: px(0),
                    min_height: px(56),
                    flex_shrink: 0.,
                    align_items: AlignItems::Center,
                    column_gap: px(8),
                    padding: UiRect::axes(px(8), px(5)),
                    border: UiRect::all(px(1)),
                    border_radius: BorderRadius::all(px(4)),
                    ..default()
                },
                chrome::skin(chrome::METAL, true, key == "ranked"),
                action,
            ));
            if key == "room" {
                mode.insert(Action::Open(Panel::Friends));
            }
            mode.with_children(|m| {
                m.spawn(chrome::icon(assets, icon, 24., kit::GOLD));
                m.spawn(Node {
                    flex_grow: 1.,
                    flex_basis: px(0),
                    min_width: px(0),
                    flex_direction: FlexDirection::Column,
                    row_gap: px(2),
                    ..default()
                })
                .with_children(|copy| {
                    copy.spawn(kit::heading(title, 16.));
                    copy.spawn(kit::muted(hint, 12.));
                });
            });
        }
        modes.spawn(kit::muted(lang.key("android.offline"), 12.));
    });
    collection(root, assets, lang);
    root.spawn((
        Node {
            position_type: PositionType::Absolute,
            right: px(8),
            top: px(64),
            width: percent(30),
            max_height: px(130),
            min_width: px(0),
            padding: UiRect::all(px(8)),
            flex_direction: FlexDirection::Column,
            row_gap: px(4),
            border: UiRect::all(px(1)),
            overflow: Overflow::scroll_y(),
            ..default()
        },
        BackgroundColor(chrome::METAL),
        BorderColor::all(chrome::EDGE),
    ))
    .with_children(|news| {
        let online = data.live.as_ref().map_or_else(
            || lang.key("live.unavailable"),
            |live| format!("{} · {}", lang.key("android.live"), live.players),
        );
        news.spawn(crate::ui::shell::button(
            online,
            Action::Open(Panel::Live),
            0.,
        ));
        news.spawn(kit::heading(lang.key("lobby.news.title"), 14.));
        news.spawn(crate::ui::shell::button(
            lang.key("patches.title"),
            Action::Open(Panel::PatchNotes),
            0.,
        ));
    });
}

fn collection(root: &mut ChildSpawnerCommands, assets: &AssetServer, lang: Lang) {
    root.spawn(Node {
        position_type: PositionType::Absolute,
        left: percent(33),
        right: px(8),
        bottom: px(8),
        height: px(104),
        column_gap: px(6),
        ..default()
    })
    .with_children(|row| {
        for (key, page) in [
            ("arcanas", Some(Panel::Arcanas)),
            ("strongholds", Some(Panel::Strongholds)),
            ("troops", Some(Panel::Troops)),
            ("ranking", None),
        ] {
            let mut card = row.spawn((
                Node {
                    flex_grow: 1.,
                    flex_basis: px(0),
                    min_width: px(48),
                    height: percent(100),
                    flex_direction: FlexDirection::Column,
                    border: UiRect::all(px(1)),
                    overflow: Overflow::clip(),
                    ..default()
                },
                chrome::skin(chrome::METAL, true, false),
                if page.is_some() {
                    UiAction::Disabled
                } else {
                    UiAction::OpenRanking
                },
            ));
            if let Some(page) = page {
                card.insert(Action::Open(page));
            }
            card.with_children(|card| {
                card.spawn((
                    ImageNode::new(assets.load(format!("lobby/{key}-v2.webp"))),
                    Node {
                        width: percent(100),
                        height: px(70),
                        flex_shrink: 0.,
                        ..default()
                    },
                ));
                card.spawn((
                    kit::heading(lang.key(&format!("lobby.feature.{key}.title")), 14.),
                    Node {
                        padding: UiRect::all(px(4)),
                        ..default()
                    },
                ));
            });
        }
    });
}
