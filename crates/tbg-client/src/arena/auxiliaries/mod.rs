//! Off-lane 3D War Auxiliaries (`Mineiro` and `Mercador`): precinct spawning,
//! per-Arena palettes and placement, and the client-side state that drives
//! their construction, tier and payout animations (P74, BR-AX12).
//!
//! Coordinate contract (`ARENA-CLEAR-01`):
//! - Left (`Seat::A` / Blue) precincts sit at `x = -2.55` (`z = -3.45` Mine, `z = +3.45` Merchant).
//! - Right (`Seat::B` / Red) precincts sit at `x = +18.55`, mirrored along X.
//! - Every static and animated vertex stays strictly outside `x ∈ [-1.1, 17.1], z ∈ [-2.67, 2.67]`.
//! - A pad over the water stands on stilts and is reached by a plank bridge from the shore.

mod animate;
mod models;
mod palette;
mod placement;
mod structures;

use crate::arena::{scene::ActiveArena, theme::Theme, water};
use crate::sync::LatestView;
use crate::GameState;
use bevy::prelude::*;
use palette::AuxPalette;
use placement::{pad_center, pad_half, placement};
use tbg_core::auxiliary::{AuxiliaryKind, AuxiliaryPhase, AUXILIARY_RULE};
use tbg_core::Seat;

/// True when a static arena prop at `pos` would clip into a `Mineiro` or
/// `Mercador` pad or bridge (BR-AX15). Leaves `camp::BRAZIERS` untouched.
pub(super) fn overlaps_precinct(theme: Theme, pos: Vec3) -> bool {
    Seat::BOTH.into_iter().any(|seat| {
        AuxiliaryKind::ALL
            .into_iter()
            .any(|kind| placement::covers(theme, seat, kind, pos.xz(), 0.35))
    })
}

/// True when an arena prop must yield to a precinct (BR-AX15): any prop whose
/// origin lies on a pad or bridge, and sea-surface floaters crossing a
/// harbor's open water. Rocks, crags and landmarks are never removed; the
/// layouts keep clear of them instead.
pub(super) fn yields_to_precinct(theme: Theme, part: &crate::arena::Part) -> bool {
    let (shape, _, t) = part;
    if overlaps_precinct(theme, t.translation) {
        return true;
    }
    if t.translation.y >= placement::FLOATER_Y {
        return false;
    }
    let mesh = crate::art::mesh::build(*shape);
    let Some(bevy::mesh::VertexAttributeValues::Float32x3(points)) =
        mesh.attribute(Mesh::ATTRIBUTE_POSITION)
    else {
        return false;
    };
    // Matches the lab: half the widest horizontal extent of the placed prop.
    let (min, max) = points
        .iter()
        .map(|p| t.transform_point(Vec3::from(*p)).xz())
        .fold((Vec2::INFINITY, Vec2::NEG_INFINITY), |(lo, hi), p| {
            (lo.min(p), hi.max(p))
        });
    placement::clears_floater(theme, t.translation, (max - min).max_element() * 0.5)
}

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<AuxiliarySceneState>()
        .add_systems(Startup, spawn_precincts)
        .add_systems(
            Update,
            (
                rebuild_on_theme_change,
                sync_from_view,
                animate::clock,
                animate::visibility,
                animate::mine,
                animate::merchant,
                animate::construction,
                animate::tiers,
            )
                .chain(),
        );
}

#[derive(Resource, Default)]
struct AuxiliarySceneState {
    built_theme: Option<Theme>,
    material: Handle<StandardMaterial>,
    slot_anim: [[SlotAnimState; 2]; 2],
    animation_time: f32,
}

#[derive(Clone, Copy, Debug, Default)]
struct SlotAnimState {
    phase: AuxiliaryPhase,
    /// Authoritative tier (`0` = not hired).
    tier: u8,
    hired_seen: bool,
    /// Seconds since this auxiliary was hired on the client clock (drives Merchant construction).
    since_hire: f32,
    /// Seconds since the last tier Evolution (drives tier reveals and the golden ring).
    tier_since: f32,
    /// Last seen `activations_used` for the Merchant, triggering the ship delivery window.
    last_activations: u8,
    /// Remaining seconds of the Merchant ship docking & crate unloading sequence.
    delivery_timer: f32,
}

#[derive(Component, Clone, Copy)]
pub(super) struct AuxPrecinctRoot {
    pub seat: Seat,
    pub kind: AuxiliaryKind,
}

#[derive(Component, Clone, Copy)]
struct AuxPartOwner(Seat);

