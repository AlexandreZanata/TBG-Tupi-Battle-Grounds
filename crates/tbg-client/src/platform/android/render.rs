//! Conservative Vulkan device capabilities avoid optional Mali descriptor-array failures.

use bevy::render::{
    settings::{Backends, WgpuSettings, WgpuSettingsPriority},
    RenderPlugin,
};
pub fn plugin() -> RenderPlugin {
    RenderPlugin {
        // Functionality enables every advertised feature; the POCO's optional
        // descriptor-array path fails allocation. Baseline PBR retains all game content.
        render_creation: WgpuSettings {
            backends: Some(Backends::VULKAN),
            priority: WgpuSettingsPriority::WebGPU,
            ..Default::default()
        }
        .into(),
        ..Default::default()
    }
}
