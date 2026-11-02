//! Fill the Practice pane with a centered arena picture while preserving its source proportions.

use bevy::prelude::*;

#[derive(Component)]
pub(in crate::training) struct ArenaPicture;

pub(super) fn spawn(p: &mut ChildSpawnerCommands, assets: &AssetServer, arena: tbg_core::Arena) {
    p.spawn((
        Node {
            width: percent(100),
            flex_basis: px(0),
            flex_grow: 1.,
            min_height: px(160),
            flex_shrink: 0.,
            overflow: Overflow::clip(),
            border_radius: BorderRadius::all(px(5)),
            ..default()
        },
        ImageNode {
            image_mode: NodeImageMode::Stretch,
            ..ImageNode::new(assets.load(format!("arenas/{}.webp", arena.slug())))
        },
        ArenaPicture,
    ));
}

// Use the same post-layout cover geometry as the shared Ranking podium: the source
// rectangle has the destination's aspect ratio, so stretching that rectangle cannot distort it.
pub(in crate::training) fn cover(
    images: Res<Assets<Image>>,
    mut pictures: Query<(&ComputedNode, &mut ImageNode), With<ArenaPicture>>,
) {
    for (node, mut image) in &mut pictures {
        let Some(source) = images.get(&image.image) else {
            continue;
        };
        if node.size.min_element() <= 0. {
            continue;
        }
        let size = source.size().as_vec2();
        let ratio = node.size.x / node.size.y;
        let crop = if size.x / size.y > ratio {
            Vec2::new(size.y * ratio, size.y)
        } else {
            Vec2::new(size.x, size.x / ratio)
        };
        let rect = Some(Rect::from_center_size(size * 0.5, crop));
        if image.rect != rect {
            image.rect = rect;
        }
    }
}
