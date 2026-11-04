//! Reads the compact joint-local meshes exported by the Blender champion pipeline.

use super::{
    figure::{self, FigureSpec},
    rig::{self, Node, Socket, NODE_COUNT},
    FigureForm, FORMS,
};
use bevy::{
    asset::RenderAssetUsages, mesh::Indices, prelude::*, render::render_resource::PrimitiveTopology,
};
use tbg_core::UnitKind;

const ARCHER: &[u8] = include_bytes!("../../assets/champions/blender/archer.tbgmesh");
const BONE_BRUTE: &[u8] = include_bytes!("../../assets/champions/blender/bonebrute.tbgmesh");
const SHIELDBEARER: &[u8] = include_bytes!("../../assets/champions/blender/shieldbearer.tbgmesh");
const HEXER: &[u8] = include_bytes!("../../assets/champions/blender/hexer.tbgmesh");

pub(super) fn form(
    kind: UnitKind,
    tier: u8,
    branch: u8,
    spec: &FigureSpec,
    assets: &mut Assets<Mesh>,
) -> Option<FigureForm> {
    let data = match kind {
        UnitKind::Archer => ARCHER,
        UnitKind::BoneBrute => BONE_BRUTE,
        UnitKind::Shieldbearer => SHIELDBEARER,
        UnitKind::Hexer => HEXER,
        _ => return None,
    };
    let row = FORMS.iter().position(|form| *form == (tier, branch))?;
    let Some((mut lit, glow)) = decode(data, row, &spec.plan) else {
        error!("Blender mesh or rig data is invalid for {kind:?} {tier}/{branch}");
        return None;
    };

    if kind == UnitKind::Archer {
        // The game animates these three pieces independently from the bow limbs.
        let bow = FigureSpec {
            plan: spec.plan,
            parts: spec
                .parts
                .iter()
                .copied()
                .filter(|part| {
                    matches!(
                        part.node,
                        Node::Prop(Socket::StringTop | Socket::StringBottom | Socket::Nock)
                    )
                })
                .collect(),
        };
        for (node, mesh) in lit.iter_mut().zip(figure::build_joint_meshes(&bow)) {
            if mesh.is_some() {
                *node = mesh;
            }
        }
    }
    Some(super::register_form(spec.plan, lit, glow, assets))
}

type Layers = ([Option<Mesh>; NODE_COUNT], [Option<Mesh>; NODE_COUNT]);

fn decode(data: &[u8], row: usize, plan: &rig::Proportions) -> Option<Layers> {
    let mut payload = Reader(data.strip_prefix(b"TBG4")?);
    let mut sizes = Reader(payload.take(FORMS.len() * 4)?);
    let start = (0..row).try_fold(0usize, |offset, _| {
        offset.checked_add(sizes.u32()? as usize)
    })?;
    payload.take(start)?;
    let mut input = Reader(payload.take(sizes.u32()? as usize)?);
    check_rig(&mut input, plan)?;
    let [groups] = input.bytes()?;
    let mut lit: [Option<Mesh>; NODE_COUNT] = Default::default();
    let mut glow: [Option<Mesh>; NODE_COUNT] = Default::default();
    for _ in 0..groups {
        let [layer, node] = input.bytes()?;
        let target = match layer {
            0 => lit.get_mut(usize::from(node))?,
            1 => glow.get_mut(usize::from(node))?,
            _ => return None,
        };
        if target.is_some() {
            return None;
        }
        *target = Some(decode_mesh(&mut input, layer == 1)?);
    }
    input.0.is_empty().then_some((lit, glow))
}

fn check_rig(input: &mut Reader<'_>, plan: &rig::Proportions) -> Option<()> {
    let mut rest = [Vec3::ZERO; NODE_COUNT];
    // Blender owns the 23 shared nodes. The animated string and nock are runtime nodes.
    for node in rig::nodes().take(Node::Prop(Socket::StringTop).index()) {
        let world = node.parent().map_or(Vec3::ZERO, |p| rest[p.index()]) + rig::rest(node, plan);
        if (world - Vec3::from_array(input.vector(4096.0)?)).length() > 0.000_25 {
            return None;
        }
        rest[node.index()] = world;
    }
    Some(())
}

