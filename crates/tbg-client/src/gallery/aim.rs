//! Aim regression lab for testing directional aiming against 8 angles, near/far, retargeting, and crowds.

use super::{figures::spawn_one, settings::Gallery};
use crate::art::{
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
            commands.entity(target_root).insert(target_anim);
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
            attacker_anim.play(match attacker_kind {
                UnitKind::Archer | UnitKind::SiegeWalker => Clip::AttackRanged,
                UnitKind::Hexer
                | UnitKind::Bloomkeeper
                | UnitKind::Rootcaller
                | UnitKind::Nester => Clip::Cast,
                _ => Clip::AttackMelee,
            });
            commands.entity(attacker_root).insert((
                attacker_anim,
                AimAttacker {
                    kind: attacker_kind,
                    target_index: a % 3,
                    far: false,
                    timer: 0.0,
                    interval: 1.2,
                },
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
        attacker_anim.play(match attacker_kind {
            UnitKind::Archer | UnitKind::SiegeWalker => Clip::AttackRanged,
            UnitKind::Hexer | UnitKind::Bloomkeeper | UnitKind::Rootcaller | UnitKind::Nester => {
                Clip::Cast
            }
            _ => Clip::AttackMelee,
        });

        commands.entity(attacker_root).insert((
            attacker_anim,
            AimAttacker {
                kind: attacker_kind,
                target_index: options.angle % 8,
                far: options.far,
                timer: 0.0,
                interval: 1.2,
            },
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
    mut attackers: Query<(&mut Animator, &mut AimAttacker)>,
    mut targets: Query<(&AimTarget, &mut Visibility)>,
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

    for (mut anim, mut attacker) in &mut attackers {
        attacker.timer += time.delta_secs();
        if attacker.timer >= attacker.interval {
            attacker.timer = 0.0;
            let clip = match attacker.kind {
                UnitKind::Archer | UnitKind::SiegeWalker => Clip::AttackRanged,
                UnitKind::Hexer
                | UnitKind::Bloomkeeper
                | UnitKind::Rootcaller
                | UnitKind::Nester => Clip::Cast,
                _ => Clip::AttackMelee,
            };
            anim.play(clip);
        }

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

        for (target, mut vis) in &mut targets {
            if target.index == current_target && target.far == current_far {
                if kill_target {
                    *vis = Visibility::Hidden;
                } else if *vis == Visibility::Hidden && !kill_target {
                    *vis = Visibility::Inherited;
                }
            } else if target.far != current_far {
                *vis = Visibility::Hidden;
            }
        }
    }
}
