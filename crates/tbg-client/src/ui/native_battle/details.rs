//! Keep complete Synergy and Augment readouts reachable without covering mobile combat.

use super::{
    super::{hud::Hud, kit, label, ActionQueue, UiAction},
    chrome,
};
use crate::{i18n::Lang, sync::LatestView, GameState};
use bevy::prelude::*;

#[derive(Resource, Default)]
struct Open(bool);
#[derive(Component)]
struct Backdrop;

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<Open>()
        .add_systems(OnExit(GameState::InMatch), |mut open: ResMut<Open>| {
            open.0 = false
        })
        .add_systems(
            Update,
            toggle
                .after(crate::ui::input::taps)
                .after(crate::ui::keyboard::navigate)
                .before(crate::ui::actions::apply)
                .run_if(in_state(GameState::InMatch)),
        );
}

fn toggle(
    mut queue: ResMut<ActionQueue>,
    mut open: ResMut<Open>,
    view: Res<LatestView>,
    mut panels: Query<&mut Node, With<Backdrop>>,
) {
    queue.0.retain(|action| match action {
        UiAction::ToggleBattleDetails => {
            open.0 = !open.0;
            false
        }
        UiAction::Deselect if open.0 => {
            open.0 = false;
            false
        }
        _ => true,
    });
    if view.0.as_ref().is_some_and(|v| v.augment_round.is_some()) {
        open.0 = false;
    }
    for mut node in &mut panels {
        node.display = if open.0 { Display::Flex } else { Display::None };
    }
}

pub fn spawn(parent: &mut ChildSpawnerCommands, lang: Lang) {
    parent
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: percent(100),
                height: percent(100),
                ..default()
            },
            Hud::PrivateControls,
            GlobalZIndex(22),
        ))
        .with_children(|root| {
            root.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    width: percent(100),
                    height: percent(100),
                    display: Display::None,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                Backdrop,
                UiAction::Disabled,
                BackgroundColor(Color::BLACK.with_alpha(0.4)),
            ))
            .with_children(|backdrop| {
                backdrop
                    .spawn((
                        Node {
                            width: percent(94),
                            max_height: percent(94),
                            padding: UiRect::all(px(8)),
                            flex_direction: FlexDirection::Column,
                            row_gap: px(8),
                            ..default()
                        },
                        chrome::frame(chrome::GOLD),
                    ))
                    .with_children(|panel| {
                        panel
                            .spawn(Node {
                                width: percent(100),
                                justify_content: JustifyContent::SpaceBetween,
                                align_items: AlignItems::Center,
                                flex_shrink: 0.,
                                ..default()
                            })
                            .with_children(|heading| {
                                heading
                                    .spawn(chrome::title(lang.key("android.battle.details"), 16.));
                                heading.spawn(chrome::upgrade_close(UiAction::ToggleBattleDetails));
                            });
                        panel
                            .spawn((
                                Node {
                                    width: percent(100),
                                    min_height: px(0),
                                    overflow: Overflow::scroll_y(),
                                    flex_direction: FlexDirection::Column,
                                    row_gap: px(12),
                                    ..default()
                                },
                                ScrollPosition::default(),
                            ))
                            .with_children(|body| {
                                for hud in [Hud::Info, Hud::Augments] {
                                    body.spawn((
                                        Node {
                                            width: percent(100),
                                            flex_shrink: 0.,
                                            ..default()
                                        },
                                        label("", 13.),
                                        hud,
                                    ));
                                }
                                body.spawn(kit::muted(lang.key("battle.shop_help"), 11.));
                            });
                    });
            });
        });
}