/// A structure that appears only from `tier` onward, popping in when bought.
#[derive(Component, Clone, Copy)]
struct TierPart {
    slot: usize,
    tier: u8,
}

/// Construction pieces that bring an offshore pad within reach of the shore.
#[derive(Component, Clone, Copy)]
enum Approach {
    Platform {
        slot: usize,
    },
    Plank {
        slot: usize,
        index: usize,
        count: usize,
    },
    Rails {
        slot: usize,
    },
}

/// Golden ring that sweeps out of the pad on every tier Evolution.
#[derive(Component, Clone, Copy)]
struct TierRing(usize);

#[derive(Component, Clone, Copy, PartialEq, Eq)]
enum MinePart {
    Outcrop,
    Shoring,
    Rails,
    Cart,
    Ore,
    Lantern,
    Sparks,
    MinerBody,
    MinerPickaxe,
    WinchDrum,
    Bucket,
    Sheave,
    Banner,
    Crystals,
    Apprentice,
}

#[derive(Component, Clone, Copy)]
enum MerchantPart {
    PierPilings,
    PierDeck,
    StallFrame,
    StallAwning,
    StallCargo,
    MerchantBody,
    SpyglassArm,
    Ship,
    DeliveryCrate,
    Pennant,
    Pavilion,
    ChestLid,
    Coins,
}

/// Precinct-local spots of the tier structures, shared with the collision lab.
/// Each keeps clear of the base precinct and of the other tiers.
mod spots {
    use bevy::math::Vec3;
    pub const WINCH: Vec3 = Vec3::new(0.62, 0.0, -0.56);
    pub const ORE_PILE: Vec3 = Vec3::new(0.85, 0.0, 0.55);
    pub const HEADFRAME: Vec3 = Vec3::new(-0.42, 0.42, 0.0);
    pub const BANNER: Vec3 = Vec3::new(1.0, 0.0, -0.7);
    pub const APPRENTICE: Vec3 = Vec3::new(0.42, 0.0, 0.6);
    pub const GOODS: Vec3 = Vec3::new(0.66, 0.0, -0.1);
    pub const PAVILION: Vec3 = Vec3::new(-0.97, 0.0, -0.1);
    pub const CHEST: Vec3 = Vec3::new(0.66, 0.0, 0.3);
}

/// Material shared by every precinct mesh. Seat B is mirrored with a negative
/// X scale, which flips triangle winding; rendering both faces keeps the
/// outward faces (and their correctly transformed normals) visible instead of
/// the dark interiors that made the red side look grey.
fn precinct_material() -> StandardMaterial {
    StandardMaterial {
        base_color: Color::WHITE,
        perceptual_roughness: 0.95,
        cull_mode: None,
        ..default()
    }
}

fn spawn_precincts(
    mut commands: Commands,
    mut state: ResMut<AuxiliarySceneState>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut mats: ResMut<Assets<StandardMaterial>>,
) {
    state.material = mats.add(precinct_material());
    let theme = Theme::selected();
    state.built_theme = Some(theme);
    for seat in Seat::BOTH {
        for kind in AuxiliaryKind::ALL {
            let root = commands
                .spawn((
                    Name::new(format!("auxiliary {seat:?} {kind:?}")),
                    AuxPrecinctRoot { seat, kind },
                    placement(theme, seat, kind).root,
                    Visibility::Hidden,
                ))
                .id();
            populate_precinct(
                &mut commands,
                &mut meshes,
                &state.material,
                root,
                seat,
                kind,
                theme,
            );
        }
    }
}

/// Spawns one mesh child and returns it, so moving parts can nest further pieces.
struct Builder<'a, 'w, 's> {
    commands: &'a mut Commands<'w, 's>,
    meshes: &'a mut Assets<Mesh>,
    material: Handle<StandardMaterial>,
    seat: Seat,
}

impl Builder<'_, '_, '_> {
    fn part(&mut self, parent: Entity, mesh: Mesh, at: Transform, tag: impl Bundle) -> Entity {
        self.commands
            .spawn((
                Mesh3d(self.meshes.add(mesh)),
                MeshMaterial3d(self.material.clone()),
                at,
                tag,
                AuxPartOwner(self.seat),
                ChildOf(parent),
            ))
            .id()
    }

    /// Mesh-less pivot: its transform carries a tier reveal while children animate.
    fn group(&mut self, parent: Entity, at: Transform, tag: impl Bundle) -> Entity {
        self.commands
            .spawn((
                at,
                Visibility::Inherited,
                tag,
                AuxPartOwner(self.seat),
                ChildOf(parent),
            ))
            .id()
    }
}

