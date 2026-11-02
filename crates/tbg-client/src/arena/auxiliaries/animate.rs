//! War Auxiliary motion: construction (bridge, platform, precinct), working
//! loops that grow livelier with every tier, Merchant payouts and tier reveals.

use super::{
    Approach, AuxPartOwner, AuxPrecinctRoot, AuxiliarySceneState, MerchantPart, MinePart,
    SlotAnimState, TierPart, TierRing,
};
use crate::arena::water;
use crate::sync::LatestView;
use crate::GameState;
use bevy::prelude::*;
use std::f32::consts::PI;
use tbg_core::auxiliary::{AuxiliaryPhase, AUXILIARY_RULE};

/// Merchant ship approach, berth and departure after each payout.
pub(super) const DELIVERY_SECS: f32 = 3.2;
/// Client-side construction length of the Merchant (it has no preparation phase).
const MERCHANT_BUILD_SECS: f32 = 3.0;
/// Share of construction spent laying the bridge and the platform.
const APPROACH_SHARE: f32 = 0.3;
const REVEAL_SECS: f32 = 1.1;
const RING_SECS: f32 = 1.3;

fn ease_out_cubic(t: f32) -> f32 {
    let inv = 1.0 - t.clamp(0.0, 1.0);
    1.0 - inv * inv * inv
}

fn spring_pop(t: f32) -> f32 {
    let x = t.clamp(0.0, 1.0);
    if x >= 1.0 {
        1.0
    } else {
        1.0 - (-6.0 * x).exp() * (8.5 * x).cos()
    }
}

fn tier_ix(anim: &SlotAnimState) -> usize {
    usize::from(anim.tier.clamp(1, 3) - 1)
}

/// Seconds of loop time; frozen under low motion so poses stay still.
fn loop_time(state: &AuxiliarySceneState) -> f32 {
    if crate::config::low_motion() {
        0.0
    } else {
        state.animation_time
    }
}

/// Construction progress in `[0, 1]`: the Miner follows the authoritative
/// preparation countdown (including Augment pauses); the Merchant builds on
/// the client clock right after hiring.
fn build_progress(anim: &SlotAnimState, slot: usize) -> f32 {
    if slot == 1 {
        return (anim.since_hire / MERCHANT_BUILD_SECS).clamp(0.0, 1.0);
    }
    match anim.phase {
        AuxiliaryPhase::Preparing { ticks_left } => {
            let total = AUXILIARY_RULE.miner_prep_ticks.max(1) as f32;
            (1.0 - ticks_left as f32 / total).clamp(0.0, 1.0)
        }
        AuxiliaryPhase::Available => 0.0,
        _ => 1.0,
    }
}

/// Precinct construction after the bridge: `[0, 1]` once the approach is done.
fn precinct_progress(anim: &SlotAnimState, slot: usize) -> f32 {
    ((build_progress(anim, slot) - APPROACH_SHARE * 0.6) / (1.0 - APPROACH_SHARE * 0.6))
        .clamp(0.0, 1.0)
}

pub(super) fn clock(
    time: Res<Time>,
    view: Res<LatestView>,
    mut state: ResMut<AuxiliarySceneState>,
) {
    let paused = view.0.as_ref().is_some_and(|v| v.augment_round.is_some());
    let dt = if paused { 0.0 } else { time.delta_secs() };
    state.animation_time += dt;
    for slot in state
        .slot_anim
        .iter_mut()
        .flatten()
        .filter(|s| s.hired_seen)
    {
        slot.since_hire += dt;
        slot.tier_since += dt;
        slot.delivery_timer = (slot.delivery_timer - dt).max(0.0);
    }
}

pub(super) fn visibility(
    state: Res<AuxiliarySceneState>,
    game_state: Option<Res<State<GameState>>>,
    mut roots: Query<(&AuxPrecinctRoot, &mut Visibility)>,
) {
    let hidden_screen = game_state.is_some_and(|s| *s.get() == GameState::Gallery);
    for (root, mut vis) in &mut roots {
        let anim = state.slot_anim[root.seat.index()][usize::from(root.kind.slot())];
        let show = !hidden_screen && anim.phase != AuxiliaryPhase::Available;
        vis.set_if_neq(if show {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        });
    }
}

