//! Debug-only laboratory journey using the same identity-bound actions as visible controls.
use crate::{
    training::{Action, Requests, Tab, Trial},
    GameState,
};
use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Update, journey);
    if crate::config::url_param("capture_legacy_crowd").is_some() {
        super::packing::plugin(app);
    }
    if crate::config::url_param("capture_barrel").is_some() {
        super::barrel::plugin(app);
    }
    if crate::config::url_param("capture_support_rear").is_some() {
        app.init_resource::<SupportProbe>().add_systems(
            PostUpdate,
            support_probe
                .after(crate::art::anim::animate)
                .after(bevy::transform::TransformSystems::Propagate),
        );
    }
}
fn journey(
    state: Res<State<GameState>>,
    time: Res<Time>,
    trial: Res<Trial>,
    mut queue: ResMut<Requests>,
    mut stage: Local<u8>,
    mut observed: Local<(u32, u32, f32)>,
    mut catalog: ResMut<crate::training::Catalog>,
) {
    if *stage == 0 && *state.get() == GameState::Menu {
        if crate::config::url_param("capture_support_rear").is_some() {
            use tbg_core::{training::TrainingUnitSpec, UnitKind};
            assert_eq!(
                tbg_core::content::def(catalog.selected).archetype,
                tbg_core::Archetype::Support
            );
            let placed = |kind, x| TrainingUnitSpec {
                x: Some(x),
                ..TrainingUnitSpec::base(kind)
            };
            // The Furnace wounds itself through real rules; the Shieldbearer attacks the front healer.
            catalog.allies = vec![
                placed(catalog.selected, 8_000),
                placed(UnitKind::BloodFurnace, 6_000),
            ];
            catalog.enemies = vec![placed(UnitKind::Shieldbearer, 8_600)];
            catalog.dummies = false;
        }
        if crate::config::url_param("capture_ogre_crowd").is_some() {
            use tbg_core::{training::TrainingUnitSpec, UnitKind::*};
            let placed = |kind, tier, branch, x| TrainingUnitSpec {
                x: Some(x),
                tier,
                branch,
                ..TrainingUnitSpec::base(kind)
            };
            catalog.allies = vec![
                placed(SiegeWalker, 0, 0, 7800),
                placed(SiegeWalker, 2, 0, 7400),
                placed(BasaltBulwark, 0, 0, 8400),
                placed(HookBrawler, 0, 0, 8200),
                placed(HideMender, 0, 0, 6000),
                placed(RivetBallista, 0, 0, 6900),
            ];
            catalog.enemies = vec![
                placed(Shieldbearer, 0, 0, 10400),
                placed(Longaxe, 0, 0, 11100),
                placed(BoneBrute, 0, 0, 10900),
                placed(Hexer, 0, 0, 12300),
                placed(Archer, 0, 0, 12900),
            ];
            catalog.dummies = false;
            eprintln!("Ogre crowd scene configured through the authoritative Training catalog");
        }
        if crate::config::url_param("capture_barrel").is_some() {
            use tbg_core::{training::TrainingUnitSpec, UnitKind::*};
            let placed = |kind, x| TrainingUnitSpec {
                x: Some(x),
                ..TrainingUnitSpec::base(kind)
            };
            catalog.selected = GustDrummer;
            catalog.allies = vec![
                placed(GustDrummer, 8000),
                placed(BloodFurnace, 6300),
                placed(BloodFurnace, 6700),
            ];
            catalog.enemies = vec![placed(
                Shieldbearer,
                if crate::config::url_param("capture_barrel_intercept").is_some() {
                    7800
                } else {
                    // Both Furnaces can attack here without walking into the caster.
                    5900
                },
            )];
            if crate::config::url_param("capture_barrel_intercept").is_some() {
                // PK-01 needs legal body space; the burst reaches the wounded carrier and near Furnace.
                catalog.allies = vec![
                    placed(GustDrummer, 9200),
                    placed(BloodFurnace, 5000),
                    placed(BloodFurnace, 6800),
                ];
            }
            catalog.dummies = false;
        }
        if crate::config::url_param("capture_abyssal_crowd").is_some() {
            use tbg_core::{training::TrainingUnitSpec, UnitKind::*};
            let placed = |kind, tier, branch, x| TrainingUnitSpec {
                x: Some(x),
                tier,
                branch,
                ..TrainingUnitSpec::base(kind)
            };
            catalog.allies = vec![
                placed(Trident, 0, 0, 8400),
                placed(NacreDuelist, 0, 0, 8200),
                placed(ReefLancer, 1, 0, 7900),
                placed(TrenchFist, 0, 0, 8000),
                placed(PearlSlinger, 0, 0, 6600),
                placed(AbyssHarpooner, 0, 0, 6200),
                placed(UndertowHerald, 1, 1, 7200),
                placed(TrenchLamplighter, 2, 0, 6400),
                placed(FoamWeaver, 0, 0, 5800),
            ];
            catalog.enemies = vec![
                placed(BasaltBulwark, 0, 0, 10400),
                placed(HookBrawler, 0, 0, 10900),
                placed(SiegeWalker, 0, 0, 12300),
                placed(PodSnapper, 0, 0, 11800),
                placed(GustDrummer, 0, 0, 13100),
            ];
            catalog.dummies = false;
        }
        if crate::config::url_param("capture_legacy_crowd").is_some() {
            use tbg_core::{training::TrainingUnitSpec, UnitKind::*};
            let placed = |kind, x| TrainingUnitSpec {
                x: Some(x),
                ..TrainingUnitSpec::base(kind)
            };
            catalog.selected = Colossus;
            catalog.allies = vec![placed(Colossus, 9000)];
            catalog.allies.extend((0..10).map(|_| placed(Archer, 7000)));
            catalog.allies.push(placed(FoamWeaver, 5000));
            catalog.enemies = vec![placed(BasaltBulwark, 13500)];
            catalog.dummies = false;
        }
        queue.0.push(Action::Open(Tab::Editor));
        *stage = 1;
        return;
    }
    if *stage == 1 && time.elapsed_secs() > 1. {
        queue.0.push(Action::Start);
        *stage = 2;
        return;
    }
    let Some(s) = &trial.session else { return };
    let id = s.id();
    match *stage {
        2 if if crate::config::url_param("capture_barrel").is_some() {
            !s.state().barrels.is_empty()
        } else {
            s.active_ticks() >= 10
        } =>
        {
            queue.0.push(Action::Control(id, 0));
            *observed = (id.0, s.active_ticks(), time.elapsed_secs());
            *stage = 3;
        }
        3 if crate::config::url_param("capture_legacy_crowd").is_some() => {}
        3 if time.elapsed_secs() > observed.2 + 0.5 => {
            assert!(matches!(
                s.lifecycle(),
                tbg_core::training::Lifecycle::Paused(_)
            ));
            observed.1 = s.active_ticks();
            queue.0.push(Action::Control(id, 1));
            *stage = 4;
        }
        4 if s.active_ticks() > observed.1 => {
            assert_eq!(s.active_ticks(), observed.1 + 1);
            queue.0.push(Action::Control(id, 2));
            *stage = 5;
        }
        5 if id.0 != observed.0 => {
            assert_eq!(s.active_ticks(), 0);
            queue.0.push(Action::Control(
                tbg_core::training::SessionId(observed.0),
                0,
            ));
            *stage = 6;
        }
        6 => {
            assert_eq!(
                s.active_ticks(),
                0,
                "stale controls must not start the reset trial"
            );
            queue.0.push(Action::Control(id, 0));
            *stage = 7;
        }
        7 if s.active_ticks() >= 20 => {
            assert_eq!(*state.get(), GameState::Training);
            assert!(s.state().outcome.is_none());
            eprintln!("Training journey passed: pause, exact single step, reset identity, stale action rejection and independent local rendering");
            *stage = 8;
        }
        _ => {}
    }
}