fn populate_precinct(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    material: &Handle<StandardMaterial>,
    root: Entity,
    seat: Seat,
    kind: AuxiliaryKind,
    theme: Theme,
) {
    commands.entity(root).despawn_related::<Children>();
    let pal = AuxPalette::for_theme(theme);
    let mut b = Builder {
        commands,
        meshes,
        material: material.clone(),
        seat,
    };
    spawn_approach(&mut b, root, kind, pal, theme);
    let slot = usize::from(kind.slot());
    b.part(
        root,
        structures::tier_ring(pal),
        Transform::from_scale(Vec3::ZERO),
        TierRing(slot),
    );
    match kind {
        AuxiliaryKind::Miner => spawn_mine(&mut b, root, pal, seat == Seat::A),
        AuxiliaryKind::Merchant => spawn_merchant(&mut b, root, pal, seat == Seat::A),
    }
}

fn spawn_approach(
    b: &mut Builder,
    root: Entity,
    kind: AuxiliaryKind,
    pal: AuxPalette,
    theme: Theme,
) {
    let place = placement(theme, b.seat, kind);
    let slot = usize::from(kind.slot());
    if !place.on_water {
        return;
    }
    let platform = structures::platform(pal, pad_center(kind), pad_half(kind));
    b.part(
        root,
        platform,
        Transform::default(),
        Approach::Platform { slot },
    );
    let Some((land, edge)) = place.bridge else {
        return;
    };
    let span = edge - land;
    let length = span.length();
    let yaw = (-span.z).atan2(span.x);
    let axis = b
        .commands
        .spawn((
            Transform::from_translation(land).with_rotation(Quat::from_rotation_y(yaw)),
            Visibility::Inherited,
            ChildOf(root),
        ))
        .id();
    let count = (length / structures::PLANK_STEP).ceil().max(1.0) as usize;
    for index in 0..count {
        let at = Transform::from_xyz((index as f32 + 0.5) * length / count as f32, 0.0, 0.0);
        let tag = Approach::Plank { slot, index, count };
        b.part(axis, structures::bridge_plank(pal, index), at, tag);
    }
    let rails = structures::bridge_rails(pal, length);
    b.part(axis, rails, Transform::default(), Approach::Rails { slot });
}

fn spawn_mine(b: &mut Builder, root: Entity, pal: AuxPalette, blue: bool) {
    let cart_at = Transform::from_xyz(0.62, 0.03, 0.0);
    for (part, mesh, at) in [
        (
            MinePart::Outcrop,
            models::mine_outcrop(pal),
            Transform::default(),
        ),
        (
            MinePart::Shoring,
            models::mine_shoring(pal),
            Transform::default(),
        ),
        (
            MinePart::Rails,
            models::mine_rails(pal),
            Transform::default(),
        ),
        (MinePart::Cart, models::mine_cart(pal), cart_at),
        (MinePart::Ore, models::mine_ore(pal), cart_at),
        (
            MinePart::Lantern,
            models::mine_lantern(pal),
            Transform::from_xyz(0.24, 0.68, 0.0),
        ),
        (
            MinePart::Sparks,
            models::mine_sparks(pal),
            Transform::from_xyz(0.22, 0.28, 0.0).with_scale(Vec3::ZERO),
        ),
    ] {
        b.part(root, mesh, at, part);
    }
    let facing = Quat::from_rotation_y(std::f32::consts::PI);
    let body = b.part(
        root,
        models::miner_body(blue),
        Transform::from_xyz(0.46, 0.0, 0.22).with_rotation(facing),
        MinePart::MinerBody,
    );
    b.part(
        body,
        models::miner_pickaxe(),
        Transform::from_xyz(0.0, 0.44, 0.16),
        MinePart::MinerPickaxe,
    );

    // Tier 2: a crank winch hauling ore buckets and a glowing heap by the rails.
    let tier = |tier| TierPart { slot: 0, tier };
    let winch = b.part(
        root,
        structures::winch_frame(pal),
        Transform::from_translation(spots::WINCH),
        tier(2),
    );
    b.part(
        winch,
        structures::winch_drum(pal),
        Transform::from_xyz(0.0, 0.62, 0.0),
        MinePart::WinchDrum,
    );
    b.part(
        winch,
        structures::ore_bucket(pal),
        Transform::from_xyz(0.0, 0.3, 0.0),
        MinePart::Bucket,
    );
    b.part(
        root,
        structures::ore_pile(pal),
        Transform::from_translation(spots::ORE_PILE),
        tier(2),
    );

    // Tier 3: a headframe on the crag, a crystal crown, a banner and an apprentice.
    let head = b.part(
        root,
        structures::headframe(pal),
        Transform::from_translation(spots::HEADFRAME),
        tier(3),
    );
    b.part(
        head,
        structures::sheave(pal),
        Transform::from_xyz(0.0, 1.5, 0.0),
        MinePart::Sheave,
    );
    let crown = b.group(root, Transform::from_xyz(-0.12, 0.6, -0.44), tier(3));
    b.part(
        crown,
        structures::crystal_crown(pal),
        Transform::default(),
        MinePart::Crystals,
    );
    let flag = b.group(root, Transform::from_translation(spots::BANNER), tier(3));
    b.part(
        flag,
        structures::banner(pal, blue),
        Transform::default(),
        MinePart::Banner,
    );
    let helper = b.group(root, Transform::default(), tier(3));
    b.part(
        helper,
        models::miner_body(blue),
        Transform::from_translation(spots::APPRENTICE).with_scale(Vec3::splat(0.82)),
        MinePart::Apprentice,
    );
}

