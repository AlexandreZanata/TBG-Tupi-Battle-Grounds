// Bevy 0.20 MATERIAL_BIND_GROUP_INDEX = 3; extension bindings start at 100.
import bevy_pbr::render::{
    forward_io::{VertexOutput, FragmentOutput},
    pbr_fragment::pbr_input_from_standard_material,
    pbr_functions::{alpha_discard, apply_pbr_lighting, main_pass_post_lighting_processing},
    pbr_types::STANDARD_MATERIAL_FLAGS_UNLIT_BIT,
};

@group(3) @binding(100) var fog_mask: texture_2d<f32>;
@group(3) @binding(101) var fog_sampler: sampler;

@fragment
fn fragment(in: VertexOutput, @builtin(front_facing) is_front: bool) -> FragmentOutput {
    var pbr_input = pbr_input_from_standard_material(in, is_front);
    pbr_input.material.base_color = alpha_discard(pbr_input.material.flags,
        pbr_input.material.alpha_cutoff, pbr_input.material.base_color);
    var out: FragmentOutput;
    if (pbr_input.material.flags & STANDARD_MATERIAL_FLAGS_UNLIT_BIT) == 0u {
        out.color = apply_pbr_lighting(pbr_input);
    } else {
        out.color = pbr_input.material.base_color;
    }
    let uv = vec2<f32>((in.world_position.x + 100.0) / 200.0,
        (100.0 - in.world_position.z) / 200.0);
    let visibility = textureSample(fog_mask, fog_sampler, clamp(uv, vec2<f32>(0.0), vec2<f32>(1.0))).r;
    let dim = out.color.rgb * vec3<f32>(0.12, 0.16, 0.25);
    out.color = vec4<f32>(mix(dim, out.color.rgb, visibility), out.color.a);
    out.color = main_pass_post_lighting_processing(pbr_input, out.color);
    return out;
}
