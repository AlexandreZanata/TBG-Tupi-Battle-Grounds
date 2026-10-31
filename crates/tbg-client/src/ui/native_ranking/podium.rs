//! Three equally sized podiums with shared medal, name and record plate geometry.

use super::*;

#[derive(Component)]
pub(super) struct Grove;

pub(super) fn cover(
    images: Res<Assets<Image>>,
    mut groves: Query<(&ComputedNode, &mut ImageNode), With<Grove>>,
) {
    for (node, mut image) in &mut groves {
        let Some(source) = images.get(&image.image) else {
            continue;
        };
        if node.size.y <= 0. {
            continue;
        }
        let size = source.size().as_vec2();
        let ratio = node.size.x / node.size.y;
        let crop = if size.x / size.y > ratio {
            Vec2::new(size.y * ratio, size.y)
        } else {
            Vec2::new(size.x, size.x / ratio)
        };
        image.rect = Some(Rect::from_center_size(size * 0.5, crop));
    }
}

pub(super) fn spawn(
    parent: &mut ChildSpawnerCommands,
    ranking: &Ranking,
    assets: &AssetServer,
    lang: Lang,
) {
    parent
        .spawn((
            Node {
                width: percent(100),
                height: percent(43),
                min_height: px(190),
                flex_shrink: 0.,
                overflow: Overflow::clip(),
                border_radius: BorderRadius::all(px(5)),
                ..default()
            },
            BackgroundColor(Color::srgb_u8(15, 35, 35)),
        ))
        .with_children(|stage| {
            stage.spawn((
                ImageNode {
                    image_mode: NodeImageMode::Stretch,
                    color: Color::WHITE.with_alpha(0.65),
                    ..ImageNode::new(assets.load("ranking/podium-grove.webp"))
                },
                Node {
                    position_type: PositionType::Absolute,
                    left: px(0),
                    top: px(0),
                    width: percent(100),
                    height: percent(100),
                    ..default()
                },
                Grove,
            ));
            for (rank, art) in [(2, "silver"), (1, "gold"), (3, "bronze")] {
                let leader = ranking.leaders[rank - 1].as_ref();
                let pending = ranking.page.is_none() && ranking.want.is_some();
                stage
                    .spawn(Node {
                        width: percent(100. / 3.),
                        height: percent(100),
                        align_items: AlignItems::End,
                        justify_content: JustifyContent::Center,
                        ..default()
                    })
                    .with_children(|slot| {
                        slot.spawn(Node {
                            height: percent(100),
                            width: Val::Auto,
                            aspect_ratio: Some(1.),
                            max_width: percent(100),
                            ..default()
                        })
                        .with_children(|card| {
                            card.spawn((
                                ImageNode::new(assets.load(format!("ranking/podium-{art}.webp"))),
                                Node {
                                    position_type: PositionType::Absolute,
                                    width: percent(100),
                                    height: percent(100),
                                    ..default()
                                },
                            ));
                            card.spawn((
                                Node {
                                    position_type: PositionType::Absolute,
                                    top: percent(8),
                                    left: percent(38),
                                    width: percent(24),
                                    height: percent(24),
                                    border: UiRect::all(px(2)),
                                    border_radius: BorderRadius::MAX,
                                    align_items: AlignItems::Center,
                                    justify_content: JustifyContent::Center,
                                    ..default()
                                },
                                frame(),
                                children![kit::heading(rank.to_string(), 30.)],
                            ));
                            card.spawn((
                                ImageNode {
                                    color: Color::WHITE.with_alpha(if leader.is_some() {
                                        1.
                                    } else {
                                        0.35
                                    }),
                                    ..ImageNode::new(assets.load("ranking/player-crest.webp"))
                                },
                                Node {
                                    position_type: PositionType::Absolute,
                                    top: percent(34),
                                    left: percent(34),
                                    width: percent(32),
                                    height: percent(28),
                                    ..default()
                                },
                            ));
                            plate(card, 16., 68., 68., 13., |name| {
                                name.spawn((
                                    kit::heading(
                                        leader.map_or_else(
                                            || words(lang, "Open place", "Lugar disponível"),
                                            |p| p.name.clone(),
                                        ),
                                        15.,
                                    ),
                                    TextLayout::no_wrap(),
                                    kit::Pending(pending),
                                    Node {
                                        width: percent(100),
                                        min_height: px(19),
                                        ..default()
                                    },
                                ));
                            });
                            plate(card, 12., 82., 76., 16., |stats| {
                                for (value, label) in [
                                    (
                                        leader.map_or("—".into(), |p| p.rating.to_string()),
                                        words(lang, "Rating", "Pontuação"),
                                    ),
                                    (
                                        leader.map_or("—".into(), |p| {
                                            format!("{} · {} · {}", p.wins, p.losses, p.draws)
                                        }),
                                        words(lang, "W · L · D", "V · D · E"),
                                    ),
                                ] {
                                    stats
                                        .spawn(Node {
                                            width: percent(50),
                                            align_items: AlignItems::Center,
                                            justify_content: JustifyContent::Center,
                                            flex_direction: FlexDirection::Column,
                                            overflow: Overflow::clip(),
                                            ..default()
                                        })
                                        .with_children(|field| {
                                            field.spawn((
                                                text(value, 12.),
                                                kit::Pending(pending),
                                                Node {
                                                    width: percent(90),
                                                    min_height: px(15),
                                                    ..default()
                                                },
                                            ));
                                            field.spawn(kit::muted(label, 8.));
                                        });
                                }
                            });
                        });
                    });
            }
        });
}

fn plate(
    parent: &mut ChildSpawnerCommands,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    content: impl FnOnce(&mut ChildSpawnerCommands),
) {
    parent
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: percent(x),
                top: percent(y),
                width: percent(w),
                height: percent(h),
                border: UiRect::all(px(1)),
                border_radius: BorderRadius::all(px(3)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                overflow: Overflow::clip(),
                ..default()
            },
            frame(),
        ))
        .with_children(content);
}