fn spawn_merchant(b: &mut Builder, root: Entity, pal: AuxPalette, blue: bool) {
    for (part, mesh, at) in [
        (
            MerchantPart::PierPilings,
            models::merchant_pier_pilings(pal),
            Transform::default(),
        ),
        (
            MerchantPart::PierDeck,
            models::merchant_pier_deck(pal),
            Transform::default(),
        ),
        (
            MerchantPart::StallFrame,
            models::merchant_stall_frame(pal),
            Transform::default(),
        ),
        (
            MerchantPart::StallAwning,
            models::merchant_stall_awning(pal),
            Transform::default(),
        ),
        (
            MerchantPart::StallCargo,
            models::merchant_stall_cargo(pal),
            Transform::default(),
        ),
        (
            MerchantPart::Ship,
            models::merchant_ship(pal, blue),
            Transform::from_xyz(-3.4, water::LEVEL, 2.25),
        ),
        (
            MerchantPart::DeliveryCrate,
            models::merchant_crate_parcel(pal),
            Transform::from_xyz(0.0, -0.05, 1.35).with_scale(Vec3::ZERO),
        ),
    ] {
        b.part(root, mesh, at, part);
    }
    let body = b.part(
        root,
        models::merchant_body(blue),
        Transform::from_xyz(0.0, 0.0, 0.52)
            .with_rotation(Quat::from_rotation_y(-std::f32::consts::FRAC_PI_2)),
        MerchantPart::MerchantBody,
    );
    b.part(
        body,
        models::merchant_spyglass_arm(blue),
        Transform::from_xyz(0.0, 0.44, 0.16),
        MerchantPart::SpyglassArm,
    );

    // Tier 2: stacked goods and team pennants on the pier bollards.
    let tier = |tier| TierPart { slot: 1, tier };
    b.part(
        root,
        structures::goods_stack(pal),
        Transform::from_translation(spots::GOODS),
        tier(2),
    );
    for dx in [-0.28, 0.28] {
        let post = b.group(root, Transform::from_xyz(dx, 0.02, 1.48), tier(2));
        b.part(
            post,
            structures::pennant(pal, blue),
            Transform::default(),
            MerchantPart::Pennant,
        );
    }

    // Tier 3: a market pavilion and a treasure chest that bursts with coins on payouts.
    let tent = b.group(root, Transform::from_translation(spots::PAVILION), tier(3));
    b.part(
        tent,
        structures::pavilion(pal, blue),
        Transform::default(),
        MerchantPart::Pavilion,
    );
    let chest = b.part(
        root,
        structures::chest(pal),
        Transform::from_translation(spots::CHEST),
        tier(3),
    );
    b.part(
        chest,
        structures::chest_lid(pal),
        Transform::from_xyz(0.0, 0.12, -0.085),
        MerchantPart::ChestLid,
    );
    b.part(
        chest,
        structures::coins(pal),
        Transform::from_xyz(0.0, 0.12, 0.0).with_scale(Vec3::ZERO),
        MerchantPart::Coins,
    );
}

