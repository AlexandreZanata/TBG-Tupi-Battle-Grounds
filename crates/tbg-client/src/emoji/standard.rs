//! Permanent standard emoji ids, sourced from the attributed Twemoji 14.0.2 set.
use bevy::prelude::*;
pub fn handle(assets: &AssetServer, id: u8) -> Handle<Image> {
    assets.load(format!("emoji/standard/{}.png", id.min(15)))
}
