//! Off-lane 3D War Auxiliaries (`Mineiro` and `Mercador`), 3-stage procedural construction
//! animations, active mining/shipping loops, and 8-Arena visual reconciliation (P74).
//!
//! Coordinate contract (`ARENA-CLEAR-01`):
//! - Left (`Seat::A` / Blue) precincts sit at `x = -2.55` (`z = -3.45` Mine, `z = +3.45` Merchant).
//! - Right (`Seat::B` / Red) precincts sit at `x = +18.55` (`z = -3.45` Mine, `z = +3.45` Merchant).
//! - Every static and animated vertex stays strictly outside `x ∈ [-1.1, 17.1], z ∈ [-2.67, 2.67]`.

mod models;
mod palette;

use crate::arena::{scene::ActiveArena, theme::Theme, water};
use crate::sync::LatestView;
use crate::GameState;
use bevy::prelude::*;
use palette::AuxPalette;
use tbg_core::auxiliary::{AuxiliaryKind, AuxiliaryPhase, AUXILIARY_RULE};
use tbg_core::Seat;

/// World X centers for Left (`Seat::A` / Blue) and Right (`Seat::B` / Red) precincts.
pub(super) const LEFT_X: f32 = -2.55;
pub(super) const RIGHT_X: f32 = 18.55;
/// World Z center for the North/upper flank `Mineiro` precinct (`z < -2.67`).
pub(super) const MINE_Z: f32 = -3.45;
/// World Z center for the South/lower flank `Mercador` stall (`z > +2.67`).
pub(super) const MERCHANT_Z: f32 = 3.45;

/// True when a static arena prop at `pos` would clip into a `Mineiro` or `Mercador` pad.
/// Leaves `camp::BRAZIERS` (`z = ±1.5..1.7` at `x = -2.1/18.1` and `x = 1.9/14.1`) untouched.
pub(super) fn overlaps_precinct(pos: Vec3) -> bool {
    let near_x = (pos.x - LEFT_X).abs() < 0.88 || (pos.x - RIGHT_X).abs() < 0.88;
    let near_z = (pos.z - MINE_Z).abs() < 0.68 || (pos.z - MERCHANT_Z).abs() < 0.68;
    near_x && near_z
}

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<AuxiliarySceneState>()
        .add_systems(Startup, spawn_precincts)
        .add_systems(
            Update,
            (rebuild_on_theme_change, sync_from_view, animate_precincts).chain(),
        );
}

#[derive(Resource)]
struct AuxiliarySceneState {
    built_theme: Option<Theme>,
    material: Handle<StandardMaterial>,
    slot_anim: [[SlotAnimState; 2]; 2],
    animation_time: f32,
}

