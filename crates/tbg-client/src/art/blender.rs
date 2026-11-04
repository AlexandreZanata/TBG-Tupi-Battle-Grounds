//! Reads the compact joint-local meshes exported by the Blender champion pipeline.

use super::{
    FORMS, FigureForm,
    figure::{self, FigureSpec},
    rig::{self, NODE_COUNT, Node, Socket},
};
use bevy::{
    asset::RenderAssetUsages, mesh::Indices, prelude::*, render::render_resource::PrimitiveTopology,
};
use tbg_core::UnitKind;

const ARCHER: &[u8] = include_bytes!("../../assets/champions/blender/archer.tbgmesh");
const BONE_BRUTE: &[u8] = include_bytes!("../../assets/champions/blender/bonebrute.tbgmesh");
const SHIELDBEARER: &[u8] = include_bytes!("../../assets/champions/blender/shieldbearer.tbgmesh");
const HEXER: &[u8] = include_bytes!("../../assets/champions/blender/hexer.tbgmesh");
const SAWBOT: &[u8] = include_bytes!("../../assets/champions/blender/sawbot.tbgmesh");
const BLOOMKEEPER: &[u8] = include_bytes!("../../assets/champions/blender/bloomkeeper.tbgmesh");
const SIEGE_WALKER: &[u8] = include_bytes!("../../assets/champions/blender/siegewalker.tbgmesh");
const ROOTCALLER: &[u8] = include_bytes!("../../assets/champions/blender/rootcaller.tbgmesh");
const NESTER: &[u8] = include_bytes!("../../assets/champions/blender/nester.tbgmesh");
const RAMPART: &[u8] = include_bytes!("../../assets/champions/blender/rampart.tbgmesh");
const LONGAXE: &[u8] = include_bytes!("../../assets/champions/blender/longaxe.tbgmesh");
const FARWATCH: &[u8] = include_bytes!("../../assets/champions/blender/farwatch.tbgmesh");
const BOMBARD: &[u8] = include_bytes!("../../assets/champions/blender/bombard.tbgmesh");
const ROTCAP: &[u8] = include_bytes!("../../assets/champions/blender/rotcap.tbgmesh");
const OLD_FINN: &[u8] = include_bytes!("../../assets/champions/blender/oldfinn.tbgmesh");
const ANTIDOTE_BREWER: &[u8] =
    include_bytes!("../../assets/champions/blender/antidotebrewer.tbgmesh");
const BLOOD_FURNACE: &[u8] = include_bytes!("../../assets/champions/blender/bloodfurnace.tbgmesh");
const BLOODWEAVER: &[u8] = include_bytes!("../../assets/champions/blender/bloodweaver.tbgmesh");
const OILPRESS: &[u8] = include_bytes!("../../assets/champions/blender/oilpress.tbgmesh");
const DRUMGUARD: &[u8] = include_bytes!("../../assets/champions/blender/drumguard.tbgmesh");
const LEECHROOT: &[u8] = include_bytes!("../../assets/champions/blender/leechroot.tbgmesh");
const SHATTERBACK: &[u8] = include_bytes!("../../assets/champions/blender/shatterback.tbgmesh");
const CARGO_CRAB: &[u8] = include_bytes!("../../assets/champions/blender/cargocrab.tbgmesh");
const PUPPETMASTER: &[u8] = include_bytes!("../../assets/champions/blender/puppetmaster.tbgmesh");
const SMUGGLER: &[u8] = include_bytes!("../../assets/champions/blender/smuggler.tbgmesh");
const BONEHORN: &[u8] = include_bytes!("../../assets/champions/blender/bonehorn.tbgmesh");
const RAVAGER: &[u8] = include_bytes!("../../assets/champions/blender/ravager.tbgmesh");
const ACIDMAW: &[u8] = include_bytes!("../../assets/champions/blender/acidmaw.tbgmesh");
const WHISPERER: &[u8] = include_bytes!("../../assets/champions/blender/whisperer.tbgmesh");
const BULWARK: &[u8] = include_bytes!("../../assets/champions/blender/bulwark.tbgmesh");
const DEMOLISHER: &[u8] = include_bytes!("../../assets/champions/blender/demolisher.tbgmesh");
const CROWCALLER: &[u8] = include_bytes!("../../assets/champions/blender/crowcaller.tbgmesh");
const TRIGGER: &[u8] = include_bytes!("../../assets/champions/blender/trigger.tbgmesh");
const RATTLE_TICK: &[u8] = include_bytes!("../../assets/champions/blender/rattletick.tbgmesh");
const CAPYBARA: &[u8] = include_bytes!("../../assets/champions/blender/capybara.tbgmesh");
const TRIDENT: &[u8] = include_bytes!("../../assets/champions/blender/trident.tbgmesh");
const SOLAR_SENTINEL: &[u8] =
    include_bytes!("../../assets/champions/blender/solarsentinel.tbgmesh");
