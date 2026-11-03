//! Landscape laboratory toolbar reserves separate rows for status and thumb controls.

use super::*;

pub(super) fn toolbar(root: &mut ChildSpawnerCommands, lang: Lang, id: SessionId) {
    root.spawn((
        Node {
            width: percent(100),
            flex_shrink: 0.,
            padding: UiRect::all(px(6)),
            flex_direction: FlexDirection::Column,
            row_gap: px(4),
            ..default()
        },
        BackgroundColor(kit::PANEL),
    ))
    .with_children(|bar| {
        bar.spawn(Node {
            width: percent(100),
            column_gap: px(8),
            align_items: AlignItems::Center,
            ..default()
        })
        .with_children(|head| {
            head.spawn((
                Node {
                    max_width: percent(40),
                    min_width: px(0),
                    ..default()
                },
                kit::heading(lang.key("training.editor"), 16.),
            ));
            head.spawn((
                Node {
                    flex_grow: 1.,
                    flex_basis: px(0),
                    min_width: px(0),
                    ..default()
                },
                kit::muted("", 11.),
                Field::Clock,
            ));
            head.spawn((
                Node {
                    max_width: percent(23),
                    min_width: px(0),
                    ..default()
                },
                kit::muted("", 11.),
                Field::Status,
            ));
        });
        bar.spawn(Node {
            width: percent(100),
            column_gap: px(6),
            ..default()
        })
        .with_children(|actions| {
            for (key, action) in [
                ("play_pause", Action::Control(id, 0)),
                ("step", Action::Control(id, 1)),
                ("reset", Action::Control(id, 2)),
                ("back", Action::Back),
            ] {
                actions
                    .spawn(button(
                        lang.key(&format!("mobile.training.{key}")),
                        action,
                        0.,
                    ))
                    .insert(Node {
                        flex_grow: 1.,
                        flex_basis: px(0),
                        min_width: px(48),
                        min_height: px(48),
                        padding: UiRect::axes(px(6), px(4)),
                        border: UiRect::all(px(1)),
                        border_radius: BorderRadius::all(px(5)),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        ..default()
                    });
            }
        });
    });
}

pub(super) fn heading(parent: &mut ChildSpawnerCommands, text: String) {
    parent.spawn((
        Node {
            width: percent(100),
            min_width: px(0),
            flex_shrink: 0.,
            ..default()
        },
        kit::heading(text, 16.),
        TextLayout::new(Justify::Left, LineBreak::AnyCharacter),
    ));
}