/// Bridge planks drop into place one by one, rails follow, then the platform rises.
pub(super) fn construction(
    state: Res<AuxiliarySceneState>,
    mut parts: Query<(&Approach, &AuxPartOwner, &mut Transform)>,
) {
    for (approach, owner, mut tr) in &mut parts {
        let (slot, index, count) = match *approach {
            Approach::Platform { slot } | Approach::Rails { slot } => (slot, 0, 1),
            Approach::Plank { slot, index, count } => (slot, index, count),
        };
        let anim = &state.slot_anim[owner.0.index()][slot];
        let u = build_progress(anim, slot) / APPROACH_SHARE;
        match approach {
            Approach::Platform { .. } => {
                let s = ease_out_cubic((u - 0.5) * 2.0).max(0.001);
                tr.scale = Vec3::new(1.0, s, 1.0);
            }
            Approach::Rails { .. } => tr.scale = Vec3::new(u.clamp(0.001, 1.0), 1.0, 1.0),
            Approach::Plank { .. } => {
                let start = index as f32 / count as f32 * 0.8;
                let e = ease_out_cubic((u - start) / 0.2);
                tr.translation.y = (1.0 - e) * 0.32;
                tr.scale = Vec3::splat(e.max(0.001));
            }
        }
    }
}

/// Tier structures pop in when bought; the golden ring sweeps out once.
pub(super) fn tiers(
    state: Res<AuxiliarySceneState>,
    mut parts: Query<(&TierPart, &AuxPartOwner, &mut Transform), Without<TierRing>>,
    mut rings: Query<(&TierRing, &AuxPartOwner, &mut Transform), Without<TierPart>>,
) {
    let instant = crate::config::low_motion();
    for (part, owner, mut tr) in &mut parts {
        let anim = &state.slot_anim[owner.0.index()][part.slot];
        let reveal = if anim.tier < part.tier {
            0.0
        } else if anim.tier > part.tier || instant {
            1.0
        } else {
            spring_pop(anim.tier_since / REVEAL_SECS)
        };
        // Never fully zero: a zero scale makes the renderer drop the subtree's bounds.
        tr.scale = Vec3::splat(reveal.max(0.0001));
    }
    for (ring, owner, mut tr) in &mut rings {
        let anim = &state.slot_anim[owner.0.index()][ring.0];
        let u = anim.tier_since / RING_SECS;
        let active = anim.tier >= 2 && u < 1.0 && !instant;
        let s = if active {
            0.4 + 1.4 * ease_out_cubic(u)
        } else {
            0.0
        };
        let fade = if active { 1.0 - u * u } else { 0.0 };
        let center = super::pad_center(tbg_core::auxiliary::AuxiliaryKind::ALL[ring.0]);
        tr.translation = Vec3::new(center.x, 0.02 + 0.35 * u, center.y);
        tr.scale = Vec3::new(s, fade.max(0.001), s);
    }
}

/// Pickaxe swing cycle in seconds; every tier works faster.
const SWING_SECS: [f32; 3] = [1.35, 1.1, 0.9];
const SPARK_SCALE: [f32; 3] = [1.0, 1.3, 1.65];

