//! Native GLES compatibility profile; the POCO Vulkan allocator is blocked by driver errors.

use bevy::render::{
    settings::{Backends, WgpuFeatures, WgpuLimits, WgpuSettings, WgpuSettingsPriority},
    RenderPlugin,
};
use std::ffi::{c_char, c_int, CStr};

pub fn plugin() -> RenderPlugin {
    let backend = if emulator() {
        Backends::VULKAN
    } else {
        Backends::GL
    };
    // Select the engine's complete uniform-buffer path, including its device-derived
    // phase allocation. Disabling only the PBR builder leaves incompatible GPU batches.
    let render_creation = WgpuSettings {
        backends: Some(backend),
        priority: WgpuSettingsPriority::WebGL2,
        features: WgpuFeatures::empty(),
        limits: WgpuLimits::downlevel_webgl2_defaults().using_resolution(WgpuLimits::default()),
        ..Default::default()
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
