//! Unit entities mirrored from snapshots: spawn, interpolate, face, bob, despawn.

use std::collections::HashMap;

use bevy::camera::visibility::RenderLayers;
use bevy::prelude::*;
use tbg_moba_core::protocol::UnitView;
use tbg_moba_core::types::Side;
use tbg_moba_core::UnitKind;

use crate::app::Screen;
use crate::art::anim::{Animator, Clip};
use crate::art::FigureLibrary;
use crate::link::Feed;
use crate::palette;
use crate::scene::terrain::{height, lane_distance, LANE_HALF_WIDTH};
use crate::scene::{to_world, Painted};

pub mod champions;
mod fx;
pub use fx::Marker;
pub mod models;

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
    pub yaw: f32,
    lunge: f32,
    /// Height of the health bar above the feet.
    bar_height: f32,
    bar: Entity,
    fill: Entity,
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
    crystal_material: [Handle<StandardMaterial>; 2],
    ring: Handle<Mesh>,
}

pub fn plugin(app: &mut App) {
    app.init_resource::<Spawned>()
        .init_resource::<models::Models>()
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (sync, animate, bars)
                .chain()
                .run_if(in_state(Screen::Match)),
        )
        .add_systems(OnExit(Screen::Match), clear);
    fx::plugin(app);
    champions::plugin(app);
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
) {
    let glow = |m: &mut Assets<StandardMaterial>, c: Color, e: LinearRgba| {
        m.add(StandardMaterial {
            base_color: c,
            emissive: e,
            ..default()
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
                &mut materials,
                palette::BLUE_GLOW,
                LinearRgba::rgb(0.6, 2.4, 6.0),
            ),
            glow(
                &mut materials,
                palette::RED_GLOW,
                LinearRgba::rgb(6.0, 1.2, 0.6),
            ),
        ],
        ring: meshes.add(
            Annulus::new(1.0, 1.25)
                .mesh()
                .resolution(24)
                .build()
                .rotated_by(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
        ),
    };
    commands.insert_resource(shared);
}

/// Ground height where a unit stands: bridges and the plaza carry it over water.
pub fn ground_y(p: Vec2) -> f32 {
    let h = height(p);
    if h < 0.0 && lane_distance(p) < LANE_HALF_WIDTH + 0.5 {
        0.45
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
    for u in &world.units {
        let pos = map_pos(u);
        if let Some(&entity) = spawned.by_id.get(&u.id) {
            if let Ok((mut v, animator)) = visuals.get_mut(entity) {
                let hp = u.hp_permille as f32 / 1000.0;
                if let (Some(mut a), Some(champion)) = (animator, champions::champion_of(u.kind)) {
                    // The same cues as the 1v1 battle: attack clip, flinch on a hit.
                    if u.attacked {
                        a.play(champions::attack_clip(champion));
                    } else if hp < v.hp {
                        a.play_if_free(Clip::Hit);
                    }
                }
                v.prev = v.curr;
                v.curr = pos;
                v.hp = hp;
                v.target = u.target;
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
            if let Some(mut a) = animator {
                // Champions play their 1v1 Death clip before leaving the field.
                commands.entity(v.bar).despawn();
                commands.entity(v.fill).despawn();
                commands.entity(entity).remove::<UnitVisual>();
                champions::die(&mut commands, entity, &mut a);
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
    let yaw = match u.side {
        Side::Team(tbg_moba_core::Team::Red) => std::f32::consts::PI * 0.75,
        _ => -std::f32::consts::PI * 0.25,
    };
    let champion = champions::champion_of(u.kind);
    let scale = champion.map_or(models::scale(u.kind), |_| champions::scale(u.kind));
    let bar_height = champion.map_or(models::bar_height(u.kind), |k| {
        lib.get(k, 0, 0).overhead * scale + 0.5
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
                yaw,
                lunge: 0.0,
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
                Mesh3d(models.get(u.kind, u.side, meshes)),
                MeshMaterial3d(painted.0.clone()),
            ));
        }
    }
    let mut entity = commands.entity(id);
    let icon_size = match u.kind {
        UnitKind::Stronghold => 5.0,
        UnitKind::Tower(..) => 3.0,
        UnitKind::Commander(_) => 2.8,
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
        if tint == 3 && matches!(u.kind, UnitKind::Commander(_)) {
            c.spawn((
                Mesh3d(shared.ring.clone()),
                MeshMaterial3d(shared.bar_fill[3].clone()),
                // The ring keeps the same ground size whatever the rig scale.
                Transform::from_xyz(0.0, 0.08, 0.0).with_scale(Vec3::splat(1.3 / scale)),
            ));
        }
    });
    entity.id()
}

fn animate(
    feed: Res<Feed>,
    time: Res<Time>,
    mut units: Query<(&mut UnitVisual, &mut Transform, Option<&mut Animator>), Without<Crystal>>,
    mut crystals: Query<(&Crystal, &mut Transform), Without<UnitVisual>>,
) {
    let alpha = feed.alpha(time.elapsed_secs_f64());
    let dt = time.delta_secs();
    let t = time.elapsed_secs();
    let targets: HashMap<u32, Vec2> = units.iter().map(|(v, ..)| (v.id, v.curr)).collect();
    for (mut v, mut transform, animator) in &mut units {
        if v.kind.structure() {
            continue;
        }
        let pos = v.prev.lerp(v.curr, alpha);
        let step = v.curr - v.prev;
        let facing = if step.length_squared() > 1e-4 {
            Some(step)
        } else {
            targets
                .get(&v.target)
                .map(|t| *t - v.curr)
                .filter(|d| d.length_squared() > 1e-4)
        };
        if let Some(d) = facing {
            // Map (x, y-north) → Bevy (x, -z); figures face -Z.
            let goal = (-d.x).atan2(d.y);
            let diff = (goal - v.yaw + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU)
                - std::f32::consts::PI;
            v.yaw += diff * (dt * 12.0).min(1.0);
        }
        v.lunge = (v.lunge - dt * 5.0).max(0.0);
        let walking = step.length_squared() > 1e-4;
        let bob = if walking {
            (t * 11.0 + v.id as f32).sin().abs() * 0.09
        } else {
            0.0
        };
        let forward =
            Vec3::new(-v.yaw.sin(), 0.0, -v.yaw.cos()) * (v.lunge * (1.0 - v.lunge) * 1.2);
        if let Some(mut a) = animator {
            // The 1v1 rig walks, faces and aims by itself (champions.rs).
            a.aim_at = targets
                .get(&v.target)
                .map(|t| to_world(t.x, t.y, ground_y(*t) + 1.0));
            transform.translation = to_world(pos.x, pos.y, ground_y(pos));
            continue;
        }
        transform.translation = to_world(pos.x, pos.y, ground_y(pos) + bob) + forward;
        transform.rotation = Quat::from_rotation_y(v.yaw);
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
        let always = v.kind.structure() || matches!(v.kind, UnitKind::Commander(_));
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
