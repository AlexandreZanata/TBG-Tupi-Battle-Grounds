//! Reads the compact evaluated Blender geometry shared by every game client.

mod catalog;
pub(super) mod packs;
pub(super) mod scenery;

pub(super) use catalog::{arena, auxiliary, classic, stronghold};

use bevy::{
    asset::RenderAssetUsages, mesh::Indices, prelude::*, render::render_resource::PrimitiveTopology,
};

/// Keep a capture-only reference to the procedural renderers for matched comparisons.
pub(super) fn reference() -> bool {
    cfg!(any(debug_assertions, feature = "ui-acceptance"))
        && crate::config::url_param("capture_legacy_art").is_some()
}

pub(super) struct Model {
    pub origin: Vec3,
    pub batches: Vec<Batch>,
}

pub(super) struct Batch {
    pub name: String,
    pub socket: Vec3,
    pub track: Vec<(u16, Transform)>,
    pub mesh: Mesh,
}

struct Reader<'a>(&'a [u8]);

impl<'a> Reader<'a> {
    fn take(&mut self, count: usize) -> Option<&'a [u8]> {
        let (data, rest) = self.0.split_at_checked(count)?;
        self.0 = rest;
        Some(data)
    }

    fn bytes<const N: usize>(&mut self) -> Option<[u8; N]> {
        self.take(N)?.try_into().ok()
    }

    fn floats<const N: usize>(&mut self) -> Option<[f32; N]> {
        let mut values = [0.0; N];
        for value in &mut values {
            *value = f32::from_le_bytes(self.bytes()?);
            if !value.is_finite() {
                return None;
            }
        }
        Some(values)
    }

    fn pose(&mut self) -> Option<(u16, Transform)> {
        let tick = u16::from_le_bytes(self.bytes()?);
        let translation = Vec3::from_array(self.floats()?);
        let rotation = Quat::from_array(self.floats()?);
        let scale = Vec3::from_array(self.floats()?);
        Some((
            tick,
            Transform {
                translation,
                rotation,
                scale,
            },
        ))
    }

    fn batch(&mut self, units: f32) -> Option<Batch> {
        let [len] = self.bytes()?;
        let name = std::str::from_utf8(self.take(usize::from(len))?)
            .ok()?
            .to_owned();
        let frames = usize::from(u16::from_le_bytes(self.bytes()?));
        let vertices = u32::from_le_bytes(self.bytes()?) as usize;
        let indices = u32::from_le_bytes(self.bytes()?) as usize;
        let socket = Vec3::from_array(self.floats()?);
        let track: Vec<_> = (0..frames).map(|_| self.pose()).collect::<Option<_>>()?;
        if track.first()?.0 != 0 || !track.is_sorted_by(|a, b| a.0 < b.0) {
            return None;
        }
        let mesh = self.mesh(units, vertices, indices)?;
        Some(Batch {
            name,
            socket,
            track,
            mesh,
        })
    }

    fn mesh(&mut self, units: f32, count: usize, index_count: usize) -> Option<Mesh> {
        let packed = self.take(count.checked_mul(12)?)?;
        let mut positions = Vec::with_capacity(count);
        let mut normals = Vec::with_capacity(count);
        let mut colors = Vec::with_capacity(count);
        for vertex in packed.chunks_exact(12) {
            let [x, y, z, nx, ny, nz, r, g, b] = unpack(vertex);
            positions.push([x / units, y / units, z / units]);
            normals.push(Vec3::new(nx, ny, nz).normalize_or_zero().to_array());
            colors.push(
                Color::srgb_u8(r as u8, g as u8, b as u8)
                    .to_linear()
                    .to_f32_array(),
            );
        }
        let indices = if count <= u16::MAX as usize {
            let bytes = self.take(index_count.checked_mul(2)?)?;
            let values = bytes
                .chunks_exact(2)
                .map(|n| u16::from_le_bytes([n[0], n[1]]));
            Indices::U16(values.collect())
        } else {
            let bytes = self.take(index_count.checked_mul(4)?)?;
            let values = bytes
                .chunks_exact(4)
                .map(|n| u32::from_le_bytes([n[0], n[1], n[2], n[3]]));
            Indices::U32(values.collect())
        };
        if index_count % 3 != 0 || indices.iter().any(|index| index >= count) {
            return None;
        }
        let mut mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::RENDER_WORLD,
        );
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
        mesh.insert_indices(indices);
        Some(mesh)
    }
}

fn unpack(v: &[u8]) -> [f32; 9] {
    [
        f32::from(i16::from_le_bytes([v[0], v[1]])),
        f32::from(i16::from_le_bytes([v[2], v[3]])),
        f32::from(i16::from_le_bytes([v[4], v[5]])),
        f32::from(v[6] as i8),
        f32::from(v[7] as i8),
        f32::from(v[8] as i8),
        f32::from(v[9]),
        f32::from(v[10]),
        f32::from(v[11]),
    ]
}

impl Model {
    pub fn read(data: &[u8]) -> Option<Self> {
        // The largest evaluated terrain stays below 2 MiB. Bound corrupt packs
        // before allocating vertex buffers on a resource-constrained client.
        let decoded =
            miniz_oxide::inflate::decompress_to_vec_zlib_with_limit(data, 4 * 1024 * 1024).ok()?;
        let mut reader = Reader(&decoded);
        if &reader.bytes::<4>()? != b"TBGD" {
            return None;
        }
        let count = usize::from(u16::from_le_bytes(reader.bytes()?));
        let units = f32::from(u16::from_le_bytes(reader.bytes()?));
        if count == 0 || units == 0.0 {
            return None;
        }
        let origin = Vec3::from_array(reader.floats()?);
        let batches = (0..count)
            .map(|_| reader.batch(units))
            .collect::<Option<_>>()?;
        reader.0.is_empty().then_some(Self { origin, batches })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_blender_packs_decode_and_have_live_motion_or_expected_batches() {
        let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/arena/blender");
        let mut count = 0;
        for entry in std::fs::read_dir(folder).unwrap() {
            let path = entry.unwrap().path();
            if path.extension().and_then(|ext| ext.to_str()) != Some("tbgd") {
                continue;
            }
            let name = path.file_stem().unwrap().to_str().unwrap();
            let data = std::fs::read(&path).unwrap();
            let pack = Model::read(&data).unwrap_or_else(|| panic!("invalid pack: {name}"));
            assert!(
                pack.batches
                    .iter()
                    .all(|part| part.mesh.count_vertices() > 0),
                "{name}"
            );
            if name.contains("-miner-") || name.contains("-merchant-") {
                assert!(
                    pack.batches.iter().any(|part| part.track.len() > 20),
                    "{name}"
                );
                assert!(pack.origin.x < 2.0 || pack.origin.x > 14.0, "{name}");
            } else if name.starts_with("bastion-b2-") {
                let ballista = pack
                    .batches
                    .iter()
                    .find(|part| part.name == "gun2")
                    .unwrap();
                assert!(ballista.socket.x > 0.4 && ballista.socket.y > 0.2, "{name}");
                assert!(
                    pack.batches.iter().any(|part| part.name == "gun2bolt"),
                    "{name}"
                );
            } else if !name.contains('-') {
                assert!(
                    pack.batches.iter().any(|part| part.name == "water"),
                    "{name}"
                );
                assert!(
                    pack.batches.iter().any(|part| part.name == "terrain"),
                    "{name}"
                );
            }
            count += 1;
        }
        assert_eq!(count, 96);
    }
}
