//! Bounded presentation-only emoji assets and animation shared by all desktop targets.
use bevy::prelude::*;
use std::collections::{BTreeMap, BTreeSet};
use tbg_core::protocol::EmojiRef;
pub mod atlas;
pub mod sheet;
pub mod standard;
#[derive(Resource, Default)]
struct Sheets {
    pending: BTreeMap<[u8; 16], crate::auth::api::Request>,
    loaded: BTreeMap<[u8; 16], Handle<Image>>,
    failed: BTreeSet<[u8; 16]>,
}
pub fn plugin(app: &mut App) {
    app.init_resource::<Sheets>().add_systems(Update, update);
}
pub fn spawn(
    parent: &mut ChildSpawnerCommands,
    reference: EmojiRef,
    fallback: u8,
    born: f64,
    size: f32,
) {
    parent.spawn((
        ImageNode::default(),
        Node {
            width: px(size),
            height: px(size),
            ..default()
        },
        atlas::Sprite {
            reference,
            fallback,
            born,
        },
    ));
}
fn update(
    mut sheets: ResMut<Sheets>,
    mut images: ResMut<Assets<Image>>,
    assets: Res<AssetServer>,
    time: Res<Time>,
    mut sprites: Query<(&mut ImageNode, &atlas::Sprite)>,
) {
    let now = time.elapsed_secs_f64();
    let pending: Vec<_> = sheets.pending.keys().copied().collect();
    for id in pending {
        let result = sheets.pending.get_mut(&id).and_then(|r| r.take(now));
        let Some(result) = result else {
            continue;
        };
        sheets.pending.remove(&id);
        let bytes = result
            .ok()
            .filter(|r| r.status == 200)
            .and_then(|r| r.bytes)
            .filter(|b| sheet::dimensions(b).is_some());
        let image = bytes.and_then(|b| {
            Image::from_buffer(
                &b,
                bevy::image::ImageType::Extension("png"),
                bevy::image::CompressedImageFormats::NONE,
                true,
                bevy::image::ImageSampler::default(),
                bevy::asset::RenderAssetUsages::default(),
            )
            .ok()
        });
        if let Some(image) = image {
            if sheets.loaded.len() >= 32 {
                if let Some(old) = sheets.loaded.keys().next().copied() {
                    if let Some(handle) = sheets.loaded.remove(&old) {
                        images.remove(handle.id());
                    }
                }
            }
            sheets.loaded.insert(id, images.add(image));
        } else {
            sheets.failed.insert(id);
        }
    }
    for (mut node, sprite) in &mut sprites {
        match &sprite.reference {
            EmojiRef::Standard { id } => {
                node.image = standard::handle(&assets, *id);
                node.rect = None;
            }
            EmojiRef::Custom { id, .. } => {
                if let Some(handle) = sheets.loaded.get(id) {
                    node.image = handle.clone();
                    node.rect = atlas::rect(&sprite.reference, now - sprite.born);
                } else {
                    node.image = standard::handle(&assets, sprite.fallback);
                    node.rect = None;
                    if sheets.pending.len() < 6
                        && sheets.failed.len() < 128
                        && !sheets.pending.contains_key(id)
                        && !sheets.failed.contains(id)
                    {
                        let hex: String = id.iter().map(|b| format!("{b:02x}")).collect();
                        sheets.pending.insert(
                            *id,
                            crate::auth::api::Request::sheet(
                                format!("/api/emoji/{hex}.png"),
                                false,
                            ),
                        );
                    }
                }
            }
        }
    }
}
