//! Unit entities mirrored from snapshots: spawn, interpolate, face, bob, despawn.

use std::collections::HashMap;

use bevy::camera::visibility::RenderLayers;
use bevy::prelude::*;
use tbg_moba_core::protocol::UnitView;
use tbg_moba_core::types::Side;
use tbg_moba_core::UnitKind;

use crate::app::Screen;
use crate::art::aim::AimTarget;
use crate::art::anim::{Animator, Clip};
use crate::art::melee::HitRecoil;
use crate::art::FigureLibrary;
use crate::link::Feed;
use crate::palette;
use crate::scene::fog::{FogMask, FogMaterial, FogTexture};
use crate::scene::terrain::{height, lane_distance, LANE_HALF_WIDTH};
use crate::scene::{to_world, Painted};

// This adapter provides the same visual facts expected by the BattleLine release system.
#[path = "../../../../crates/tbg-client/src/visuals/attacks.rs"]
mod attacks;
pub mod champions;
mod cues;
mod fx;
pub mod models;
mod monsters;
#[cfg(not(target_arch = "wasm32"))]
mod motion_probe;
mod status;

/// Render layer seen only by the minimap camera.
pub const MINIMAP_LAYER: usize = 2;

#[derive(Component)]
pub struct UnitVisual {
    pub id: u32,
    pub kind: UnitKind,
    pub side: Side,
    prev: Vec2,
    pub curr: Vec2,
    pub hp: f32,
    target: u32,
    activity: tbg_moba_core::state::Activity,
    pub yaw: f32,
    lunge: f32,
    still: f32,
    effect: tbg_moba_core::protocol::ChampionView,
    aim: Option<(AimTarget, Vec3)>,
    range: f32,
    hold: f32,
    splash: f32,
    secondary_shot: Option<Vec3>,
    recoil: HitRecoil,
    dying: bool,
    flash_applied: bool,
    age: f32,
    flash: f32,
    /// Height of the health bar above the feet.
    bar_height: f32,
    bar: Entity,
    fill: Entity,
}

impl UnitVisual {
    fn kind(&self) -> tbg_moba_core::Champion {
        champions::champion_of(self.kind).unwrap_or(tbg_moba_core::Champion::Shieldbearer)
    }
}

#[derive(Component)]
struct Crystal {
    base: f32,
}

/// Snapshot id → entity, plus the generation last mirrored.
#[derive(Resource, Default)]
pub struct Spawned {
    pub by_id: HashMap<u32, Entity>,
    generation: u64,
}

#[derive(Resource)]
struct Shared {
    bar_mesh: Handle<Mesh>,
    bar_back: Handle<StandardMaterial>,
    bar_fill: [Handle<StandardMaterial>; 4],
    icon: Handle<Mesh>,
    icon_material: [Handle<StandardMaterial>; 4],
    crystal: Handle<Mesh>,
    crystal_material: [Handle<FogMaterial>; 2],
    flash: Handle<FogMaterial>,
}

pub fn plugin(app: &mut App) {
    app.init_resource::<Spawned>()
        .init_resource::<models::Models>()
        .add_systems(Startup, setup.after(crate::scene::fog::setup))
        .add_systems(
            Update,
            (
                sync,
                cues::apply,
                structures,
                animate,
                champions::face,
                bars,
            )
                .chain()
                .after(crate::link::Receive)
                .before(crate::art::anim::animate)
                .run_if(in_state(Screen::Match)),
        )
        .add_systems(OnExit(Screen::Match), clear);
    fx::plugin(app);
    champions::plugin(app);
    monsters::plugin(app);
    status::plugin(app);
    #[cfg(not(target_arch = "wasm32"))]
    motion_probe::plugin(app);
}

