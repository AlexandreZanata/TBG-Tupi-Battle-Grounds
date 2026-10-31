//! One tiny generated repeat tile adds water caustics or molten cell edges without downloads.

use super::super::theme::Theme;
use bevy::{
    asset::RenderAssetUsages,
    image::{ImageAddressMode, ImageSampler, ImageSamplerDescriptor},
    mesh::VertexAttributeValues,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};

pub(super) fn surface(mut mesh: Mesh, theme: Theme) -> Mesh {
    if let Some(VertexAttributeValues::Float32x3(points)) = mesh.attribute(Mesh::ATTRIBUTE_POSITION)
    {
        let scale = if theme == Theme::Volcanic {
            0.085
        } else {
            0.23
        };
        let uv: Vec<[f32; 2]> = points
            .iter()
            .map(|p| [p[0] * scale, p[2] * scale])
            .collect();
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uv);
    }
    if theme == Theme::Volcanic {
        if let Some(VertexAttributeValues::Float32x4(colors)) =
            mesh.attribute_mut(Mesh::ATTRIBUTE_COLOR)
        {
            for c in colors {
                c[0] *= 2.6;
                c[1] *= 5.5;
                c[2] *= 2.0;
            }
        }
    }
    mesh
}

pub(in crate::arena) fn create(theme: Theme, images: &mut Assets<Image>) -> StandardMaterial {
    let mut image = Image::new(
        Extent3d {
            width: 256,
            height: 256,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        pixels(theme),
        TextureFormat::Rgba8Unorm,
        RenderAssetUsages::RENDER_WORLD,
    );
    image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
        address_mode_u: ImageAddressMode::Repeat,
        address_mode_v: ImageAddressMode::Repeat,
        ..ImageSamplerDescriptor::linear()
    });
    StandardMaterial {
        base_color: Color::WHITE,
        base_color_texture: Some(images.add(image)),
        unlit: true,
        ..default()
    }
}

fn random(x: i32, y: i32, salt: u32) -> f32 {
    let n = (x.rem_euclid(8) as u32).wrapping_mul(0x9e37_79b9)
        ^ (y.rem_euclid(8) as u32).wrapping_mul(0x85eb_ca6b)
        ^ salt;
    let n = (n ^ (n >> 15)).wrapping_mul(0x7feb_352d);
    ((n ^ (n >> 16)) & 65535) as f32 / 65535.0
}

fn edge(x: f32, y: f32) -> f32 {
    let (cx, cy) = (x.floor() as i32, y.floor() as i32);
    let (mut first, mut second) = (f32::MAX, f32::MAX);
    for dy in -1..=1 {
        for dx in -1..=1 {
            let (u, v) = (cx + dx, cy + dy);
            let px = u as f32 + 0.18 + random(u, v, 71) * 0.64;
            let py = v as f32 + 0.18 + random(u, v, 197) * 0.64;
            let d = (x - px).powi(2) + (y - py).powi(2);
            if d < first {
                second = first;
                first = d;
            } else if d < second {
                second = d;
            }
        }
    }
    (1.0 - ((second.sqrt() - first.sqrt()) / 0.14).clamp(0.0, 1.0)).powi(2)
}

fn field(x: f32, y: f32) -> f32 {
    let (ix, iy) = (x.floor() as i32, y.floor() as i32);
    let (u, v) = (x - x.floor(), y - y.floor());
    let (u, v) = (u * u * (3.0 - 2.0 * u), v * v * (3.0 - 2.0 * v));
    let a = random(ix, iy, 811);
    let b = random(ix + 1, iy, 811);
    let c = random(ix, iy + 1, 811);
    let d = random(ix + 1, iy + 1, 811);
    (a + (b - a) * u) * (1.0 - v) + (c + (d - c) * u) * v
}

fn pixels(theme: Theme) -> Vec<u8> {
    let mut pixels = Vec::with_capacity(256 * 256 * 4);
    for y in 0..256 {
        for x in 0..256 {
            let phase_x = x as f32 * std::f32::consts::TAU / 256.0;
            let phase_y = y as f32 * std::f32::consts::TAU / 256.0;
            let u = x as f32 / 32.0 + (phase_y * 3.0).sin() * 0.45 + (phase_x * 5.0).cos() * 0.12;
            let v = y as f32 / 32.0 + (phase_x * 2.0).cos() * 0.50 + (phase_y * 7.0).sin() * 0.15;
            let wave = edge(u, v);
            let broken = 0.55 + ((phase_x * 3.0 + phase_y * 5.0).sin() * 0.5 + 0.5) * 0.45;
            if theme == Theme::Volcanic {
                let turbulence = field(u, v) * 0.62
                    + field(u * 2.0, v * 2.0) * 0.26
                    + field(u * 4.0, v * 4.0) * 0.12;
                let flow = (1.0 - (turbulence - 0.48).abs() / 0.18).clamp(0.0, 1.0);
                let heat = flow * flow * (3.0 - 2.0 * flow);
                pixels.extend([
                    (165.0 + heat * 90.0) as u8,
                    (4.0 + heat.powi(3) * 251.0) as u8,
                    (1.0 + heat.powi(6) * 220.0) as u8,
                    255,
                ]);
            } else {
                let value = 0.91 + wave * broken * 0.09;
                let v = (value * 255.0).round() as u8;
                pixels.extend([v, v, v, 255]);
            }
        }
    }
    pixels
}
