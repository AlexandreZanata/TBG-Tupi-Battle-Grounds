//! Native GLES compatibility profile; the POCO Vulkan allocator is blocked by driver errors.

use bevy::render::{
    settings::{Backends, WgpuFeatures, WgpuSettings, WgpuSettingsPriority},
    RenderPlugin,
};
pub fn plugin() -> RenderPlugin {
    RenderPlugin {
        // Keep the same PBR/art path with the engine's uniform-buffer fallback.
        // Vulkan failed twice on the owned POCO; do not silently retry that backend.
        render_creation: WgpuSettings {
            backends: Some(Backends::GL),
            priority: WgpuSettingsPriority::WebGL2,
            features: WgpuFeatures::empty(),
            ..Default::default()
        }
        .into(),
        ..Default::default()
    }
}
