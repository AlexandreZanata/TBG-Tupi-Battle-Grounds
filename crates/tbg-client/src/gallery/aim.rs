//! Aim regression lab for testing directional aiming against 8 angles, near/far, retargeting, and crowds.

use super::{figures::spawn_one, settings::Gallery};
use crate::art::{
    aim::{Aim, BASE_YAW_OWN},
    anim::{Animator, Clip},
    FigureLibrary,
};
use bevy::prelude::*;
use tbg_core::UnitKind;

#[derive(Component)]
pub struct AimAttacker {
    pub kind: UnitKind,
    pub target_index: usize,
    pub far: bool,
    pub timer: f32,
    pub interval: f32,
}

#[derive(Component)]
pub struct AimTarget {
    pub index: usize,
    pub far: bool,
}

pub fn spawn(
    commands: &mut Commands,
    lib: &FigureLibrary,
    options: &Gallery,
    mut mats: ResMut<Assets<StandardMaterial>>,
    _window: &Window,
) {
    let material = if options.silhouette {
        mats.add(StandardMaterial {
            base_color: Color::BLACK,
            unlit: true,
            ..default()
        })
    } else {
        lib.material.clone()
    };

    let attacker_kind = options.unit.unwrap_or(UnitKind::Archer);
    let target_kind = if attacker_kind == UnitKind::BoneBrute {
        UnitKind::Shieldbearer
    } else {
        UnitKind::BoneBrute
    };

    let center = Vec3::new(8.0, 0.0, 0.0);

    if options.crowd {
        // Crowd mode: 12 attackers surrounding 3 targets.
        for t in 0..3 {
            let angle = t as f32 * std::f32::consts::TAU / 3.0;
            let pos = center + Vec3::new(angle.sin() * 1.8, 0.0, angle.cos() * 1.8);
            let (target_root, mut target_anim) =
                spawn_one(commands, lib, target_kind, (0, 0), pos, &material);
            target_anim.play(Clip::Idle);
            commands.entity(target_root).insert((
                target_anim,
                AimTarget {
                    index: t,
                    far: false,
                },
            ));
        }
        for a in 0..12 {
            let angle = a as f32 * std::f32::consts::TAU / 12.0;
            let pos = center + Vec3::new(angle.sin() * 4.0, 0.0, angle.cos() * 4.0);
            let (attacker_root, mut attacker_anim) = spawn_one(
                commands,
                lib,
                attacker_kind,
                (options.tier, options.branch),
                pos,
                &material,
            );
            let clip = match attacker_kind {
                UnitKind::Archer | UnitKind::SiegeWalker => Clip::AttackRanged,
                UnitKind::Hexer
                | UnitKind::Bloomkeeper
                | UnitKind::Rootcaller
                | UnitKind::Nester => Clip::Cast,
                _ => Clip::AttackMelee,
            };
            attacker_anim.play(clip);
            let hold = match clip {
                Clip::AttackRanged => 0.18,
                Clip::Cast => 0.22,
                _ => 0.30,
            };
            let mut aim = Aim::new(
                tbg_core::Target::Unit(a as u32),
                0.0,
                hold,
                BASE_YAW_OWN,
                BASE_YAW_OWN,
            );
            aim.last_target_point = Some(center);
            commands.entity(attacker_root).insert((
                attacker_anim,
                AimAttacker {
                    kind: attacker_kind,
                    target_index: a % 3,
                    far: false,
                    timer: 0.0,
                    interval: 1.2,
                },
                aim,
            ));
        }
    } else {
        // Standard 8-angle ring mode.
        let (attacker_root, mut attacker_anim) = spawn_one(
            commands,
            lib,
            attacker_kind,
            (options.tier, options.branch),
            center,
            &material,
        );
        let clip = match attacker_kind {
            UnitKind::Archer | UnitKind::SiegeWalker => Clip::AttackRanged,
            UnitKind::Hexer | UnitKind::Bloomkeeper | UnitKind::Rootcaller | UnitKind::Nester => {
                Clip::Cast
            }
            _ => Clip::AttackMelee,
        };
        attacker_anim.play(clip);
        let hold = match clip {
            Clip::AttackRanged => 0.18,
            Clip::Cast => 0.22,
            _ => 0.30,
        };

        let target_dist = if options.far { 4.2 } else { 1.6 };
        let target_angle = (options.angle % 8) as f32 * std::f32::consts::TAU / 8.0;
        let initial_target_pos = center
            + Vec3::new(
                target_angle.sin() * target_dist,
                0.0,
                target_angle.cos() * target_dist,
            );

        let mut aim = Aim::new(
            tbg_core::Target::Unit(0),
            0.0,
            hold,
            BASE_YAW_OWN,
            BASE_YAW_OWN,
        );
        aim.last_target_point = Some(initial_target_pos);

        commands.entity(attacker_root).insert((
            attacker_anim,
            AimAttacker {
                kind: attacker_kind,
                target_index: options.angle % 8,
                far: options.far,
                timer: 0.0,
                interval: 1.2,
            },
            aim,
        ));

        // Spawn targets at 8 angles (both near and far).
        for i in 0..8 {
            let angle = i as f32 * std::f32::consts::TAU / 8.0;
            for far in [false, true] {
                let dist = if far { 4.2 } else { 1.6 };
                let pos = center + Vec3::new(angle.sin() * dist, 0.0, angle.cos() * dist);
                let (target_root, mut target_anim) =
                    spawn_one(commands, lib, target_kind, (0, 0), pos, &material);
                target_anim.play(Clip::Idle);
                let visible = (!options.far && !far) || (options.far && far);
                if !visible {
                    commands.entity(target_root).insert(Visibility::Hidden);
                }
                commands
                    .entity(target_root)
                    .insert((target_anim, AimTarget { index: i, far }));
            }
        }
    }

    super::title::spawn(commands, options, attacker_kind);
}