fn unlit(materials: &mut Assets<StandardMaterial>, color: Color) -> Handle<StandardMaterial> {
    materials.add(StandardMaterial {
        base_color: color,
        unlit: true,
        ..default()
    })
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut fog_materials: ResMut<Assets<FogMaterial>>,
    fog: Res<FogTexture>,
) {
    let glow = |m: &mut Assets<FogMaterial>, c: Color, e: LinearRgba| {
        m.add(FogMaterial {
            base: StandardMaterial {
                base_color: c,
                emissive: e,
                ..default()
            },
            extension: FogMask {
                texture: fog.0.clone(),
            },
        })
    };
    let shared = Shared {
        bar_mesh: meshes.add(Rectangle::new(1.0, 1.0)),
        bar_back: unlit(&mut materials, Color::srgba(0.02, 0.02, 0.03, 0.85)),
        bar_fill: [
            palette::HP_ALLY,
            palette::HP_ENEMY,
            palette::NEUTRAL,
            palette::GOLD,
        ]
        .map(|c| unlit(&mut materials, c)),
        icon: meshes.add(
            Circle::new(1.0)
                .mesh()
                .resolution(12)
                .build()
                .rotated_by(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
        ),
        icon_material: [
            palette::BLUE_GLOW,
            palette::RED_GLOW,
            palette::NEUTRAL,
            palette::GOLD,
        ]
        .map(|c| unlit(&mut materials, c)),
        crystal: meshes.add(models::crystal()),
        crystal_material: [
            glow(
                &mut fog_materials,
                palette::BLUE_GLOW,
                LinearRgba::rgb(0.6, 2.4, 6.0),
            ),
            glow(
                &mut fog_materials,
                palette::RED_GLOW,
                LinearRgba::rgb(6.0, 1.2, 0.6),
            ),
        ],
        flash: glow(
            &mut fog_materials,
            Color::WHITE,
            LinearRgba::rgb(1.2, 0.8, 0.4),
        ),
    };
    commands.insert_resource(shared);
}

/// Ground height where a unit stands: bridges and the plaza carry it over water.
pub fn ground_y(p: Vec2) -> f32 {
    let h = height(p);
    if let Some(deck) = crate::scene::landmarks::bridge_surface(p) {
        h.max(deck)
    } else if lane_distance(p) < LANE_HALF_WIDTH {
        crate::scene::terrain::road_height(p)
    } else {
        h.max(-0.6)
    }
}

fn map_pos(u: &UnitView) -> Vec2 {
    Vec2::new(u.x as f32, u.y as f32) / 100.0
}

/// Which colour a health bar or icon takes from the viewer's seat.
fn tint(side: Side, owner: Option<tbg_moba_core::Seat>, feed: &Feed) -> usize {
    let mine = feed.seat;
    match side {
        Side::Neutral => 2,
        _ if owner.is_some() && owner == mine => 3,
        Side::Team(team) if mine.is_some_and(|s| s.team() == team) => 0,
        Side::Team(_) => 1,
    }
}

#[allow(clippy::too_many_arguments)] // one mirror pass needs the feed, assets and fx queue
fn sync(
    mut commands: Commands,
    feed: Res<Feed>,
    mut spawned: ResMut<Spawned>,
    mut models: ResMut<models::Models>,
    mut meshes: ResMut<Assets<Mesh>>,
    painted: Res<Painted>,
    shared: Res<Shared>,
    lib: Res<FigureLibrary>,
    mut visuals: Units,
    mut shots: MessageWriter<fx::Shot>,
    mut bursts: MessageWriter<fx::Burst>,
) {
    if feed.generation == spawned.generation {
        return;
    }
    if feed.generation < spawned.generation {
        // A new match started in place: forget the old one.
        for (_, e) in spawned.by_id.drain() {
            despawn_unit(&mut commands, e, &visuals);
        }
    }
    spawned.generation = feed.generation;
    let Some(world) = &feed.curr else { return };
    let positions: HashMap<u32, Vec2> = world.units.iter().map(|u| (u.id, map_pos(u))).collect();
    let previous: HashMap<u32, Vec2> = feed
        .prev
        .iter()
        .flat_map(|w| &w.units)
        .map(|u| (u.id, map_pos(u)))
        .collect();
    for u in &world.units {
        let pos = map_pos(u);
        if let Some(&entity) = spawned.by_id.get(&u.id) {
            if let Ok((mut v, animator)) = visuals.get_mut(entity) {
                let hp = u.hp_permille as f32 / 1000.0;
                if let (Some(mut a), Some(champion)) = (animator, champions::champion_of(u.kind)) {
                    // The same cues as the 1v1 battle: attack clip, flinch on a hit.
                    if u.attacked {
                        a.observe_counter(u.champion.counter);
                        a.play(champions::attack_clip(champion));
                    } else if hp < v.hp {
                        a.play_if_free(Clip::Hit);
                    }
                    if !matches!(
                        a.clip_time().0,
                        Clip::AttackMelee | Clip::AttackRanged | Clip::Cast
                    ) {
                        a.observe_counter(u.champion.counter);
                    }
                    a.set_reduction(u.champion.reduction);
                    let f = u.champion;
                    a.observe_special(
                        f.ready_in == 0,
                        (f.cast_left > 0).then_some(f.cast_left),
                        f.cast_windup,
                        f.rooted,
                    );
                    if u.attacked {
                        v.aim = positions.get(&u.target).map(|p| {
                            let target = if world
                                .units
                                .iter()
                                .any(|w| w.id == u.target && w.kind.structure())
                            {
                                AimTarget::Stronghold(tbg_core::Seat::B)
                            } else {
                                AimTarget::Unit(u.target)
                            };
                            (
                                target,
                                to_world(p.x, p.y, ground_y(*p) + 0.6 * champions::scale(u.kind)),
                            )
                        });
                    }
                }
                v.effect = u.champion;
                v.prev = previous.get(&u.id).copied().unwrap_or(pos);
                v.curr = pos;
                if hp != v.hp {
                    v.flash = if hp < v.hp { 0.08 } else { -0.15 };
                }
                v.hp = hp;
                v.target = u.target;
                v.activity = u.activity;
                if u.attacked {
                    v.lunge = 1.0;
                }
            }
        } else {
            let entity = spawn_unit(
                &mut commands,
                u,
                pos,
                &feed,
                &mut models,
                &mut meshes,
                &painted,
                &shared,
                &lib,
            );
            spawned.by_id.insert(u.id, entity);
        }
        // Champions fire their own kit shots from the animation (champions.rs).
        if u.attacked && champions::champion_of(u.kind).is_none() {
            if let Some(&to) = positions.get(&u.target) {
                shots.write(fx::Shot {
                    target: u.target,
                    from: pos,
                    to,
                    kind: u.kind,
                    side: u.side,
                });
            }
        }
    }
    let gone: Vec<u32> = spawned
        .by_id
        .keys()
        .copied()
        .filter(|id| !positions.contains_key(id))
        .collect();
    for id in gone {
        let Some(entity) = spawned.by_id.remove(&id) else {
            continue;
        };
        if let Ok((v, animator)) = visuals.get_mut(entity) {
            if !feed
                .motion_events
                .iter()
                .any(|e| matches!(e, tbg_moba_core::state::Event::UnitDied{unit,..} if *unit==id))
            {
                despawn_unit(&mut commands, entity, &visuals);
                continue;
            }
            if let Some(mut a) = animator {
                // Champions play their 1v1 Death clip before leaving the field.
                commands.entity(v.bar).despawn();
                commands.entity(v.fill).despawn();
                commands.entity(entity).remove::<UnitVisual>();
                champions::die(&mut commands, entity, &mut a);
                continue;
            }
            if matches!(v.kind, UnitKind::Neutral(_)) {
                commands.entity(v.bar).despawn();
                commands.entity(v.fill).despawn();
                commands
                    .entity(entity)
                    .remove::<UnitVisual>()
                    .despawn_related::<Children>();
                monsters::die(&mut commands, entity);
                continue;
            }
            bursts.write(fx::Burst {
                at: v.curr,
                kind: v.kind,
                side: v.side,
            });
        }
        despawn_unit(&mut commands, entity, &visuals);
    }
}

#[allow(clippy::too_many_arguments)] // assembling one figure touches every shared asset
fn spawn_unit(
    commands: &mut Commands,
    u: &UnitView,
    pos: Vec2,
    feed: &Feed,
    models: &mut models::Models,
    meshes: &mut Assets<Mesh>,
    painted: &Painted,
    shared: &Shared,
    lib: &FigureLibrary,
) -> Entity {
    let tint = tint(u.side, u.owner, feed);
    let width = models::bar_width(u.kind);
    let fill = commands
        .spawn((
            Mesh3d(shared.bar_mesh.clone()),
            MeshMaterial3d(shared.bar_fill[tint].clone()),
            Transform::default(),
        ))
        .id();
    let bar = commands
        .spawn((
            Mesh3d(shared.bar_mesh.clone()),
            MeshMaterial3d(shared.bar_back.clone()),
            Transform::default().with_scale(Vec3::new(width + 0.12, 0.3, 1.0)),
            Visibility::Visible,
        ))
        .id();
    let stats = tbg_moba_core::content::base_stats(u.kind);
    let yaw = u.side.team().map_or(0.0, |team| {
        let nexus = tbg_moba_core::map::stronghold(team);
        let offset = pos - Vec2::new(nexus.x as f32, nexus.y as f32) / 1000.0;
        let exit_radius = (tbg_moba_core::map::NEXUS_PLINTH_RADIUS
            + stats.radius
            + tbg_moba_core::map::LANE_HALF_WIDTH) as f32
            / 1000.0;
        if offset.length_squared() <= exit_radius.powi(2) && offset != Vec2::ZERO {
            offset.x.atan2(-offset.y)
        } else if team == tbg_moba_core::Team::Blue {
            std::f32::consts::PI * 0.75
        } else {
            -std::f32::consts::PI * 0.25
        }
    });
    let champion = champions::champion_of(u.kind);
    let scale = champion.map_or(models::scale(u.kind), |_| champions::scale(u.kind));
    let bar_height = champion.map_or(models::bar_height(u.kind), |k| {
        lib.get(k, 0, 0).overhead * scale + 0.5
    });
    let hold = champion
        .and_then(|k| tbg_core::content::def(k).on_hit)
        .map_or(0.0, |hit| match hit {
            tbg_core::OnHit::Root { ticks } => f32::from(ticks) / tbg_core::content::TICK_HZ as f32,
            _ => 0.0,
        });
    let id = commands
        .spawn((
            Transform::from_translation(to_world(pos.x, pos.y, ground_y(pos)))
                .with_scale(Vec3::splat(scale)),
            Visibility::default(),
            UnitVisual {
                id: u.id,
                kind: u.kind,
                side: u.side,
                prev: pos,
                curr: pos,
                hp: u.hp_permille as f32 / 1000.0,
                target: u.target,
                activity: u.activity,
                yaw,
                lunge: 0.0,
                still: 0.15,
                effect: u.champion,
                aim: None,
                range: stats.range as f32 / 1000.0,
                hold,
                splash: stats.splash as f32 / 1000.0,
                secondary_shot: None,
                recoil: HitRecoil::default(),
                dying: false,
                flash_applied: false,
                age: 0.0,
                flash: 0.0,
                bar_height,
                bar,
                fill,
            },
        ))
        .id();
    match champion {
        Some(kind) => champions::spawn_figure(commands, lib, id, kind, u.id),
        None => {
            commands.entity(id).insert((
                Mesh3d(models.get(u.kind, u.side, 0, meshes)),
                MeshMaterial3d(painted.0.clone()),
            ));
        }
    }
    let mut entity = commands.entity(id);
    let icon_size = match u.kind {
        UnitKind::Stronghold => 5.0,
        UnitKind::Tower(..) => 3.0,
        _ => 1.3,
    } / scale;
    entity.with_children(|c| {
        c.spawn((
            Mesh3d(shared.icon.clone()),
            MeshMaterial3d(shared.icon_material[tint].clone()),
            Transform::from_xyz(0.0, 30.0, 0.0).with_scale(Vec3::splat(icon_size)),
            RenderLayers::layer(MINIMAP_LAYER),
        ));
        if let (Some((y, s)), Side::Team(team)) = (models::crystal_offset(u.kind), u.side) {
            c.spawn((
                Mesh3d(shared.crystal.clone()),
                MeshMaterial3d(shared.crystal_material[team.index()].clone()),
                Transform::from_xyz(0.0, y, 0.0).with_scale(Vec3::splat(s)),
                Crystal { base: y },
            ));
        }
    });
    entity.id()
}

fn animate(
    mut commands: Commands,
    lib: Res<FigureLibrary>,
    vfx: Res<crate::vfx::VfxAssets>,
    feed: Res<Feed>,
    time: Res<Time>,
    shared: Res<Shared>,
    painted: Res<Painted>,
    mut units: Query<
        (
            &mut UnitVisual,
            &mut Transform,
            Option<&mut Animator>,
            Option<&mut MeshMaterial3d<FogMaterial>>,
        ),
        Without<Crystal>,
    >,
    mut crystals: Query<(&Crystal, &mut Transform), Without<UnitVisual>>,
) {
    let alpha = feed.alpha;
    let dt = time.delta_secs();
    let t = time.elapsed_secs();
    let targets: HashMap<u32, (Vec2, Side)> = units
        .iter()
        .map(|(v, ..)| (v.id, (v.prev.lerp(v.curr, alpha), v.side)))
        .collect();
    for (mut v, mut transform, animator, material) in &mut units {
        if v.kind.structure() {
            continue;
        }
        let pos = v.prev.lerp(v.curr, alpha);
        let step = v.curr - v.prev;
        // A committed target sets facing even when collision adds a sideways
        // step. Interpolate both endpoints on the same clock.
        let facing = targets
            .get(&v.target)
            .map(|(at, _)| *at - pos)
            .filter(|delta| delta.length_squared() > 1e-4)
            .or_else(|| (step.length_squared() > 1e-4).then_some(step));
        if let Some(d) = facing {
            // The shared rigs face +Z; the map's north is -Z.
            let goal = d.x.atan2(-d.y);
            v.yaw = crate::art::aim::turn_toward(
                v.yaw,
                goal,
                dt * crate::art::aim::MAX_SLEW_RATE,
                crate::art::aim::DEAD_ZONE,
            );
        }
        v.lunge = (v.lunge - dt * 5.0).max(0.0);
        let walking = step.length_squared() > 1e-4;
        let bob = if walking {
            (t * 11.0 + v.id as f32).sin().abs() * 0.09
        } else {
            0.0
        };
        let forward = Vec3::new(v.yaw.sin(), 0.0, v.yaw.cos()) * (v.lunge * (1.0 - v.lunge) * 1.2);
        if let Some(mut a) = animator {
            let chest = 0.6 * champions::scale(v.kind);
            let aim = targets.get(&v.target);
            a.aim_at = aim
                .map(|(p, _)| to_world(p.x, p.y, ground_y(*p) + chest))
                .or_else(|| {
                    matches!(
                        a.clip_time().0,
                        Clip::AttackMelee | Clip::AttackRanged | Clip::Cast
                    )
                    .then(|| v.aim.map(|(_, p)| p))
                    .flatten()
                });
            a.aim_is_ally = aim.is_some_and(|(_, side)| *side == v.side);
            let mut at = to_world(pos.x, pos.y, ground_y(pos));
            if let Some((mount, _)) = targets.get(&v.effect.carrier) {
                let hip = crate::art::rig::rest(
                    crate::art::rig::Node::Body(crate::art::rig::Joint::Hips),
                    &a.plan,
                )
                .y * a.plan.scale;
                at = to_world(
                    mount.x,
                    mount.y,
                    ground_y(*mount)
                        + crate::art::kits::saddle_height(0) * champions::scale(v.kind)
                        - hip,
                );
            }
            v.still = if transform.translation.xz().distance_squared(at.xz()) > 0.0004_f32.powi(2) {
                0.0
            } else {
                v.still + dt
            };
            a.set_base(
                if v.still < 0.15
                    && v.effect.carrier == 0
                    && matches!(
                        v.activity,
                        tbg_moba_core::state::Activity::Advance
                            | tbg_moba_core::state::Activity::Pursue
                    )
                {
                    Clip::Walk
                } else {
                    Clip::Idle
                },
            );
            transform.translation = at;
            let flash = v.flash;
            if flash != 0.0 && !v.flash_applied {
                let mat = vfx.champion_tint(flash > 0.0);
                for j in a.joints {
                    commands.entity(j).try_insert(MeshMaterial3d(mat.clone()));
                }
                v.flash_applied = true;
            }
            v.flash = flash.signum() * (flash.abs() - dt).max(0.0);
            if v.flash == 0.0 && v.flash_applied {
                for j in a.joints {
                    commands
                        .entity(j)
                        .try_insert(MeshMaterial3d(lib.material.clone()));
                }
                v.flash_applied = false;
            }
            continue;
        }
        transform.translation = to_world(pos.x, pos.y, ground_y(pos) + bob) + forward;
        transform.rotation = Quat::from_rotation_y(v.yaw + std::f32::consts::PI);
        if let UnitKind::Neutral(kind) = v.kind {
            v.age += dt;
            v.flash = (v.flash - dt).max(0.0);
            monsters::pose(kind, v.id, v.age, t, v.lunge, &mut transform);
            if let Some(mut material) = material {
                material.0 = if v.flash > 0.0 {
                    shared.flash.clone()
                } else {
                    painted.0.clone()
                };
            }
        }
    }
    for (c, mut transform) in &mut crystals {
        transform.translation.y = c.base + (t * 1.6).sin() * 0.25;
        transform.rotation = Quat::from_rotation_y(t * 0.8);
    }
}

/// Health bars float above units and always face the camera.
fn bars(
    camera: Query<&GlobalTransform, With<crate::camera::MainCamera>>,
    units: Query<(&UnitVisual, &Transform)>,
    mut parts: Query<(&mut Transform, &mut Visibility), Without<UnitVisual>>,
) {
    let Ok(camera) = camera.single() else { return };
    let rotation = camera.compute_transform().rotation;
    for (v, transform) in &units {
        let width = models::bar_width(v.kind);
        let always = v.kind.structure();
        let shown = always || v.hp < 0.999;
        let top = transform
            .translation
            .with_y(transform.translation.y.max(0.0) + v.bar_height);
        if let Ok((mut bar, mut vis)) = parts.get_mut(v.bar) {
            bar.translation = top;
            bar.rotation = rotation;
            *vis = if shown {
                Visibility::Visible
            } else {
                Visibility::Hidden
            };
        }
        if let Ok((mut fill, mut vis)) = parts.get_mut(v.fill) {
            let w = width * v.hp.clamp(0.0, 1.0);
            fill.rotation = rotation;
            fill.translation = top + rotation * Vec3::new((w - width) / 2.0, 0.0, 0.01);
            fill.scale = Vec3::new(w.max(0.001), 0.22, 1.0);
            *vis = if shown {
                Visibility::Visible
            } else {
                Visibility::Hidden
            };
        }
    }
}

/// Removes a figure with its free-standing health bar parts.
/// The mirror's unit query: every figure, with its rig when it is a champion.
type Units<'w, 's> = Query<'w, 's, (&'static mut UnitVisual, Option<&'static mut Animator>)>;

fn despawn_unit(commands: &mut Commands, entity: Entity, units: &Units) {
    if let Ok((v, _)) = units.get(entity) {
        commands.entity(v.bar).despawn();
        commands.entity(v.fill).despawn();
    }
    commands.entity(entity).despawn();
}

fn clear(
    mut commands: Commands,
    mut spawned: ResMut<Spawned>,
    units: Units,
    dying: Query<Entity, With<champions::Dying>>,
) {
    for entity in &dying {
        commands.entity(entity).despawn();
    }
    for (_, entity) in spawned.by_id.drain() {
        despawn_unit(&mut commands, entity, &units);
    }
    spawned.generation = 0;
}

/// Damage bands use the Team's last-observed HP, so hidden damage never changes a silhouette.
fn structures(
    mut models: ResMut<models::Models>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut q: Query<(&UnitVisual, &mut Mesh3d)>,
) {
    for (v, mut mesh) in &mut q {
        if v.kind.structure() {
            let damage = if v.hp < 0.33 {
                2
            } else if v.hp < 0.66 {
                1
            } else {
                0
            };
            let handle = models.get(v.kind, v.side, damage, &mut meshes);
            if mesh.0 != handle {
                mesh.0 = handle;
            }
        }
    }
}