fn decode_mesh(input: &mut Reader<'_>, glow: bool) -> Option<Mesh> {
    let count = usize::from(input.u16()?);
    let index_count = input.u32()? as usize;
    if count == 0 || index_count == 0 || !index_count.is_multiple_of(3) {
        return None;
    }
    // Consume the bounded slices before allocating vectors.
    let mut vertices = Reader(input.take(count * 15)?);
    let mut index_data = Reader(input.take(index_count.checked_mul(2)?)?);
    let mut positions = Vec::with_capacity(count);
    let mut normals = Vec::with_capacity(count);
    let mut colors = Vec::with_capacity(count);
    for _ in 0..count {
        positions.push(vertices.vector(4096.0)?);
        let normal = Vec3::from_array(vertices.vector(32767.0)?).normalize_or_zero();
        normals.push(normal.to_array());
        let [r, g, b] = vertices.bytes()?;
        if matches!([r, g, b], [63, 169, 245] | [242, 84, 91]) {
            return None;
        }
        let color = Color::srgb_u8(r, g, b).to_linear();
        let shade = if glow { 1.0 } else { 0.9 + 0.15 * normal.y };
        colors.push([
            color.red * shade,
            color.green * shade,
            color.blue * shade,
            1.0,
        ]);
    }
    let indices = (0..index_count)
        .map(|_| index_data.u16().filter(|index| usize::from(*index) < count))
        .collect::<Option<Vec<_>>>()?;
    Some(
        Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        )
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
        .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, colors)
        .with_inserted_indices(Indices::U16(indices)),
    )
}

struct Reader<'a>(&'a [u8]);

impl<'a> Reader<'a> {
    fn take(&mut self, size: usize) -> Option<&'a [u8]> {
        let (bytes, rest) = self.0.split_at_checked(size)?;
        self.0 = rest;
        Some(bytes)
    }

    fn bytes<const N: usize>(&mut self) -> Option<[u8; N]> {
        self.take(N)?.try_into().ok()
    }

    fn u16(&mut self) -> Option<u16> {
        Some(u16::from_le_bytes(self.bytes()?))
    }

    fn u32(&mut self) -> Option<u32> {
        Some(u32::from_le_bytes(self.bytes()?))
    }

    fn vector(&mut self, scale: f32) -> Option<[f32; 3]> {
        let mut result = [0.0; 3];
        for component in &mut result {
            *component = f32::from(i16::from_le_bytes(self.bytes()?)) / scale;
        }
        Some(result)
    }
}

#[cfg(test)]
#[test]
fn blender_assets_cover_all_forms_and_keep_the_animated_archer_string() {
    let mut assets = Assets::default();
    for kind in [
        UnitKind::Archer,
        UnitKind::BoneBrute,
        UnitKind::Shieldbearer,
        UnitKind::Hexer,
    ] {
        let kit = super::kits::kit(kind).unwrap();
        for (tier, branch) in FORMS {
            let spec = (kit.figure)(tier, branch);
            let form = form(kind, tier, branch, &spec, &mut assets).unwrap();
            assert!(form.meshes[rig::Joint::Head.index()].is_some());
            assert!(std::ptr::eq(form.meshes(true), form.meshes(false)));
            let grip = if kind == UnitKind::Archer {
                Socket::OffhandL
            } else {
                Socket::WeaponR
            };
            assert!(form.meshes[Node::Prop(grip).index()].is_some());
            if kind == UnitKind::Archer {
                for socket in [
                    Socket::OffhandL,
                    Socket::StringTop,
                    Socket::StringBottom,
                    Socket::Nock,
                ] {
                    assert!(form.meshes[Node::Prop(socket).index()].is_some());
                }
            }
        }
    }
}