pub fn update(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mut attackers: Query<(
        &mut Animator,
        &mut AimAttacker,
        Option<&mut Aim>,
        &Transform,
    )>,
    mut targets: Query<
        (&AimTarget, &mut Visibility, &Transform, &mut Animator),
        Without<AimAttacker>,
    >,
    mut transforms: Query<&mut Transform, (Without<AimAttacker>, Without<AimTarget>)>,
) {
    let toggle_far = keys.just_pressed(KeyCode::KeyN);
    let kill_target = keys.just_pressed(KeyCode::KeyK);
    let mut selected_angle = None;

    if keys.just_pressed(KeyCode::Digit1) {
        selected_angle = Some(0);
    }
    if keys.just_pressed(KeyCode::Digit2) {
        selected_angle = Some(1);
    }
    if keys.just_pressed(KeyCode::Digit3) {
        selected_angle = Some(2);
    }
    if keys.just_pressed(KeyCode::Digit4) {
        selected_angle = Some(3);
    }
    if keys.just_pressed(KeyCode::Digit5) {
        selected_angle = Some(4);
    }
    if keys.just_pressed(KeyCode::Digit6) {
        selected_angle = Some(5);
    }
    if keys.just_pressed(KeyCode::Digit7) {
        selected_angle = Some(6);
    }
    if keys.just_pressed(KeyCode::Digit8) {
        selected_angle = Some(7);
    }
    if keys.just_pressed(KeyCode::Space) {
        selected_angle = Some(usize::MAX); // cycle
    }

    for (mut anim, mut attacker, mut aim_opt, attacker_t) in &mut attackers {
        attacker.timer += time.delta_secs();

        if let Some(angle) = selected_angle {
            if angle == usize::MAX {
                attacker.target_index = (attacker.target_index + 1) % 8;
            } else {
                attacker.target_index = angle;
            }
        }
        if toggle_far {
            attacker.far = !attacker.far;
        }

        let current_target = attacker.target_index;
        let current_far = attacker.far;
        let mut active_target_pos = None;

        let strike_triggered = anim.events.contains(&crate::art::anim::ClipEvent::Strike);

        for (target, mut vis, target_t, mut target_anim) in &mut targets {
            if target.index == current_target && target.far == current_far {
                if kill_target {
                    *vis = Visibility::Hidden;
                } else if *vis == Visibility::Hidden && !kill_target {
                    *vis = Visibility::Inherited;
                }
                if *vis != Visibility::Hidden {
                    active_target_pos = Some(target_t.translation);
                    if strike_triggered {
                        target_anim.play_if_free(Clip::Hit);
                        target_anim.apply_hit_stop(0.050);
                    }
                }
            } else if target.far != current_far {
                *vis = Visibility::Hidden;
            }
        }

        let clip = match attacker.kind {
            UnitKind::Archer | UnitKind::SiegeWalker => Clip::AttackRanged,
            UnitKind::Hexer | UnitKind::Bloomkeeper | UnitKind::Rootcaller | UnitKind::Nester => {
                Clip::Cast
            }
            _ => Clip::AttackMelee,
        };

        if attacker.timer >= attacker.interval {
            attacker.timer = 0.0;
            anim.play(clip);
            let hold = match clip {
                Clip::AttackRanged => 0.18,
                Clip::Cast => 0.22,
                _ => 0.30,
            };
            if let Some(ref mut aim) = aim_opt {
                aim.retarget(
                    tbg_core::Target::Unit(current_target as u32),
                    time.elapsed_secs_f64(),
                    hold,
                );
            }
        }

        if let Some(ref mut aim) = aim_opt {
            aim.update(
                time.elapsed_secs_f64(),
                time.delta_secs(),
                active_target_pos,
                attacker_t.translation,
            );

            let profile = crate::art::melee::strike_profile_for_unit(attacker.kind, 0, 0);
            let lunge_dist =
                crate::art::melee::lunge_distance(attacker.timer, 0.30, 0.50, profile.lunge);
            let lunge_offset = Vec3::new(
                aim.current_yaw.sin() * lunge_dist,
                0.0,
                aim.current_yaw.cos() * lunge_dist,
            );

            if let Ok(mut fig_t) = transforms.get_mut(anim.figure) {
                fig_t.rotation = Quat::from_rotation_y(aim.current_yaw);
                fig_t.translation = lunge_offset;
            }
        }
    }
}
