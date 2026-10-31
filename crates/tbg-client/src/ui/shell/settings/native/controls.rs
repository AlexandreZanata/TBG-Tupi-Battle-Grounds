//! Shared metal sections, compact buttons and aligned preference rows for Linux settings.

use super::*;
use crate::ui::shell::{catalog, lobby::chrome};

pub(super) fn section(
    parent: &mut ChildSpawnerCommands,
    assets: &AssetServer,
    title: String,
    icon: &str,
    content: impl FnOnce(&mut ChildSpawnerCommands),
) {
    parent
        .spawn(catalog::card())
        .insert(Node {
            padding: UiRect::all(px(18)),
            border: UiRect::all(px(1)),
            border_radius: BorderRadius::all(px(7)),
            flex_direction: FlexDirection::Column,
            min_width: px(0),
            row_gap: px(10),
            ..default()
        })
        .with_children(|card| {
            card.spawn((
                Node {
                    width: percent(100),
                    align_items: AlignItems::Center,
                    column_gap: px(10),
                    border: UiRect::bottom(px(1)),
                    padding: UiRect::bottom(px(10)),
                    ..default()
                },
                BorderColor::all(chrome::EDGE.with_alpha(0.45)),
            ))
            .with_children(|heading| {
                heading.spawn(chrome::icon(assets, icon, 22., kit::GOLD));
                heading.spawn(kit::heading(title, 23.));
            });
            content(card);
        });
}

pub(super) fn group() -> Node {
    Node {
        width: percent(100),
        column_gap: px(8),
        ..default()
    }
}

pub(super) fn button(label: String, selected: bool) -> impl Bundle {
    let idle = if selected {
        Color::srgb_u8(46, 66, 67)
    } else {
        Color::srgb_u8(15, 31, 39)
    };
    (
        Node {
            flex_grow: 1.,
            flex_basis: px(0),
            min_width: px(0),
            min_height: px(40),
            padding: UiRect::axes(px(10), px(8)),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            border: UiRect::all(px(1)),
            border_radius: BorderRadius::all(px(4)),
            overflow: Overflow::clip(),
            ..default()
        },
        BackgroundColor(idle),
        BorderColor::all(chrome::EDGE),
        kit::buttons::Chrome {
            enabled: true,
            focused: selected,
        },
        kit::buttons::Palette {
            idle,
            hover: Color::srgb_u8(56, 73, 65),
            pressed: Color::srgb_u8(21, 39, 44),
            border: chrome::EDGE.with_alpha(0.7),
        },
        children![(
            Text::new(label),
            TextFont::from_font_size(FontSize::Px(14.)),
            TextLayout::no_wrap(),
            TextColor(if selected {
                kit::GOLD
            } else {
                Color::srgb_u8(229, 230, 218)
            }),
        )],
    )
}

pub(super) fn row(
    parent: &mut ChildSpawnerCommands,
    title: String,
    hint: String,
    control: impl Bundle,
) {
    parent
        .spawn((Node {
            width: percent(100),
            min_width: px(0),
            min_height: px(44),
            align_items: AlignItems::Center,
            column_gap: px(14),
            ..default()
        },))
        .with_children(|row| {
            row.spawn(Node {
                flex_grow: 1.,
                flex_basis: px(0),
                min_width: px(0),
                flex_direction: FlexDirection::Column,
                row_gap: px(3),
                ..default()
            })
            .with_children(|copy| {
                copy.spawn(crate::ui::label(title, 14.));
                copy.spawn(kit::muted(hint, 11.));
            });
            row.spawn(Node {
                width: px(112),
                flex_shrink: 0.,
                ..default()
            })
            .with_children(|choice| {
                choice.spawn(control);
            });
        });
}

pub(super) fn heading(parent: &mut ChildSpawnerCommands, assets: &AssetServer, lang: Lang) {
    parent
        .spawn(kit::panel())
        .insert(Node {
            width: percent(100),
            min_height: px(82),
            flex_shrink: 0.,
            padding: UiRect::axes(px(18), px(14)),
            border: UiRect::all(px(1)),
            border_radius: BorderRadius::all(px(7)),
            align_items: AlignItems::Center,
            column_gap: px(14),
            ..default()
        })
        .insert(BorderColor::all(chrome::EDGE))
        .with_children(|header| {
            header.spawn(chrome::icon(assets, "settings", 36., kit::GOLD));
            header
                .spawn(Node {
                    flex_grow: 1.,
                    flex_direction: FlexDirection::Column,
                    row_gap: px(4),
                    ..default()
                })
                .with_children(|title| {
                    title.spawn(kit::heading(lang.key("lobby.settings.title"), 30.));
                    title.spawn(kit::muted(
                        words(
                            lang,
                            "Make this camp your own.",
                            "Deixe este acampamento do seu jeito.",
                        ),
                        13.,
                    ));
                });
            header.spawn(kit::muted(
                words(
                    lang,
                    "Changes are saved automatically",
                    "Alterações salvas automaticamente",
                ),
                11.,
            ));
        });
}
