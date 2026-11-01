//! Keep PBR lighting on GLES drivers without cube-array shadow LOD sampling.

use bevy::{prelude::*, shader::Source};

pub fn adapt(mut shaders: ResMut<Assets<Shader>>, mut done: Local<bool>) {
    if *done {
        return;
    }
    let Some(id) = shaders
        .iter()
        .find(|(_, shader)| shader.path.ends_with("/render/shadow_sampling.wesl"))
        .map(|(id, _)| id)
    else {
        return;
    };
    *done = true;
    let Some(shader) = shaders.get_mut(id) else {
        return;
    };
    let Source::Wesl(source) = &shader.source else {
        error!("Android GLES shadow adapter requires the pinned Bevy 0.20 WESL module");
        return;
    };
    let Some(adapted) = without_point_shadow_lookup(source) else {
        error!("Android GLES shadow adapter did not match the pinned shader function");
        return;
    };
    let mut replacement = Shader::from_wesl(adapted, shader.path.clone());
    replacement.shader_defs = shader.shader_defs.clone();
    replacement.file_dependencies = shader.file_dependencies.clone();
    replacement.validate_shader = shader.validate_shader.clone();
    *shader = replacement;
    info!("Android GLES: unsupported point-shadow LOD lookup disabled; PBR lighting retained");
}

fn without_point_shadow_lookup(source: &str) -> Option<String> {
    let function = source.find("fn sample_shadow_cubemap_hardware(")?;
    let body = function + source[function..].find('{')?;
    let end = body + source[body..].find("\n}")?;
    let mut adapted = source.to_owned();
    // ponytail: this GLES profile has no point-light shadows. Keep directional
    // shadows/PBR and restore the lookup when the driver/engine supports its LOD.
    // Low shadows already disables shadow-casting lights; this removes an
    // unreachable shader operation that still requires an absent GL extension.
    adapted.replace_range(body + 1..end, "\n    return 1.0;");
    Some(adapted)
}