/// Gallery portrait (`--gallery=auxiliary`): the in-game figure alone, in a
/// working pose, for the HUD card renders.
pub(crate) fn spawn_portrait(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    material: &Handle<StandardMaterial>,
    kind: AuxiliaryKind,
    blue: bool,
) {
    let (body, tool, tool_at, angle) = match kind {
        AuxiliaryKind::Miner => (
            models::miner_body(blue),
            models::miner_pickaxe(),
            Vec3::new(0.0, 0.44, 0.16),
            0.75,
        ),
        AuxiliaryKind::Merchant => (
            models::merchant_body(blue),
            models::merchant_spyglass_arm(blue),
            Vec3::new(0.0, 0.44, 0.16),
            0.38,
        ),
    };
    let figure = commands
        .spawn((
            Mesh3d(meshes.add(body)),
            MeshMaterial3d(material.clone()),
            Transform::default(),
        ))
        .id();
    commands.spawn((
        Mesh3d(meshes.add(tool)),
        MeshMaterial3d(material.clone()),
        Transform::from_translation(tool_at).with_rotation(Quat::from_rotation_z(angle)),
        ChildOf(figure),
    ));
}

fn rebuild_on_theme_change(
    mut commands: Commands,
    active_arena: Option<Res<ActiveArena>>,
    mut state: ResMut<AuxiliarySceneState>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut roots: Query<(Entity, &AuxPrecinctRoot, &mut Transform)>,
) {
    let want = active_arena.map_or_else(Theme::selected, |a| a.0);
    if state.built_theme == Some(want) {
        return;
    }
    state.built_theme = Some(want);
    for (entity, root, mut transform) in &mut roots {
        *transform = placement(want, root.seat, root.kind).root;
        populate_precinct(
            &mut commands,
            &mut meshes,
            &state.material,
            entity,
            root.seat,
            root.kind,
            want,
        );
    }
}

/// `--auxiliaries[=tier]` review preview: every precinct active at that tier.
fn preview(state: &mut AuxiliarySceneState, tier: u8) {
    for seat_slots in &mut state.slot_anim {
        seat_slots[0].phase = AuxiliaryPhase::Active {
            ticks_left: 2_400,
            progress: 0,
            activations_used: 0,
        };
        seat_slots[1].phase = AuxiliaryPhase::Active {
            ticks_left: 0,
            progress: 1,
            activations_used: 1,
        };
        for slot in seat_slots {
            slot.tier = tier;
            slot.hired_seen = true;
            slot.since_hire = slot.since_hire.max(15.0);
            slot.tier_since = slot.tier_since.max(15.0);
        }
    }
}

fn sync_from_view(
    view: Res<LatestView>,
    game_state: Option<Res<State<GameState>>>,
    mut state: ResMut<AuxiliarySceneState>,
) {
    let in_match = game_state.is_none_or(|s| *s.get() == GameState::InMatch);
    let preview_tier = crate::config::url_param("auxiliaries").map(|v| {
        v.parse::<u8>()
            .unwrap_or(1)
            .clamp(1, tbg_core::auxiliary::AUXILIARY_MAX_TIER)
    });
    if !in_match && preview_tier.is_none() {
        state.slot_anim = Default::default();
        return;
    }
    let Some(v) = &view.0 else {
        if let Some(tier) = preview_tier {
            preview(&mut state, tier);
        }
        return;
    };
    for seat in Seat::BOTH {
        for (slot_idx, entry) in v.auxiliaries[seat.index()].iter().enumerate() {
            observe(
                &mut state.slot_anim[seat.index()][slot_idx],
                slot_idx,
                entry.phase,
                entry.tier,
            );
        }
    }
}

fn observe(anim: &mut SlotAnimState, slot_idx: usize, phase: AuxiliaryPhase, tier: u8) {
    if phase == AuxiliaryPhase::Available {
        *anim = SlotAnimState::default();
        return;
    }
    if !anim.hired_seen {
        *anim = SlotAnimState {
            hired_seen: true,
            tier,
            // Joining mid-match (reconnect, spectate) shows structures already built.
            tier_since: if tier > 1 { 15.0 } else { 0.0 },
            ..SlotAnimState::default()
        };
    }
    if tier > anim.tier {
        anim.tier = tier;
        anim.tier_since = 0.0;
    }
    if slot_idx == 1 {
        let used = match phase {
            AuxiliaryPhase::Active {
                activations_used, ..
            } => activations_used,
            AuxiliaryPhase::Completed => {
                let t = usize::from(anim.tier.clamp(1, 3) - 1);
                AUXILIARY_RULE.merchant_max_activations[t]
            }
            _ => 0,
        };
        if used > anim.last_activations {
            anim.last_activations = used;
            anim.delivery_timer = animate::DELIVERY_SECS;
        }
    }
    anim.phase = phase;
}

#[cfg(test)]
mod tests;
