//! Uniform gold-framed native Shop cards, with authoritative offers and prices.

use super::{
    super::{hud::Hud, hud_update::buyable, kit, PendingBuy, UiAction},
    chrome,
};
use crate::{sync::LatestView, GameState};
use bevy::prelude::*;
use tbg_core::content::{def, ENERGY_SCALE};

#[derive(Component)]
struct Surface(usize);
#[derive(Component)]
enum Readout {
    Name(usize),
    Role(usize),
    Cost(usize),
}

pub fn plugin(app: &mut App) {
    app.add_systems(Update, update.run_if(in_state(GameState::InMatch)));
}

pub fn spawn(parent: &mut ChildSpawnerCommands, assets: &AssetServer) {
    parent
        .spawn(Node {
            column_gap: px(8),
            ..default()
        })
        .with_children(|row| {
            for i in 0..5 {
                card(row, assets, i);
            }
        });
}

fn card(parent: &mut ChildSpawnerCommands, assets: &AssetServer, i: usize) {
    parent
        .spawn((
            Node {
                width: px(166),
                height: px(225),
                flex_shrink: 0.,
                ..default()
            },
            ImageNode {
                image_mode: NodeImageMode::Stretch,
                ..ImageNode::new(assets.load("battle/native/shop-card.webp"))
            },
            UiAction::Buy(i as u8),
            Surface(i),
        ))
        .with_children(|card| {
            card.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    top: px(9),
                    left: px(7),
                    width: px(152),
                    height: px(137),
                    ..default()
                },
                ImageNode::default(),
                Hud::Portrait(i),
            ));
            card.spawn(Node {
                position_type: PositionType::Absolute,
                left: px(12),
                right: px(10),
                top: px(147),
                height: px(22),
                overflow: Overflow::clip(),
                ..default()
            })
            .with_children(|title| {
                title.spawn((
                    chrome::title("", 16.),
                    TextLayout::no_wrap(),
                    Readout::Name(i),
                ));
            });
            card.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: px(12),
                    top: px(170),
                    ..default()
                },
                kit::muted("", 11.),
                Readout::Role(i),
            ));
            card.spawn(Node {
                position_type: PositionType::Absolute,
                left: px(9),
                right: px(9),
                bottom: px(8),
                height: px(29),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                column_gap: px(7),
                ..default()
            })
            .with_children(|cost| {
                cost.spawn((
                    Node {
                        width: px(16),
                        height: px(16),
                        border: UiRect::all(px(2)),
                        border_radius: BorderRadius::all(percent(50)),
                        ..default()
                    },
                    BorderColor::all(Color::srgb_u8(255, 228, 162)),
                    chrome::gradient(Color::srgb_u8(255, 233, 137), Color::srgb_u8(181, 115, 13)),
                ));
                cost.spawn(kit::muted("Cost", 12.));
                cost.spawn((chrome::title("", 19.), Readout::Cost(i)));
            });
            super::reserve::spawn(card, assets, i);
        });
}

fn update(
    view: Res<LatestView>,
    pending: Res<PendingBuy>,
    mut surfaces: Query<(&Surface, &mut ImageNode)>,
    mut texts: Query<(&Readout, &mut Text)>,
) {
    let Some(v) = &view.0 else { return };
    for (surface, mut image) in &mut surfaces {
        let available = buyable(v, surface.0)
            && pending
                .0
                .is_none_or(|(slot, _)| slot != u8::MAX && usize::from(slot) != surface.0);
        image.color = if available {
            Color::WHITE
        } else {
            Color::srgb(0.65, 0.65, 0.65)
        };
    }
    for (field, mut text) in &mut texts {
        text.0 = match field {
            Readout::Name(i) => def(v.offers[*i]).name.into(),
            Readout::Role(i) => format!("{:?}", def(v.offers[*i]).archetype),
            Readout::Cost(i) => (def(v.offers[*i]).cost / ENERGY_SCALE).to_string(),
        };
    }
}
