//! Native GLES compatibility profile; the POCO Vulkan allocator is blocked by driver errors.

use bevy::render::{
    settings::{Backends, WgpuFeatures, WgpuLimits, WgpuSettings, WgpuSettingsPriority},
    RenderPlugin,
};
use std::ffi::{c_char, c_int, CStr};

pub fn plugin() -> RenderPlugin {
    let render_creation = if emulator() {
        // Emulator EGL exposes no sRGB window surface, which Bevy 0.20 requires on GLES
        // (SURFACE_VIEW_FORMATS); its Vulkan device uses the engine's normal limits.
        WgpuSettings {
            backends: Some(Backends::VULKAN),
            ..Default::default()
        }
    } else {
        // Keep the same PBR/art path with the engine's uniform-buffer fallback.
        // Vulkan failed twice on the owned POCO; do not silently retry that backend.
        WgpuSettings {
            backends: Some(Backends::GL),
            priority: WgpuSettingsPriority::WebGL2,
            features: WgpuFeatures::empty(),
            // Overriding priority does not recompute limits in Default::default().
            // Zero vertex storage selects the engine's actual uniform-buffer layouts.
            // The POCO landscape surface is wider than WebGL2's 2048px minimum.
            limits: WgpuLimits::downlevel_webgl2_defaults().using_resolution(WgpuLimits::default()),
            ..Default::default()
        }
    };
    RenderPlugin {
        render_creation: render_creation.into(),
        // ponytail: P69 diagnostic — compile pipelines synchronously so failures surface at once.
        synchronous_pipeline_compilation: true,
        ..Default::default()
    }
}

extern "C" {
    fn __system_property_get(name: *const c_char, value: *mut c_char) -> c_int;
}

/// `ro.boot.qemu` is set only by the Android emulator's boot configuration.
fn emulator() -> bool {
    let mut value = [0 as c_char; 92];
    // SAFETY: Bionic writes at most PROP_VALUE_MAX (92) bytes, NUL terminated.
    let length = unsafe { __system_property_get(c"ro.boot.qemu".as_ptr(), value.as_mut_ptr()) };
    // SAFETY: the buffer is NUL terminated by the call above or still all zero.
    let emulator = length > 0 && unsafe { CStr::from_ptr(value.as_ptr()) }.to_bytes() == b"1";
    bevy::log::info!(
        "Android render backend: {}",
        if emulator {
            "Vulkan (emulator)"
        } else {
            "GLES"
        }
    );
    emulator
}
