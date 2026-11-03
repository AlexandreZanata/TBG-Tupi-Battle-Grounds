//! Server events drive independent ability poses, contact effects and hit recoil.
use super::{ground_y, Spawned, UnitVisual};
use crate::{
    art::anim::Animator,
    link::Feed,
    scene::to_world,
    vfx::{
        impacts::{self, Area, Impact},
        shots, VfxAssets,
    },
};
use bevy::prelude::*;
use tbg_moba_core::{state::Event, Champion};

pub(super) fn apply(
    mut commands: Commands,
    mut feed: ResMut<Feed>,
    spawned: Res<Spawned>,
    assets: Res<VfxAssets>,
    mut units: Query<(&mut UnitVisual, &Transform, &mut Animator)>,
    globals: Query<&GlobalTransform>,
) {
    for event in std::mem::take(&mut feed.motion_events) {
        match event {
            Event::ChampionAbility {
                unit,
                kind,
                target,
                at,
                radius,
                started,
                ..
            } => {
                let p = Vec2::new(at.x as f32 / 1000.0, at.y as f32 / 1000.0);
                let at = to_world(p.x, p.y, ground_y(p) + 0.6);
                let target_pos = spawned
                    .by_id
                    .get(&target)
                    .and_then(|e| globals.get(*e).ok())
                    .map(|g| g.translation() + Vec3::Y * 1.8)
                    .unwrap_or(at);
                let visual = spawned
                    .by_id
                    .get(&unit)
                    .and_then(|e| units.get_mut(*e).ok());
                if started {
                    if let Some((_, _, mut a)) = visual {
                        a.special_at = Some(target_pos);
                        a.start_special();
                    }
                    continue;
                }
                let form = visual.as_ref().map_or((0, 0), |(_, _, a)| a.form);
                if let Some((mut v, _, mut a)) = visual {
                    a.resolve_special();
                    if kind == Champion::Trigger {
                        v.secondary_shot = Some(target_pos);
                    }
                    if matches!(kind, Champion::Nester | Champion::Smuggler) {
                        a.gesture = 0.0;
                    }
                    if kind == Champion::RattleTick {
                        let socket =
                            crate::art::rig::Node::Prop(crate::art::rig::Socket::Nock).index();
                        if let Ok(g) = globals.get(a.joints[socket]) {
                            shots::fire(
                                &mut commands,
                                &assets,
                                shots::ShotKind::Bullet,
                                g.transform_point(Vec3::new(0.0, 0.04, 0.24)),
                                target_pos,
                                Area::default(),
                            );
                        }
                    }
                }
                if let Some(impact) = special_impact(kind, form) {
                    impacts::land(
                        &mut commands,
                        &assets,
                        impact,
                        at,
                        Area {
                            splash: radius as f32 / 1000.0,
                            hold: 0.0,
                        },
                    );
                }
            }
            Event::CombatTrace {
                kind,
                unit,
                from,
                to,
            } => {
                let world = |p: tbg_moba_core::geom::Pos| {
                    let p = Vec2::new(p.x as f32, p.y as f32) / 1000.0;
                    to_world(p.x, p.y, ground_y(p) + 0.6)
                };
                let (from, to) = (world(from), world(to));
                match kind {
                    Champion::PyreWarden => {
                        let nozzle = spawned
                            .by_id
                            .get(&unit)
                            .and_then(|e| units.get_mut(*e).ok())
                            .map(|(_, _, a)| {
                                let socket =
                                    crate::art::rig::Node::Prop(crate::art::rig::Socket::WeaponR);
                                (
                                    a.joints[socket.index()],
                                    crate::art::kits::pyre_warden::muzzle(a.form),
                                )
                            });
                        crate::vfx::flame::profane_flame(&mut commands, &assets, from, to, nozzle);
                    }
                    Champion::Volthar | Champion::Bloodweaver | Champion::Acidmaw => {
                        let shot = match kind {
                            Champion::Volthar => shots::ShotKind::StormArc,
                            Champion::Bloodweaver => shots::ShotKind::Blood,
                            _ => shots::ShotKind::Acid,
                        };
                        shots::fire(&mut commands, &assets, shot, from, to, Area::default());
                    }
                    _ => {}
                }
            }
            Event::UnitHit { unit, from, .. } => {
                if let Some((mut v, t, _)) = spawned
                    .by_id
                    .get(&unit)
                    .and_then(|e| units.get_mut(*e).ok())
                {
                    let p = Vec2::new(from.x as f32 / 1000.0, from.y as f32 / 1000.0);
                    v.recoil
                        .trigger(to_world(p.x, p.y, ground_y(p)), t.translation);
                }
            }
            _ => {}
        }
    }
}

fn special_impact(kind: Champion, (t, b): (u8, u8)) -> Option<Impact> {
    use tbg_core::UnitKind::*;
    if crate::art::kits::completion::kit(kind).is_some()
        || crate::art::kits::ogres::contains(kind)
        || crate::art::kits::abyssals::contains(kind)
        || matches!(kind, Volthar | Astreon)
    {
        return Some(Impact::Champion(kind, t, b));
    }
    Some(match kind {
        Crowcaller => Impact::Plague,
        Whisperer => Impact::Mind,
        Bulwark | Jawguard | Drillnest | Duodrill | Megadrill => Impact::Sparks,
        Capybara => Impact::Belly,
        Trident => Impact::Rend,
        SolarSentinel => Impact::Solar,
        LuminousOracle => Impact::Harmonic,
        CelestialArcher => Impact::Purify,
        Justiciar => Impact::Judgment,
        Mantis => Impact::Precise,
        Weaver => Impact::Web,
        Hornet => Impact::Venom,
        Demolisher => Impact::Piston,
        AntidoteBrewer => Impact::Cleanse,
        Puppetmaster => Impact::Puppet,
        _ => return None,
    })
}