#[derive(Resource, Default)]
pub(super) struct SupportProbe {
    pub(super) ready: bool,
    rear_frames: u32,
    damaged: bool,
    max_yaw: f32,
}

fn support_probe(
    catalog: Res<crate::training::Catalog>,
    latest: Res<crate::sync::LatestView>,
    units: Query<(
        &crate::visuals::UnitVisual,
        &crate::art::anim::Animator,
        &Transform,
    )>,
    transforms: Query<&Transform>,
    mut probe: ResMut<SupportProbe>,
) {
    if probe.ready {
        return;
    }
    let Some(view) = &latest.0 else { return };
    probe.damaged |= view
        .units
        .iter()
        .any(|u| u.kind == catalog.selected && u.hp < u.max_hp);
    for (visual, animator, root) in &units {
        if visual.kind() != catalog.selected || visual.dying || !animator.aim_is_ally {
            continue;
        }
        let Some(target) = animator.aim_at else {
            continue;
        };
        let direction = if visual.own { 1.0 } else { -1.0 };
        if (target.x - root.translation.x) * direction >= -0.05 {
            continue;
        }
        let joint = animator.joints[crate::art::rig::Joint::Root.index()];
        let Ok(transform) = transforms.get(joint) else {
            continue;
        };
        let forward = transform.rotation * Vec3::Z;
        let yaw = forward.x.atan2(forward.z).abs();
        assert!(
            yaw < 0.02,
            "rear heal/hit changed the support's foot heading: {yaw}"
        );
        probe.rear_frames += 1;
        probe.max_yaw = probe.max_yaw.max(yaw);
    }
    if probe.rear_frames >= 60 && probe.damaged {
        probe.ready = true;
        eprintln!("Support rear-heal visual regression passed: kind={:?}, rear_frames={}, attacked=true, max_foot_yaw={}", catalog.selected, probe.rear_frames, probe.max_yaw);
    }
}