const LUMINOUS_ORACLE: &[u8] =
    include_bytes!("../../assets/champions/blender/luminousoracle.tbgmesh");
const CELESTIAL_ARCHER: &[u8] =
    include_bytes!("../../assets/champions/blender/celestialarcher.tbgmesh");
const JUSTICIAR: &[u8] = include_bytes!("../../assets/champions/blender/justiciar.tbgmesh");
const SHATTER_PLATES: [&[u8]; 4] = [
    include_bytes!("../../assets/champions/blender/shatterback-plate-0.tbgmesh"),
    include_bytes!("../../assets/champions/blender/shatterback-plate-1.tbgmesh"),
    include_bytes!("../../assets/champions/blender/shatterback-plate-2.tbgmesh"),
    include_bytes!("../../assets/champions/blender/shatterback-plate-3.tbgmesh"),
];

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
        UnitKind::Sawbot => SAWBOT,
        UnitKind::Bloomkeeper => BLOOMKEEPER,
        UnitKind::SiegeWalker => SIEGE_WALKER,
        UnitKind::Rootcaller => ROOTCALLER,
        UnitKind::Nester => NESTER,
        UnitKind::Rampart => RAMPART,
        UnitKind::Longaxe => LONGAXE,
        UnitKind::Farwatch => FARWATCH,
        UnitKind::Bombard => BOMBARD,
        UnitKind::Rotcap => ROTCAP,
        UnitKind::OldFinn => OLD_FINN,
        UnitKind::AntidoteBrewer => ANTIDOTE_BREWER,
        UnitKind::BloodFurnace => BLOOD_FURNACE,
        UnitKind::Bloodweaver => BLOODWEAVER,
        UnitKind::Oilpress => OILPRESS,
        UnitKind::Drumguard => DRUMGUARD,
        UnitKind::Leechroot => LEECHROOT,
        UnitKind::Shatterback => SHATTERBACK,
        UnitKind::CargoCrab => CARGO_CRAB,
        UnitKind::Puppetmaster => PUPPETMASTER,
        UnitKind::Smuggler => SMUGGLER,
        UnitKind::Bonehorn => BONEHORN,
        UnitKind::Ravager => RAVAGER,
        UnitKind::Acidmaw => ACIDMAW,
        UnitKind::Whisperer => WHISPERER,
        UnitKind::Bulwark => BULWARK,
        UnitKind::Demolisher => DEMOLISHER,
        UnitKind::Crowcaller => CROWCALLER,
        UnitKind::Trigger => TRIGGER,
        UnitKind::RattleTick => RATTLE_TICK,
        UnitKind::Capybara => CAPYBARA,
        UnitKind::Trident => TRIDENT,
        UnitKind::SolarSentinel => SOLAR_SENTINEL,
        UnitKind::LuminousOracle => LUMINOUS_ORACLE,
        UnitKind::CelestialArcher => CELESTIAL_ARCHER,
        UnitKind::Justiciar => JUSTICIAR,
        _ => return None,
    };
    let row = FORMS.iter().position(|form| *form == (tier, branch))?;
    let Some((mut lit, glow)) = decode(data, row, &spec.plan, kind) else {
        error!("Blender mesh or rig data is invalid for {kind:?} {tier}/{branch}");
        return None;
    };

    // The existing kit animates these sockets independently of the exported weapon.
    if matches!(kind, UnitKind::Archer | UnitKind::SiegeWalker) {
        let animated = FigureSpec {
            plan: spec.plan,
            parts: spec
                .parts
                .iter()
                .copied()
                .filter(|part| {
                    matches!(
                        (kind, part.node),
                        (
                            UnitKind::Archer,
                            Node::Prop(Socket::StringTop | Socket::StringBottom | Socket::Nock)
                        ) | (UnitKind::SiegeWalker, Node::Prop(Socket::Nock))
                    )
                })
                .collect(),
        };
        for (node, mesh) in lit.iter_mut().zip(figure::build_joint_meshes(&animated)) {
            if mesh.is_some() {
                *node = mesh;
            }
        }
    }
    Some(super::register_form(spec.plan, lit, glow, assets))
}