impl Default for AuxiliarySceneState {
    fn default() -> Self {
        Self {
            built_theme: None,
            material: Handle::default(),
            slot_anim: Default::default(),
            animation_time: 0.0,
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct SlotAnimState {
    phase: AuxiliaryPhase,
    hired_seen: bool,
    /// Seconds since this auxiliary was hired on the client clock (drives Merchant 2.0s construction).
    since_hire: f32,
    /// Last seen `activations_used` for the Merchant, triggering the 3.2s ship delivery window.
    last_activations: u8,
    /// Remaining seconds of the Merchant ship docking & crate unloading sequence.
    delivery_timer: f32,
    /// Accumulated seconds in `Active` phase.
    active_elapsed: f32,
}

#[derive(Component, Clone, Copy)]
pub(super) struct AuxPrecinctRoot {
    pub seat: Seat,
    pub kind: AuxiliaryKind,
}

#[derive(Component, Clone, Copy)]
struct AuxPartOwner(Seat);

#[derive(Component, Clone, Copy)]
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
}

fn spawn_precincts(
    mut commands: Commands,
    mut state: ResMut<AuxiliarySceneState>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut mats: ResMut<Assets<StandardMaterial>>,
) {
    let mat = mats.add(StandardMaterial {
        base_color: Color::WHITE,
        perceptual_roughness: 0.95,
        ..default()
    });
    state.material = mat;
    let theme = Theme::selected();
    state.built_theme = Some(theme);
    for seat in Seat::BOTH {
        for kind in AuxiliaryKind::ALL {
            let root = commands
                .spawn((
                    Name::new(format!("auxiliary {seat:?} {kind:?}")),
                    AuxPrecinctRoot { seat, kind },
                    precinct_transform(seat, kind),
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

fn precinct_transform(seat: Seat, kind: AuxiliaryKind) -> Transform {
    let x = match seat {
        Seat::A => LEFT_X,
        Seat::B => RIGHT_X,
    };
    let z = match kind {
        AuxiliaryKind::Miner => MINE_Z,
        AuxiliaryKind::Merchant => MERCHANT_Z,
    };
    // Mirror the Right (`Seat::B`) precinct along X so both face inward toward their Stronghold/lane.
    let scale_x = match seat {
        Seat::A => 1.0,
        Seat::B => -1.0,
    };
    Transform::from_xyz(x, 0.0, z).with_scale(Vec3::new(scale_x, 1.0, 1.0))
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
    let blue = seat == Seat::A;

    match kind {
        AuxiliaryKind::Miner => {
            for (part, mesh, tr) in [
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
                (
                    MinePart::Cart,
                    models::mine_cart(pal),
                    Transform::from_xyz(0.62, 0.03, 0.0),
                ),
                (
                    MinePart::Ore,
                    models::mine_ore(pal),
                    Transform::from_xyz(0.62, 0.03, 0.0),
                ),
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
                commands.spawn((
                    Mesh3d(meshes.add(mesh)),
                    MeshMaterial3d(material.clone()),
                    tr,
                    part,
                    AuxPartOwner(seat),
                    ChildOf(root),
                ));
            }
            let body = commands
                .spawn((
                    Mesh3d(meshes.add(models::miner_body(blue))),
                    MeshMaterial3d(material.clone()),
                    Transform::from_xyz(0.46, 0.0, 0.22)
                        .with_rotation(Quat::from_rotation_y(std::f32::consts::PI)),
                    MinePart::MinerBody,
                    AuxPartOwner(seat),
                    ChildOf(root),
                ))
                .id();
            commands.spawn((
                Mesh3d(meshes.add(models::miner_pickaxe())),
                MeshMaterial3d(material.clone()),
                Transform::from_xyz(0.0, 0.44, 0.16),
                MinePart::MinerPickaxe,
                AuxPartOwner(seat),
                ChildOf(body),
            ));
        }
        AuxiliaryKind::Merchant => {
            for (part, mesh, tr) in [
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
                commands.spawn((
                    Mesh3d(meshes.add(mesh)),
                    MeshMaterial3d(material.clone()),
                    tr,
                    part,
                    AuxPartOwner(seat),
                    ChildOf(root),
                ));
            }
            let body = commands
                .spawn((
                    Mesh3d(meshes.add(models::merchant_body(blue))),
                    MeshMaterial3d(material.clone()),
                    Transform::from_xyz(0.0, 0.0, 0.52)
                        .with_rotation(Quat::from_rotation_y(-std::f32::consts::FRAC_PI_2)),
                    MerchantPart::MerchantBody,
                    AuxPartOwner(seat),
                    ChildOf(root),
                ))
                .id();
            commands.spawn((
                Mesh3d(meshes.add(models::merchant_spyglass_arm(blue))),
                MeshMaterial3d(material.clone()),
                Transform::from_xyz(0.0, 0.44, 0.16),
                MerchantPart::SpyglassArm,
                AuxPartOwner(seat),
                ChildOf(body),
            ));
        }
    }
}

fn rebuild_on_theme_change(
    mut commands: Commands,
    active_arena: Option<Res<ActiveArena>>,
    mut state: ResMut<AuxiliarySceneState>,
    mut meshes: ResMut<Assets<Mesh>>,
    roots: Query<(Entity, &AuxPrecinctRoot)>,
) {
    let want = active_arena.map_or_else(Theme::selected, |a| a.0);
    if state.built_theme == Some(want) {
        return;
    }
    state.built_theme = Some(want);
    for (entity, root) in &roots {
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

fn sync_from_view(
    view: Res<LatestView>,
    game_state: Option<Res<State<GameState>>>,
    mut state: ResMut<AuxiliarySceneState>,
) {
    let in_match = game_state.is_none_or(|s| *s.get() == GameState::InMatch);
    let preview = crate::config::url_param("auxiliaries").is_some();
    if !in_match && !preview {
        for seat_slots in &mut state.slot_anim {
            for slot in seat_slots {
                *slot = SlotAnimState::default();
            }
        }
        return;
    }
    if preview && view.0.is_none() {
        for seat_slots in &mut state.slot_anim {
            seat_slots[0].phase = AuxiliaryPhase::Active {
                ticks_left: 2_400,
                progress: 0,
                activations_used: 0,
            };
            seat_slots[0].hired_seen = true;
            seat_slots[0].since_hire = 15.0;
            seat_slots[1].phase = AuxiliaryPhase::Active {
                ticks_left: 0,
                progress: 2,
                activations_used: 1,
            };
            seat_slots[1].hired_seen = true;
            seat_slots[1].since_hire = 10.0;
        }
        return;
    }
    let Some(v) = &view.0 else { return };
    for seat in Seat::BOTH {
        let si = seat.index();
        for slot_idx in 0..2 {
            let new_phase = v.auxiliaries[si][slot_idx].phase;
            let anim = &mut state.slot_anim[si][slot_idx];
            if new_phase == AuxiliaryPhase::Available {
                *anim = SlotAnimState::default();
                continue;
            }
            if !anim.hired_seen {
                anim.hired_seen = true;
                anim.since_hire = 0.0;
                anim.active_elapsed = 0.0;
                anim.last_activations = 0;
                anim.delivery_timer = 0.0;
            }
            if slot_idx == 1 {
                let new_used = match new_phase {
                    AuxiliaryPhase::Active {
                        activations_used, ..
                    } => activations_used,
                    AuxiliaryPhase::Completed => AUXILIARY_RULE.merchant_max_activations,
                    _ => 0,
                };
                if new_used > anim.last_activations {
                    anim.last_activations = new_used;
                    anim.delivery_timer = 3.2;
                }
            }
            anim.phase = new_phase;
        }
    }
}

fn ease_out_cubic(t: f32) -> f32 {
    let inv = 1.0 - t.clamp(0.0, 1.0);
    1.0 - inv * inv * inv
}

fn spring_pop(t: f32) -> f32 {
    let x = t.clamp(0.0, 1.0);
    if x <= 0.0 {
        0.0
    } else if x >= 1.0 {
        1.0
    } else {
        1.0 - (-6.0 * x).exp() * (8.5 * x).cos()
    }
}

fn animate_precincts(
    time: Res<Time>,
    view: Res<LatestView>,
    game_state: Option<Res<State<GameState>>>,
    mut state: ResMut<AuxiliarySceneState>,
    mut roots: Query<(&AuxPrecinctRoot, &mut Visibility)>,
    mut mine_parts: Query<(&MinePart, &AuxPartOwner, &mut Transform), Without<MerchantPart>>,
    mut merch_parts: Query<(&MerchantPart, &AuxPartOwner, &mut Transform), Without<MinePart>>,
) {
    let paused = view.0.as_ref().is_some_and(|v| v.augment_round.is_some());
    let dt = if paused { 0.0 } else { time.delta_secs() };
    state.animation_time += dt;
    let low_motion = crate::config::low_motion();
    let t = if low_motion {
        0.0
    } else {
        state.animation_time
    };
    let hidden_screen = game_state.is_some_and(|s| *s.get() == GameState::Gallery);

    for seat_slots in &mut state.slot_anim {
        for slot in seat_slots {
            if slot.hired_seen {
                slot.since_hire += dt;
                if matches!(slot.phase, AuxiliaryPhase::Active { .. }) {
                    slot.active_elapsed += dt;
                }
                if slot.delivery_timer > 0.0 {
                    slot.delivery_timer = (slot.delivery_timer - dt).max(0.0);
                }
            }
        }
    }

    for (root, mut vis) in &mut roots {
        let anim = state.slot_anim[root.seat.index()][root.kind.slot() as usize];
        let show = !hidden_screen && anim.phase != AuxiliaryPhase::Available;
        *vis = if show {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }

    for (part, owner, mut tr) in &mut mine_parts {
        let anim = state.slot_anim[owner.0.index()][0];
        if anim.phase == AuxiliaryPhase::Available {
            continue;
        }
        // Construction progress p in [0.0, 1.0] across the 12-second (240-tick) Preparing phase.
        let prep_total = AUXILIARY_RULE.miner_prep_ticks.max(1) as f32;
        let p = match anim.phase {
            AuxiliaryPhase::Preparing { ticks_left } => {
                // Construction follows the authoritative countdown, including Augment pauses.
                (1.0 - ticks_left as f32 / prep_total).clamp(0.0, 1.0)
            }
            AuxiliaryPhase::Active { .. } | AuxiliaryPhase::Completed => 1.0,
            AuxiliaryPhase::Available => 0.0,
        };

        let c1 = ((p - 0.0) / 0.35).clamp(0.0, 1.0);
        let c2 = ((p - 0.35) / 0.40).clamp(0.0, 1.0);
        let c3 = ((p - 0.75) / 0.25).clamp(0.0, 1.0);

        match part {
            MinePart::Outcrop => {
                let s = ease_out_cubic(c1).max(0.01);
                tr.translation = Vec3::new(0.0, (1.0 - s) * -0.45, 0.0);
                tr.scale = Vec3::new(1.0, s, 1.0);
            }
            MinePart::Shoring => {
                let s = spring_pop(c2).max(0.001);
                tr.scale = Vec3::splat(s);
            }
            MinePart::Rails => {
                let s = ease_out_cubic(c2).max(0.001);
                tr.scale = Vec3::new(s, 1.0, 1.0);
            }
            MinePart::Cart => {
                let s = ease_out_cubic(c3);
                let base_x = 0.18 + 0.44 * s;
                let rock_dx = match anim.phase {
                    AuxiliaryPhase::Active { .. } => 0.05 * (t * 2.2).sin(),
                    _ => 0.0,
                };
                tr.translation = Vec3::new(base_x + rock_dx, 0.03, 0.0);
                tr.scale = Vec3::splat(s.max(0.001));
            }
            MinePart::Ore => {
                let fill = match anim.phase {
                    AuxiliaryPhase::Preparing { .. } => 0.001,
                    AuxiliaryPhase::Active { ticks_left, .. } => {
                        let dur = AUXILIARY_RULE.miner_duration_ticks.max(1) as f32;
                        let done = 1.0 - (ticks_left as f32 / dur).clamp(0.0, 1.0);
                        0.28 + 0.72 * done
                    }
                    AuxiliaryPhase::Completed => 1.0,
                    AuxiliaryPhase::Available => 0.001,
                };
                let rock_dx = match anim.phase {
                    AuxiliaryPhase::Active { .. } => 0.05 * (t * 2.2).sin(),
                    _ => 0.0,
                };
                tr.translation = Vec3::new(0.62 + rock_dx, 0.03, 0.0);
                tr.scale = Vec3::splat(fill);
            }
            MinePart::Lantern => {
                let s = spring_pop(c3).max(0.001);
                let swing = 0.14 * (t * 3.1).sin();
                tr.translation = Vec3::new(0.24, 0.68, 0.0);
                tr.rotation = Quat::from_rotation_z(swing);
                tr.scale = Vec3::splat(s);
            }
            MinePart::Sparks => {
                let cycle = (t / 1.35).fract();
                let impact = matches!(anim.phase, AuxiliaryPhase::Active { .. })
                    && (0.46..=0.66).contains(&cycle);
                if impact {
                    let burst = 1.0 - ((cycle - 0.56).abs() / 0.10).clamp(0.0, 1.0);
                    tr.translation = Vec3::new(0.22, 0.28 + burst * 0.06, 0.0);
                    tr.scale = Vec3::splat(0.4 + 0.9 * burst);
                    tr.rotation = Quat::from_rotation_y(t * 6.0);
                } else {
                    tr.scale = Vec3::ZERO;
                }
            }
            MinePart::MinerBody => {
                let walk_x = if p < 1.0 {
                    0.92 - 0.46 * ease_out_cubic(c1)
                } else {
                    0.46
                };
                let lean = match anim.phase {
                    AuxiliaryPhase::Preparing { .. } => 0.16 * (t * 6.5).sin(),
                    AuxiliaryPhase::Active { .. } => {
                        let cycle = (t / 1.35).fract();
                        if cycle < 0.46 {
                            -0.18 * (cycle / 0.46)
                        } else if cycle < 0.62 {
                            0.24
                        } else {
                            0.0
                        }
                    }
                    _ => 0.0,
                };
                let bob = if p < 1.0 {
                    0.03 * (t * 9.0).sin().abs()
                } else {
                    0.0
                };
                tr.translation = Vec3::new(walk_x, bob, 0.22);
                tr.rotation =
                    Quat::from_rotation_y(std::f32::consts::PI) * Quat::from_rotation_z(lean);
            }
            MinePart::MinerPickaxe => {
                let angle = match anim.phase {
                    AuxiliaryPhase::Preparing { .. } => 0.25 + 0.85 * (t * 6.5).sin(),
                    AuxiliaryPhase::Active { .. } => {
                        let cycle = (t / 1.35).fract();
                        if cycle < 0.46 {
                            -0.30 + 1.25 * ease_out_cubic(cycle / 0.46)
                        } else if cycle < 0.60 {
                            let strike = (cycle - 0.46) / 0.14;
                            0.95 - 1.75 * ease_out_cubic(strike)
                        } else {
                            let rec = (cycle - 0.60) / 0.40;
                            -0.80 + 0.50 * ease_out_cubic(rec)
                        }
                    }
                    _ => -0.55,
                };
                tr.rotation = Quat::from_rotation_z(angle);
            }
        }
    }

    for (part, owner, mut tr) in &mut merch_parts {
        let anim = state.slot_anim[owner.0.index()][1];
        if anim.phase == AuxiliaryPhase::Available {
            continue;
        }
        // 2.0-second visual construction sequence upon hiring the Merchant.
        let p = (anim.since_hire / 2.0).clamp(0.0, 1.0);
        let m1 = ((p - 0.0) / 0.35).clamp(0.0, 1.0);
        let m2 = ((p - 0.35) / 0.37).clamp(0.0, 1.0);
        let m3 = ((p - 0.72) / 0.28).clamp(0.0, 1.0);

        let progress = match anim.phase {
            AuxiliaryPhase::Active { progress, .. } => progress,
            _ => 0,
        };

        match part {
            MerchantPart::PierPilings => {
                let s = ease_out_cubic(m1).max(0.001);
                tr.translation = Vec3::new(0.0, (1.0 - s) * 0.45, 0.0);
                tr.scale = Vec3::new(1.0, s, 1.0);
            }
            MerchantPart::PierDeck => {
                let s = ease_out_cubic(m1).max(0.001);
                tr.scale = Vec3::new(1.0, 1.0, s);
            }
            MerchantPart::StallFrame => {
                let s = spring_pop(m2).max(0.001);
                tr.scale = Vec3::new(1.0, s, 1.0);
            }
            MerchantPart::StallAwning => {
                let s = spring_pop(m2).max(0.001);
                let breeze = 0.025 * (t * 2.4).sin();
                tr.rotation = Quat::from_rotation_x(breeze);
                tr.scale = Vec3::splat(s);
            }
            MerchantPart::StallCargo => {
                let s = spring_pop(m3).max(0.001);
                tr.scale = Vec3::splat(s);
            }
            MerchantPart::MerchantBody => {
                let walk_z = if p < 1.0 {
                    0.12 + 0.42 * ease_out_cubic(m3)
                } else if progress >= 2 || anim.delivery_timer > 0.0 {
                    0.68
                } else {
                    0.52
                };
                let bob = 0.015 * (t * 2.6).sin().abs();
                tr.translation = Vec3::new(0.0, bob, walk_z);
                tr.rotation = Quat::from_rotation_y(-std::f32::consts::FRAC_PI_2);
            }
            MerchantPart::SpyglassArm => {
                let angle = if anim.delivery_timer > 0.0 {
                    // Celebratory wave while unloading free-reroll cargo
                    0.55 + 0.35 * (t * 7.0).sin()
                } else if progress >= 2 {
                    // Raise brass spyglass to eye scanning the approaching Caravel
                    0.38 + 0.06 * (t * 1.8).sin()
                } else {
                    -0.45 + 0.08 * (t * 1.5).sin()
                };
                tr.rotation = Quat::from_rotation_z(angle);
            }
            MerchantPart::Ship => {
                // Continuous wave buoyancy and pitch/roll on the sea surface
                let dy = 0.045 * (t * 2.1).sin() + 0.020 * (t * 3.4).cos();
                let roll = 0.055 * (t * 1.8).sin();
                let pitch = 0.040 * (t * 2.3).cos();

                let (ship_x, ship_z, yaw) = if low_motion && anim.delivery_timer > 0.0 {
                    (0.0, 1.68, 0.0)
                } else if anim.delivery_timer > 0.0 {
                    // Delivery window (3.2s total): approach -> berth at pier -> depart
                    let u = 1.0 - (anim.delivery_timer / 3.2).clamp(0.0, 1.0);
                    if u < 0.58 {
                        let a = ease_out_cubic(u / 0.58);
                        (-0.85 * (1.0 - a), 1.82 - 0.14 * a, 0.08 * (1.0 - a))
                    } else {
                        let d = (u - 0.58) / 0.42;
                        (1.35 * d * d, 1.68 + 0.45 * d, -0.22 * d)
                    }
                } else {
                    match anim.phase {
                        AuxiliaryPhase::Completed => (-1.65, 2.05, 0.05 * (t * 0.6).sin()),
                        _ => match progress {
                            0 => (-3.35 + 0.25 * (t * 0.45).sin(), 2.35, 0.12),
                            1 => (-1.95 + 0.15 * (t * 0.55).sin(), 1.98, 0.08),
                            _ => (-0.85 + 0.08 * (t * 0.75).sin(), 1.82, 0.04),
                        },
                    }
                };
                tr.translation = Vec3::new(ship_x, water::LEVEL + dy, ship_z);
                tr.rotation = Quat::from_euler(EulerRot::YXZ, yaw, pitch, roll);
                tr.scale = Vec3::splat(ease_out_cubic(m1).max(0.001));
            }
            MerchantPart::DeliveryCrate => {
                if anim.delivery_timer > 1.1 {
                    let u = ((3.2 - anim.delivery_timer) / 2.1).clamp(0.0, 1.0);
                    let z = 1.66 - 0.36 * ease_out_cubic(u);
                    let arc_y = -0.08 + 0.28 * (u * std::f32::consts::PI).sin();
                    tr.translation = Vec3::new(0.0, arc_y, z);
                    tr.scale = Vec3::splat(1.0);
                } else {
                    tr.scale = Vec3::ZERO;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::art::mesh;
    use bevy::mesh::VertexAttributeValues;

    const PAVED_HALF_WIDTH: f32 = 2.67;
    const PAVED_MIN_X: f32 = -1.1;
    const PAVED_MAX_X: f32 = 17.1;

    fn assert_outside_road(m: &Mesh, world_tr: Transform, label: &str) {
        let Some(VertexAttributeValues::Float32x3(points)) = m.attribute(Mesh::ATTRIBUTE_POSITION)
        else {
            panic!("{label}: missing vertex positions");
        };
        for p in points {
            let w = world_tr.transform_point(Vec3::from(*p));
            let in_road = w.y > 0.02
                && w.x > PAVED_MIN_X + 0.001
                && w.x < PAVED_MAX_X - 0.001
                && w.z > -PAVED_HALF_WIDTH + 0.001
                && w.z < PAVED_HALF_WIDTH - 0.001;
            assert!(
                !in_road,
                "{label} intruded into the BattleLine road at world point {w:?}"
            );
        }
    }

    #[test]
    fn p74_auxiliary_precincts_and_ships_clear_the_wide_road_across_all_eight_arenas() {
        for arena in tbg_core::Arena::ALL {
            let theme = Theme::from(arena);
            let pal = AuxPalette::for_theme(theme);
            for seat in Seat::BOTH {
                let blue = seat == Seat::A;
                let mine_root = precinct_transform(seat, AuxiliaryKind::Miner);
                for (name, m, local) in [
                    ("outcrop", models::mine_outcrop(pal), Transform::default()),
                    ("shoring", models::mine_shoring(pal), Transform::default()),
                    ("rails", models::mine_rails(pal), Transform::default()),
                    (
                        "cart",
                        models::mine_cart(pal),
                        Transform::from_xyz(0.62, 0.03, 0.0),
                    ),
                    (
                        "miner",
                        models::miner_body(blue),
                        Transform::from_xyz(0.46, 0.0, 0.22),
                    ),
                ] {
                    assert!(
                        mesh::triangles(&m) > 10,
                        "{theme:?} {name} must have geometry"
                    );
                    assert_outside_road(
                        &m,
                        mine_root * local,
                        &format!("{theme:?} {seat:?} {name}"),
                    );
                }

                let merch_root = precinct_transform(seat, AuxiliaryKind::Merchant);
                for (name, m, local) in [
                    (
                        "pilings",
                        models::merchant_pier_pilings(pal),
                        Transform::default(),
                    ),
                    (
                        "deck",
                        models::merchant_pier_deck(pal),
                        Transform::default(),
                    ),
                    (
                        "stall_frame",
                        models::merchant_stall_frame(pal),
                        Transform::default(),
                    ),
                    (
                        "stall_awning",
                        models::merchant_stall_awning(pal),
                        Transform::default(),
                    ),
                    (
                        "stall_cargo",
                        models::merchant_stall_cargo(pal),
                        Transform::default(),
                    ),
                    (
                        "merchant",
                        models::merchant_body(blue),
                        Transform::from_xyz(0.0, 0.0, 0.68),
                    ),
                    (
                        "ship_horizon",
                        models::merchant_ship(pal, blue),
                        Transform::from_xyz(-3.35, water::LEVEL, 2.35),
                    ),
                    (
                        "ship_docked",
                        models::merchant_ship(pal, blue),
                        Transform::from_xyz(0.0, water::LEVEL, 1.68),
                    ),
                ] {
                    assert!(
                        mesh::triangles(&m) > 10,
                        "{theme:?} {name} must have geometry"
                    );
                    assert_outside_road(
                        &m,
                        merch_root * local,
                        &format!("{theme:?} {seat:?} {name}"),
                    );
                }
            }
        }
    }
}
