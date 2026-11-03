//! Bounded presentation-only emoji assets and animation shared by all desktop targets.
use bevy::prelude::*;
use std::collections::{BTreeMap, BTreeSet};
use tbg_core::protocol::EmojiRef;
pub mod atlas;
pub mod sheet;
pub mod standard;
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Key {
    Public([u8; 16]),
    Private(u64, i64),
}
#[derive(Resource, Default)]
struct Sheets {
    pending: BTreeMap<Key, crate::auth::api::Request>,
    loaded: BTreeMap<Key, Handle<Image>>,
    failed: BTreeSet<Key>,
    epoch: u64,
}
pub fn plugin(app: &mut App) {
    app.init_resource::<Sheets>()
        .add_systems(Startup, standard::preload)
        // UI roots are rebuilt with deferred Commands during Update. Bind their
        // images after that flush so even a slow frame renders the right texture.
        .add_systems(PostUpdate, update.before(bevy::ui::UiSystems::Layout));
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
            private_asset: None,
        },
    ));
}
/// Private normalized sheets share the decoder/animation code, with account-scoped cache keys.
pub fn preview(parent: &mut ChildSpawnerCommands, asset: i64, reference: EmojiRef, born: f64) {
    parent.spawn((
        ImageNode::default(),
        Node {
            width: px(96),
            height: px(96),
            ..default()
        },
        atlas::Sprite {
            reference,
            fallback: 0,
            born,
            private_asset: Some(asset),
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
    let epoch = crate::auth::epoch();
    if sheets.epoch != epoch {
        sheets.pending.clear();
        sheets.failed.clear();
        for (_, handle) in std::mem::take(&mut sheets.loaded) {
            images.remove(handle.id());
        }
        sheets.epoch = epoch;
    }
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
                let key = sprite
                    .private_asset
                    .map(|asset| Key::Private(epoch, asset))
                    .unwrap_or(Key::Public(*id));
                if let Some(handle) = sheets.loaded.get(&key) {
                    node.image = handle.clone();
                    node.rect = atlas::rect(&sprite.reference, now - sprite.born);
                } else {
                    node.image = standard::handle(&assets, sprite.fallback);
                    node.rect = None;
                    if sheets.pending.len() < 6
                        && sheets.failed.len() < 128
                        && !sheets.pending.contains_key(&key)
                        && !sheets.failed.contains(&key)
                    {
                        let hex: String = id.iter().map(|b| format!("{b:02x}")).collect();
                        sheets.pending.insert(
                            key,
                            crate::auth::api::Request::sheet(
                                sprite
                                    .private_asset
                                    .map(|asset| format!("/api/me/emoji/{asset}/sheet.png"))
                                    .unwrap_or_else(|| format!("/api/emoji/{hex}.png")),
                                sprite.private_asset.is_some(),
                            ),
                        );
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::asset::AssetApp;

    #[test]
    fn p75_rebuilt_hud_has_a_standard_image_before_its_first_render_frame() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, bevy::asset::AssetPlugin::default()));
        app.init_asset::<Image>();
        plugin(&mut app);
        app.add_systems(
            Update,
            (|mut commands: Commands| {
                commands.spawn_empty().with_children(|parent| {
                    spawn(parent, EmojiRef::Standard { id: 0 }, 0, 0., 24.);
                });
            })
            .after(update),
        );
        app.update();
        let mut query = app.world_mut().query::<&ImageNode>();
        let image = query.single(app.world()).expect("one synthetic HUD sprite");
        assert_ne!(
            image.image,
            Handle::<Image>::default(),
            "BR-EM01/IC-11: a rebuilt HUD must not draw the unbound default texture"
        );
    }
}