pub(super) fn mine(
    state: Res<AuxiliarySceneState>,
    mut parts: Query<(&MinePart, &AuxPartOwner, &mut Transform)>,
) {
    let t = loop_time(&state);
    for (part, owner, mut tr) in &mut parts {
        let anim = &state.slot_anim[owner.0.index()][0];
        if anim.phase == AuxiliaryPhase::Available {
            continue;
        }
        let p = precinct_progress(anim, 0);
        let active = matches!(anim.phase, AuxiliaryPhase::Active { .. });
        let cycle = (t / SWING_SECS[tier_ix(anim)]).fract();
        let c1 = (p / 0.35).clamp(0.0, 1.0);
        let c2 = ((p - 0.35) / 0.40).clamp(0.0, 1.0);
        let c3 = ((p - 0.75) / 0.25).clamp(0.0, 1.0);
        match part {
            MinePart::Outcrop => {
                let s = ease_out_cubic(c1).max(0.01);
                tr.translation = Vec3::new(0.0, (1.0 - s) * -0.45, 0.0);
                tr.scale = Vec3::new(1.0, s, 1.0);
            }
            MinePart::Shoring => tr.scale = Vec3::splat(spring_pop(c2).max(0.001)),
            MinePart::Rails => tr.scale = Vec3::new(ease_out_cubic(c2).max(0.001), 1.0, 1.0),
            MinePart::Cart | MinePart::Ore => {
                let s = ease_out_cubic(c3);
                // Tier 2+ carts shuttle to the ore heap instead of only rocking.
                let travel = match (active, anim.tier) {
                    (false, _) => 0.0,
                    (true, 1) => 0.05 * (t * 2.2).sin(),
                    (true, _) => 0.16 * (t * 1.1).sin(),
                };
                let base_x = if *part == MinePart::Cart {
                    0.18 + 0.44 * s
                } else {
                    0.62
                };
                tr.translation = Vec3::new(base_x + travel, 0.03, 0.0);
                tr.scale = Vec3::splat(if *part == MinePart::Cart {
                    s.max(0.001)
                } else {
                    ore_fill(anim)
                });
            }
            MinePart::Lantern => {
                tr.rotation = Quat::from_rotation_z(0.14 * (t * 3.1).sin());
                tr.scale = Vec3::splat(spring_pop(c3).max(0.001));
            }
            MinePart::Sparks => {
                let impact = active && (0.46..=0.66).contains(&cycle);
                let burst = 1.0 - ((cycle - 0.56).abs() / 0.10).clamp(0.0, 1.0);
                tr.translation = Vec3::new(0.22, 0.28 + burst * 0.06, 0.0);
                tr.rotation = Quat::from_rotation_y(t * 6.0);
                let size = (0.4 + 0.9 * burst) * SPARK_SCALE[tier_ix(anim)];
                tr.scale = Vec3::splat(if impact { size } else { 0.0 });
            }
            MinePart::MinerBody => {
                let walk_x = if p < 1.0 {
                    0.92 - 0.46 * ease_out_cubic(c1)
                } else {
                    0.46
                };
                let bob = if p < 1.0 {
                    0.03 * (t * 9.0).sin().abs()
                } else {
                    0.0
                };
                tr.translation = Vec3::new(walk_x, bob, 0.22);
                tr.rotation =
                    Quat::from_rotation_y(PI) * Quat::from_rotation_z(body_lean(anim, t, cycle));
            }
            MinePart::MinerPickaxe => {
                tr.rotation = Quat::from_rotation_z(pickaxe_angle(anim, t, cycle))
            }
            MinePart::WinchDrum => {
                tr.rotation = Quat::from_rotation_z(if active { t * 2.6 } else { 0.0 })
            }
            MinePart::Bucket => {
                tr.translation.y = 0.3 + if active { 0.14 * (t * 1.3).sin() } else { 0.0 }
            }
            MinePart::Sheave => {
                tr.rotation = Quat::from_rotation_z(if active { -t * 3.2 } else { 0.0 })
            }
            MinePart::Banner => tr.rotation = Quat::from_rotation_x(0.1 * (t * 2.0).sin()),
            MinePart::Crystals => {
                let pulse = if active {
                    1.0 + 0.12 * (t * 3.0).sin()
                } else {
                    0.9
                };
                tr.scale = Vec3::splat(pulse);
                tr.rotation = Quat::from_rotation_y(t * 0.4);
            }
            MinePart::Apprentice => {
                // Shovels ore toward the heap: a lean-and-lift loop in place.
                let dig = if active { (t * 2.4).sin() } else { 0.0 };
                tr.translation = super::spots::APPRENTICE + Vec3::Y * 0.02 * dig.abs();
                tr.rotation =
                    Quat::from_rotation_y(-0.35) * Quat::from_rotation_z(-0.22 * dig.max(0.0));
            }
        }
    }
}

fn ore_fill(anim: &SlotAnimState) -> f32 {
    match anim.phase {
        AuxiliaryPhase::Active { ticks_left, .. } => {
            let dur = AUXILIARY_RULE.miner_duration_ticks.max(1) as f32;
            0.28 + 0.72 * (1.0 - (ticks_left as f32 / dur).clamp(0.0, 1.0))
        }
        AuxiliaryPhase::Completed => 1.0,
        _ => 0.001,
    }
}

