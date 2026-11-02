//! Native GLES compatibility profile; the POCO Vulkan allocator is blocked by driver errors.

use bevy::render::{
    settings::{Backends, WgpuFeatures, WgpuLimits, WgpuSettings, WgpuSettingsPriority},
    RenderPlugin,
};
use std::ffi::{c_char, c_int, CStr};

pub fn plugin() -> RenderPlugin {
    RenderPlugin {
        // Keep the same PBR/art path with the engine's uniform-buffer fallback.
        // Vulkan failed twice on the owned POCO; do not silently retry that backend.
        render_creation: WgpuSettings {
            backends: Some(backend(emulator())),
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

/// Emulator EGL exposes no sRGB window surface, which Bevy 0.20 requires on
/// GLES (SURFACE_VIEW_FORMATS); only emulators use Vulkan, with the same limits.
fn backend(emulator: bool) -> Backends {
    if emulator {
        Backends::VULKAN
    } else {
        Backends::GL
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
        if emulator { "Vulkan (emulator)" } else { "GLES" }
    );
    emulator
}

#[cfg(test)]
mod tests {
    #[test]
    fn devices_keep_gles_and_only_emulators_use_vulkan() {
        assert_eq!(super::backend(false), super::Backends::GL);
        assert_eq!(super::backend(true), super::Backends::VULKAN);
    }
}