type Layers = ([Option<Mesh>; NODE_COUNT], [Option<Mesh>; NODE_COUNT]);

pub(super) fn shatter_plate(
    tier: u8,
    branch: u8,
    index: usize,
    plan: &rig::Proportions,
) -> Option<Mesh> {
    let row = FORMS.iter().position(|form| *form == (tier, branch))?;
    let data = SHATTER_PLATES.get(index)?;
    let mesh = decode(data, row, plan, UnitKind::Shatterback)
        .and_then(|(mut lit, _)| lit[rig::Joint::Chest.index()].take());
    if mesh.is_none() {
        error!("Blender Shatterback plate {index} data is invalid for {tier}/{branch}");
    }
    mesh
}

fn decode(data: &[u8], row: usize, plan: &rig::Proportions, kind: UnitKind) -> Option<Layers> {
    let (magic, data) = data.split_at_checked(4)?;
    let node_count = match magic {
        b"TBG4" => Node::Prop(Socket::StringTop).index(),
        b"TBG5" => NODE_COUNT,
        _ => return None,
    };
    let mut payload = Reader(data);
    let mut sizes = Reader(payload.take(FORMS.len() * 4)?);
    let start = (0..row).try_fold(0usize, |offset, _| {
        offset.checked_add(sizes.u32()? as usize)
    })?;
    payload.take(start)?;
    let mut input = Reader(payload.take(sizes.u32()? as usize)?);
    check_rig(&mut input, plan, kind, node_count)?;
    let [groups] = input.bytes()?;
    let mut lit: [Option<Mesh>; NODE_COUNT] = Default::default();
    let mut glow: [Option<Mesh>; NODE_COUNT] = Default::default();
    for _ in 0..groups {
        let [layer, node] = input.bytes()?;
        if usize::from(node) >= node_count {
            return None;
        }
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

fn check_rig(
    input: &mut Reader<'_>,
    plan: &rig::Proportions,
    kind: UnitKind,
    node_count: usize,
) -> Option<()> {
    let rest = super::presentation::pose_world(kind, plan, &Default::default());
    for world in rest.into_iter().take(node_count) {
        if (world.w_axis.truncate() - Vec3::from_array(input.vector(4096.0)?)).length() > 0.000_25 {
            return None;
        }
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
fn blender_assets_cover_all_forms_and_keep_animated_weapon_sockets() {
    let mut assets = Assets::default();
    for kind in [
        UnitKind::Archer,
        UnitKind::BoneBrute,
        UnitKind::Shieldbearer,
        UnitKind::Hexer,
        UnitKind::Sawbot,
        UnitKind::Bloomkeeper,
        UnitKind::SiegeWalker,
        UnitKind::Rootcaller,
        UnitKind::Nester,
        UnitKind::Rampart,
        UnitKind::Longaxe,
        UnitKind::Farwatch,
        UnitKind::Bombard,
        UnitKind::Rotcap,
        UnitKind::OldFinn,
        UnitKind::AntidoteBrewer,
        UnitKind::BloodFurnace,
        UnitKind::Shatterback,
        UnitKind::Bloodweaver,
        UnitKind::Oilpress,
        UnitKind::Drumguard,
        UnitKind::Leechroot,
        UnitKind::CargoCrab,
        UnitKind::Puppetmaster,
        UnitKind::Smuggler,
        UnitKind::Bonehorn,
        UnitKind::Ravager,
        UnitKind::Acidmaw,
        UnitKind::Whisperer,
        UnitKind::Bulwark,
        UnitKind::Demolisher,
        UnitKind::Crowcaller,
        UnitKind::Trigger,
        UnitKind::RattleTick,
        UnitKind::Capybara,
        UnitKind::Trident,
        UnitKind::SolarSentinel,
        UnitKind::LuminousOracle,
        UnitKind::CelestialArcher,
        UnitKind::Justiciar,
    ] {
        let kit = super::kits::kit(kind).unwrap();
        for (tier, branch) in FORMS {
            let spec = (kit.figure)(tier, branch);
            let form = form(kind, tier, branch, &spec, &mut assets).unwrap();
            let body = if kind == UnitKind::CargoCrab {
                rig::Joint::Hips
            } else {
                rig::Joint::Head
            };
            assert!(form.meshes[body.index()].is_some());
            assert!(std::ptr::eq(form.meshes(true), form.meshes(false)));
            let grip = match kind {
                UnitKind::Archer | UnitKind::CelestialArcher => Node::Prop(Socket::OffhandL),
                UnitKind::Rotcap | UnitKind::Bonehorn | UnitKind::Acidmaw => {
                    Node::Body(rig::Joint::Head)
                }
                UnitKind::Ravager => Node::Body(rig::Joint::ForearmR),
                UnitKind::Whisperer => Node::Prop(Socket::Back),
                UnitKind::Sawbot
                | UnitKind::Rampart
                | UnitKind::Shatterback
                | UnitKind::Oilpress
                | UnitKind::Leechroot
                | UnitKind::CargoCrab
                | UnitKind::Demolisher
                | UnitKind::Capybara
                | UnitKind::Justiciar => Node::Body(rig::Joint::HandR),
                _ => Node::Prop(Socket::WeaponR),
            };
            assert!(form.meshes[grip.index()].is_some());
            if kind == UnitKind::Drumguard
                || (matches!(
                    kind,
                    UnitKind::Bloodweaver | UnitKind::AntidoteBrewer | UnitKind::Smuggler
                ) && (tier, branch) == (2, 1))
                || (kind == UnitKind::Puppetmaster && tier > 0 && branch == 1)
            {
                assert!(form.meshes[Node::Prop(Socket::OffhandL).index()].is_some());
            }
            if kind == UnitKind::LuminousOracle {
                let ring = Node::Secondary(rig::Spring::HeadGear).index();
                assert!(form.meshes[ring].is_some());
                assert!(form.glow[ring].is_some());
            }
            if kind == UnitKind::Shatterback {
                for index in 0..4 {
                    assert!(shatter_plate(tier, branch, index, &spec.plan).is_some());
                }
            }
            if kind == UnitKind::SiegeWalker {
                assert!(form.meshes[Node::Prop(Socket::Nock).index()].is_some());
            }
            let additional: &[_] = match kind {
                UnitKind::Archer | UnitKind::CelestialArcher => &[
                    Socket::OffhandL,
                    Socket::StringTop,
                    Socket::StringBottom,
                    Socket::Nock,
                ],
                UnitKind::SolarSentinel => {
                    &[Socket::OffhandL, Socket::StringTop, Socket::StringBottom]
                }
                UnitKind::Crowcaller => &[Socket::Back, Socket::Nock],
                UnitKind::Trigger => &[Socket::OffhandL, Socket::Back],
                UnitKind::RattleTick => &[Socket::Back, Socket::StringTop, Socket::Nock],
                UnitKind::Ravager => &[
                    Socket::WeaponR,
                    Socket::OffhandL,
                    Socket::Back,
                    Socket::StringTop,
                    Socket::StringBottom,
                    Socket::Nock,
                ],
                UnitKind::Acidmaw => &[
                    Socket::WeaponR,
                    Socket::OffhandL,
                    Socket::Back,
                    Socket::StringTop,
                ],
                _ => &[],
            };
            for &socket in additional {
                assert!(form.meshes[Node::Prop(socket).index()].is_some());
            }
        }
    }
}