fn body_lean(anim: &SlotAnimState, t: f32, cycle: f32) -> f32 {
    match anim.phase {
        AuxiliaryPhase::Preparing { .. } => 0.16 * (t * 6.5).sin(),
        AuxiliaryPhase::Active { .. } if cycle < 0.46 => -0.18 * (cycle / 0.46),
        AuxiliaryPhase::Active { .. } if cycle < 0.62 => 0.24,
        _ => 0.0,
    }
}

fn pickaxe_angle(anim: &SlotAnimState, t: f32, cycle: f32) -> f32 {
    match anim.phase {
        AuxiliaryPhase::Preparing { .. } => 0.25 + 0.85 * (t * 6.5).sin(),
        AuxiliaryPhase::Active { .. } if cycle < 0.46 => {
            -0.30 + 1.25 * ease_out_cubic(cycle / 0.46)
        }
        AuxiliaryPhase::Active { .. } if cycle < 0.60 => {
            0.95 - 1.75 * ease_out_cubic((cycle - 0.46) / 0.14)
        }
        AuxiliaryPhase::Active { .. } => -0.80 + 0.50 * ease_out_cubic((cycle - 0.60) / 0.40),
        _ => -0.55,
    }
}

/// Merchant ship size per tier: a sloop, a caravel, then a galleon.
const SHIP_SCALE: [f32; 3] = [1.0, 1.15, 1.32];

pub(super) fn merchant(
    state: Res<AuxiliarySceneState>,
    mut parts: Query<(&MerchantPart, &AuxPartOwner, &mut Transform)>,
) {
    let t = loop_time(&state);
    let theme = state.built_theme.unwrap_or_default();
    for (part, owner, mut tr) in &mut parts {
        let anim = &state.slot_anim[owner.0.index()][1];
        if anim.phase == AuxiliaryPhase::Available {
            continue;
        }
        let p = precinct_progress(anim, 1);
        let m1 = (p / 0.35).clamp(0.0, 1.0);
        let m2 = ((p - 0.35) / 0.37).clamp(0.0, 1.0);
        let m3 = ((p - 0.72) / 0.28).clamp(0.0, 1.0);
        let progress = match anim.phase {
            AuxiliaryPhase::Active { progress, .. } => progress,
            _ => 0,
        };
        let delivering = anim.delivery_timer > 0.0;
        let u = 1.0 - (anim.delivery_timer / DELIVERY_SECS).clamp(0.0, 1.0);
        match part {
            MerchantPart::PierPilings => {
                let s = ease_out_cubic(m1).max(0.001);
                tr.translation = Vec3::new(0.0, (1.0 - s) * 0.45, 0.0);
                tr.scale = Vec3::new(1.0, s, 1.0);
            }
            MerchantPart::PierDeck => tr.scale = Vec3::new(1.0, 1.0, ease_out_cubic(m1).max(0.001)),
            MerchantPart::StallFrame => tr.scale = Vec3::new(1.0, spring_pop(m2).max(0.001), 1.0),
            MerchantPart::StallAwning => {
                tr.rotation = Quat::from_rotation_x(0.025 * (t * 2.4).sin());
                tr.scale = Vec3::splat(spring_pop(m2).max(0.001));
            }
            MerchantPart::StallCargo => tr.scale = Vec3::splat(spring_pop(m3).max(0.001)),
            MerchantPart::MerchantBody => {
                let walk_z = if p < 1.0 {
                    0.12 + 0.42 * ease_out_cubic(m3)
                } else if progress >= 2 || delivering {
                    0.68
                } else {
                    0.52
                };
                tr.translation = Vec3::new(0.0, 0.015 * (t * 2.6).sin().abs(), walk_z);
                tr.rotation = Quat::from_rotation_y(-PI * 0.5);
            }
            MerchantPart::SpyglassArm => {
                let angle = if delivering {
                    0.55 + 0.35 * (t * 7.0).sin()
                } else if progress >= 1 {
                    0.38 + 0.06 * (t * 1.8).sin()
                } else {
                    -0.45 + 0.08 * (t * 1.5).sin()
                };
                tr.rotation = Quat::from_rotation_z(angle);
            }
            MerchantPart::Ship => {
                let heading = super::placement::layout(theme, owner.0).ship_heading;
                let (x, z, yaw) = ship_course(anim, progress, t, u, heading);
                let dy = 0.045 * (t * 2.1).sin() + 0.020 * (t * 3.4).cos();
                tr.translation = Vec3::new(x, water::LEVEL + dy, z);
                tr.rotation = Quat::from_euler(
                    EulerRot::YXZ,
                    yaw,
                    0.04 * (t * 2.3).cos(),
                    0.055 * (t * 1.8).sin(),
                );
                tr.scale = Vec3::splat(ease_out_cubic(m1).max(0.001) * SHIP_SCALE[tier_ix(anim)]);
            }
            MerchantPart::DeliveryCrate => {
                if anim.delivery_timer > 1.1 {
                    let k = ((DELIVERY_SECS - anim.delivery_timer) / 2.1).clamp(0.0, 1.0);
                    let z = 1.66 - 0.36 * ease_out_cubic(k);
                    tr.translation = Vec3::new(0.0, -0.08 + 0.28 * (k * PI).sin(), z);
                    tr.scale = Vec3::ONE;
                } else {
                    tr.scale = Vec3::ZERO;
                }
            }
            MerchantPart::Pennant => tr.rotation = Quat::from_rotation_y(0.35 * (t * 2.2).sin()),
            MerchantPart::Pavilion => tr.rotation = Quat::from_rotation_z(0.015 * (t * 1.7).sin()),
            MerchantPart::ChestLid => {
                let open = if delivering { (u * PI).sin() } else { 0.0 };
                tr.rotation = Quat::from_rotation_x(-1.2 * open);
            }
            MerchantPart::Coins => {
                let show = delivering && (0.25..0.9).contains(&u);
                let k = ((u - 0.25) / 0.65).clamp(0.0, 1.0);
                tr.translation = Vec3::new(0.0, 0.12 + 0.45 * (k * PI).sin(), 0.0);
                tr.rotation = Quat::from_rotation_y(t * 8.0);
                tr.scale = Vec3::splat(if show { 1.0 - 0.4 * k } else { 0.0 });
            }
        }
    }
}

