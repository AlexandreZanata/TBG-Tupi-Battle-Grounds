//! An explicit mobile drawer keeps both auxiliary choices clear of the five offers and Reroll.

use super::*;

#[derive(Resource, Default)]
pub(super) struct Open(pub bool);
#[derive(Component)]
struct Backdrop;

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<Open>()
        .add_systems(OnExit(GameState::InMatch), |mut open: ResMut<Open>| {
            open.0 = false
        })
        .add_systems(
            Update,
            (toggle, paint)
                .chain()
                .after(crate::ui::input::taps)
                .after(crate::ui::keyboard::navigate)
                .before(crate::ui::actions::apply)
                .run_if(in_state(GameState::InMatch)),
        );
}

fn toggle(mut queue: ResMut<crate::ui::ActionQueue>, mut open: ResMut<Open>) {
    queue.0.retain(|action| match action {
        UiAction::ToggleAuxiliaries => {
            open.0 = !open.0;
            false
        }
        UiAction::Deselect if open.0 => {
            open.0 = false;
            false
        }
        _ => true,
    });
}

fn paint(open: Res<Open>, mut panels: Query<&mut Node, With<Backdrop>>) {
    if !open.is_changed() {
        return;
    }
    for mut node in &mut panels {
        node.display = if open.0 { Display::Flex } else { Display::None };
    }
}

pub(super) fn spawn(parent: &mut ChildSpawnerCommands, assets: &AssetServer) {
    parent
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: percent(100),
                height: percent(100),
                ..default()
            },
            Hud::PrivateControls,
            GlobalZIndex(20),
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
                BackgroundColor(Color::BLACK.with_alpha(0.3)),
            ))
            .with_children(|backdrop| {
                backdrop
                    .spawn((
                        Node {
                            padding: UiRect::all(px(8)),
                            border: UiRect::all(px(2)),
                            border_radius: BorderRadius::all(px(7)),
                            flex_direction: FlexDirection::Column,
                            row_gap: px(6),
                            max_width: percent(96),
                            ..default()
                        },
                        chrome::frame(chrome::GOLD),
                    ))
                    .with_children(|panel| {
                        panel
                            .spawn(Node {
                                width: percent(100),
                                align_items: AlignItems::Center,
                                justify_content: JustifyContent::SpaceBetween,
                                ..default()
                            })
                            .with_children(|heading| {
                                heading.spawn((chrome::title("", 16.), AuxReadout::Header));
                                heading.spawn(chrome::upgrade_close(UiAction::ToggleAuxiliaries));
                            });
                        panel
                            .spawn(Node {
                                column_gap: px(4),
                                ..default()
                            })
                            .with_children(|row| {
                                for slot in 0..AUXILIARY_SLOTS {
                                    card(row, assets, slot);
                                }
                            });
                    });
            });
        });
}

pub(super) fn button(parent: &mut ChildSpawnerCommands) {
    parent
        .spawn((
            Node {
                width: px(48),
                min_width: px(48),
                height: px(48),
                flex_shrink: 0.,
                border: UiRect::all(px(2)),
                border_radius: BorderRadius::all(px(6)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            chrome::frame(chrome::GOLD),
            UiAction::ToggleAuxiliaries,
            kit::buttons::Chrome {
                enabled: true,
                focused: false,
            },
        ))
        .with_children(|button| {
            button.spawn((
                Node {
                    width: percent(100),
                    min_width: px(0),
                    ..default()
                },
                chrome::title("", 10.),
                TextLayout::justify(Justify::Center),
                AuxReadout::Toggle,
            ));
        });
}
