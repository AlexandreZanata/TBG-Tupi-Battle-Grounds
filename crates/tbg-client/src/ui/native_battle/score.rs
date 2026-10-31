//! Mirrored heraldic Stronghold plates, segmented HP bars and central match clock.

use super::{
    super::{hud::Hud, kit},
    chrome,
};
use crate::sync::{LatestView, MatchInfo};
use bevy::prelude::*;
use tbg_core::{content::TICK_HZ, Phase};

#[derive(Component)]
enum Readout {
    Clock,
    Phase,
    Hp(usize),
    Tower(usize),
}

pub fn spawn(parent: &mut ChildSpawnerCommands, info: &MatchInfo, assets: &AssetServer) {
    parent
        .spawn(Node {
            position_type: PositionType::Absolute,
            top: px(12),
            left: percent(22),
            width: percent(56),
            height: px(76),
            align_items: AlignItems::Center,
            ..default()
        })
        .with_children(|row| {
            plate(row, info, assets, 0);
            row.spawn((
                Node {
                    width: px(116),
                    height: px(76),
                    flex_shrink: 0.,
                    border: UiRect::all(px(3)),
                    border_radius: BorderRadius::all(percent(50)),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                chrome::frame(chrome::GOLD),
                ZIndex(2),
            ))
            .with_children(|clock| {
                clock.spawn((chrome::title("0:00", 29.), Readout::Clock));
                clock.spawn((kit::muted("GET READY", 8.), Readout::Phase));
            });
            plate(row, info, assets, 1);
        });
}

fn plate(
    parent: &mut ChildSpawnerCommands,
    info: &MatchInfo,
    assets: &AssetServer,
    relative: usize,
) {
    let own = relative == 0;
    let name = if own { &info.you_name } else { &info.opponent };
    parent
        .spawn((
            Node {
                flex_grow: 1.,
                flex_basis: px(0),
                min_width: px(0),
                height: px(55),
                padding: UiRect::axes(px(8), px(4)),
                border: UiRect::all(px(2)),
                border_radius: BorderRadius::all(px(7)),
                column_gap: px(10),
                align_items: AlignItems::Center,
                flex_direction: if own {
                    FlexDirection::Row
                } else {
                    FlexDirection::RowReverse
                },
                ..default()
            },
            chrome::frame(chrome::GOLD),
        ))
        .with_children(|plate| {
            plate
                .spawn((
                    Node {
                        width: px(38),
                        height: px(44),
                        flex_shrink: 0.,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        padding: UiRect::bottom(px(7)),
                        ..default()
                    },
                    ImageNode::new(assets.load(format!(
                        "battle/native/crest-{}.webp",
                        if own { "ally" } else { "enemy" }
                    ))),
                ))
                .with_children(|crest| {
                    crest.spawn(chrome::title(
                        name.chars()
                            .next()
                            .unwrap_or('?')
                            .to_uppercase()
                            .to_string(),
                        20.,
                    ));
                });
            plate
                .spawn(Node {
                    flex_grow: 1.,
                    flex_basis: px(0),
                    min_width: px(0),
                    flex_direction: FlexDirection::Column,
                    row_gap: px(4),
                    ..default()
                })
                .with_children(|body| {
                    body.spawn(Node {
                        width: percent(100),
                        justify_content: JustifyContent::SpaceBetween,
                        align_items: AlignItems::Center,
                        flex_direction: if own {
                            FlexDirection::Row
                        } else {
                            FlexDirection::RowReverse
                        },
                        ..default()
                    })
                    .with_children(|line| {
                        line.spawn((
                            Node {
                                max_width: percent(68),
                                overflow: Overflow::clip(),
                                ..default()
                            },
                            chrome::title(name.clone(), 14.),
                            TextLayout::no_wrap(),
                        ));
                        line.spawn((kit::muted("", 9.), Readout::Tower(relative)));
                    });
                    vitality(body, relative);
                });
        });
}

fn vitality(parent: &mut ChildSpawnerCommands, relative: usize) {
    let own = relative == 0;
    let (top, bottom) = if own {
        (Color::srgb_u8(95, 212, 255), Color::srgb_u8(10, 111, 180))
    } else {
        (Color::srgb_u8(255, 128, 133), Color::srgb_u8(180, 20, 44))
    };
    parent
        .spawn((
            Node {
                width: percent(100),
                height: px(20),
                overflow: Overflow::clip(),
                border_radius: BorderRadius::all(px(3)),
                border: UiRect::all(px(1)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgb_u8(6, 11, 14)),
            BorderColor::all(Color::srgb_u8(94, 75, 49)),
        ))
        .with_children(|track| {
            track.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: if own { px(0) } else { Val::Auto },
                    right: if own { Val::Auto } else { px(0) },
                    width: percent(100),
                    height: percent(100),
                    ..default()
                },
                chrome::gradient(top, bottom),
                Hud::Hp(relative),
            ));
            for i in 1..10 {
                track.spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        left: percent(i as f32 * 10.),
                        width: px(1),
                        height: percent(100),
                        ..default()
                    },
                    BackgroundColor(Color::BLACK.with_alpha(0.35)),
                ));
            }
            track.spawn((
                super::super::label("", 12.),
                TextShadow {
                    offset: Vec2::new(0., 1.),
                    color: Color::BLACK,
                },
                Readout::Hp(relative),
            ));
        });
}

pub fn update(
    view: Res<LatestView>,
    info: Res<MatchInfo>,
    mut texts: Query<(&Readout, &mut Text)>,
) {
    let Some(v) = &view.0 else { return };
    for (kind, mut text) in &mut texts {
        let seat = |relative| {
            if relative == 0 {
                v.you.index()
            } else {
                1 - v.you.index()
            }
        };
        let want = match kind {
            Readout::Clock => {
                let s = v.elapsed / TICK_HZ;
                format!("{}:{:02}", s / 60, s % 60)
            }
            Readout::Phase => if v.phase == Phase::Countdown {
                "GET READY"
            } else if v.augment_round.is_some() {
                "PAUSED"
            } else if info.ranked {
                "RANKED"
            } else {
                "BATTLE"
            }
            .into(),
            Readout::Hp(relative) => {
                let i = seat(*relative);
                format!(
                    "{} / {}",
                    v.stronghold_hp[i].max(0),
                    v.strongholds[i].max_hp
                )
            }
            Readout::Tower(relative) => {
                let tower = v.strongholds[seat(*relative)];
                format!("{:?} T{}", tower.kind, tower.tier)
            }
        };
        if text.0 != want {
            text.0 = want;
        }
    }
}