/// Ship lane distances from the berth: waiting far out, and fully loaded.
const SHIP_FAR: f32 = 2.8;
const SHIP_NEAR: f32 = 1.0;

/// Yaw that points the ship's bow (model +X) along local direction `dir`.
fn bow_yaw(dir: Vec2) -> f32 {
    (-dir.y).atan2(dir.x)
}

/// Ship position (precinct-local x, z) and yaw along the Arena's sea lane:
/// it closes in as purchases accumulate, berths at the pier on each payout,
/// sails back out, and lies at anchor once the Merchant is done.
fn ship_course(
    anim: &SlotAnimState,
    progress: u8,
    t: f32,
    u: f32,
    heading: f32,
) -> (f32, f32, f32) {
    let out = Vec2::new(heading.cos(), heading.sin());
    let need = AUXILIARY_RULE.merchant_purchases_per_activation[tier_ix(anim)].max(1);
    let wait =
        |p: u8| SHIP_FAR - (SHIP_FAR - SHIP_NEAR) * f32::from(p) / f32::from(need - 1).max(1.0);
    let inbound = bow_yaw(-out);
    let (distance, yaw) = if anim.delivery_timer > 0.0 && crate::config::low_motion() {
        (0.0, inbound)
    } else if anim.delivery_timer > 0.0 && u < 0.58 {
        (wait(need - 1) * (1.0 - ease_out_cubic(u / 0.58)), inbound)
    } else if anim.delivery_timer > 0.0 {
        let d = (u - 0.58) / 0.42;
        // Turn about at the pier, then run back out to the waiting point.
        let turn = ease_out_cubic(d / 0.35);
        (SHIP_FAR * d * d, inbound + PI * turn)
    } else if anim.phase == AuxiliaryPhase::Completed {
        (
            1.7 + 0.1 * (t * 0.6).sin(),
            inbound + 0.05 * (t * 0.6).sin(),
        )
    } else {
        (
            wait(progress.min(need - 1)) + 0.18 * (t * 0.5).sin(),
            inbound + 0.06 * (t * 0.7).sin(),
        )
    };
    let at = super::placement::ship_point(heading, distance);
    (at.x, at.y, yaw)
}
