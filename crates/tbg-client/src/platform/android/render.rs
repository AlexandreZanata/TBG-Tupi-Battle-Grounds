//! Native GLES compatibility profile; the POCO Vulkan allocator is blocked by driver errors.

use bevy::render::{
    settings::{Backends, WgpuFeatures, WgpuLimits, WgpuSettings, WgpuSettingsPriority},
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
            // Overriding priority does not recompute limits in Default::default().
            // Zero vertex storage selects the engine's actual uniform-buffer layouts.
            // The POCO landscape surface is wider than WebGL2's 2048px minimum.
            limits: WgpuLimits::downlevel_webgl2_defaults().using_resolution(WgpuLimits::default()),
            ..Default::default()
        }
        .into(),
        ..Default::default()
    }
}
