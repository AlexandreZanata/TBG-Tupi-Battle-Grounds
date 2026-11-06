//! Jungle monster silhouettes and shared desktop idle, attack, spawn and death motion.

use crate::app::Screen;
use crate::art::mesh::{self, Shape};
use bevy::prelude::*;
use tbg_moba_core::types::NeutralKind;

type Part = (Shape, Color, Transform);
const DARK: Color = Color::srgb(0.12, 0.10, 0.08);
const BONE: Color = Color::srgb(0.88, 0.82, 0.63);
fn at(x: f32, y: f32, z: f32) -> Transform {
    Transform::from_xyz(x, y, z)
}

fn cat(p: &mut Vec<Part>, wolf: bool) {
    let fur = if wolf {
        Color::srgb(0.66, 0.29, 0.12)
    } else {
        Color::srgb(0.83, 0.57, 0.17)
    };
    p.push((Shape::Bevel(0.9, 0.8, 1.7, 0.18), fur, at(0., 1., 0.)));
    p.push((Shape::Bevel(0.7, 0.6, 0.75, 0.12), fur, at(0., 1.25, -0.95)));
    p.push((
        Shape::Bevel(0.4, 0.25, 0.45, 0.07),
        DARK,
        at(0., 1.05, -1.35),
    ));
    for x in [-0.33, 0.33] {
        for z in [-0.55, 0.55] {
            p.push((Shape::Bevel(0.22, 0.85, 0.25, 0.05), DARK, at(x, 0.45, z)));
        }
        p.push((
            Shape::Cone(0.19, if wolf { 0.6 } else { 0.25 }),
            fur,
            at(x * 0.75, 1.72, -0.85),
        ));
        p.push((
            Shape::Ball(0.07),
            Color::srgb(0.9, 0.9, 0.35),
            at(x * 0.6, 1.35, -1.33),
        ));
        p.push((Shape::Cone(0.045, 0.2), BONE, at(x * 0.45, 0.96, -1.5)));
    }
    p.push((
        Shape::Pill(0.12, 0.8),
        fur,
        at(0., 1., 1.1).with_rotation(Quat::from_rotation_x(0.9)),
    ));
    if wolf {
        p.push((Shape::Canopy(0.36, 0.75, 71), DARK, at(0., 1.6, -0.35)));
    } else {
        for z in [-0.5, 0., 0.5] {
            for x in [-0.46, 0.46] {
                p.push((Shape::Ball(0.14), DARK, at(x, 1.16, z)));
            }
        }
    }
}
fn golem(p: &mut Vec<Part>) {
    let stone = Color::srgb(0.44, 0.46, 0.41);
    let moss = Color::srgb(0.23, 0.40, 0.16);
    p.push((Shape::Rock(0.95, 31), stone, at(0., 1.5, 0.)));
    p.push((Shape::Rock(0.54, 32), stone, at(0., 2.3, -0.1)));
    for x in [-1., 1.] {
        p.push((Shape::Rock(0.48, 33), stone, at(x, 1.3, 0.)));
        p.push((
            Shape::Bevel(0.48, 0.8, 0.58, 0.12),
            stone,
            at(x * 0.45, 0.4, 0.),
        ));
        p.push((Shape::Canopy(0.37, 0.16, 35), moss, at(x * 0.8, 1.82, 0.)));
        p.push((
            Shape::Cone(0.15, 0.65),
            Color::srgb(0.30, 0.82, 0.73),
            at(x * 0.6, 2.2, 0.35),
        ));
        p.push((
            Shape::Box(0.12, 0.1, 0.08),
            Color::srgb(0.9, 0.64, 0.2),
            at(x * 0.2, 2.36, -0.61),
        ));
    }
}
fn frog(p: &mut Vec<Part>) {
    let skin = Color::srgb(0.39, 0.47, 0.16);
    p.push((
        Shape::Ball(0.85),
        skin,
        at(0., 0.65, 0.).with_scale(Vec3::new(1., 0.65, 1.2)),
    ));
    p.push((
        Shape::Bevel(1.25, 0.45, 0.75, 0.16),
        skin,
        at(0., 0.6, -0.65),
    ));
    p.push((Shape::Box(0.85, 0.07, 0.08), DARK, at(0., 0.48, -1.05)));
    for x in [-0.55, 0.55] {
        p.push((Shape::Ball(0.24), skin, at(x, 0.92, -0.65)));
        p.push((Shape::Ball(0.12), BONE, at(x, 0.96, -0.84)));
        p.push((Shape::Ball(0.07), DARK, at(x, 0.96, -0.93)));
        p.push((
            Shape::Bevel(0.5, 0.35, 0.7, 0.1),
            skin,
            at(x * 1.3, 0.3, 0.4),
        ));
        p.push((
            Shape::Bevel(0.4, 0.1, 0.5, 0.04),
            skin,
            at(x * 1.4, 0.12, -0.65),
        ));
    }
    for z in [0., 0.4, 0.8] {
        p.push((
            Shape::Rock(0.13, 43),
            Color::srgb(0.27, 0.33, 0.1),
            at(0., 1.12, z),
        ));
    }
}
pub fn figure(kind: NeutralKind) -> Mesh {
    use NeutralKind::*;
    let mut p = Vec::new();
    match kind {
        WolfAlpha | WolfPup => cat(&mut p, true),
        Jaguar => cat(&mut p, false),
        Big | Golem | SmallGolem => golem(&mut p),
        Small | Frog | SmallFrog => frog(&mut p),
    }
    let small = matches!(kind, WolfPup | SmallGolem | SmallFrog | Small);
    let size = if small { 0.62 } else { 1. };
    mesh::merge(
        p.into_iter()
            .map(|(s, c, t)| {
                (
                    mesh::paint(mesh::build(s), c),
                    Transform::from_scale(Vec3::splat(size)).mul_transform(t),
                )
            })
            .collect(),
    )
    .unwrap_or_else(|| Mesh::from(Cuboid::new(0.1, 0.1, 0.1)))
}

pub fn pose(kind: NeutralKind, id: u32, age: f32, time: f32, attack: f32, t: &mut Transform) {
    let pace = match kind {
        NeutralKind::Golem | NeutralKind::SmallGolem => 1.8,
        NeutralKind::Jaguar => 3.5,
        _ => 4.5,
    };
    let breath = (time * pace + id as f32).sin() * 0.025;
    let scale = super::models::scale(tbg_moba_core::UnitKind::Neutral(kind));
    t.scale = Vec3::new(1. + breath, 1. - breath, 1. + breath) * scale;
    t.translation.y -= 2. * (1. - (age / 0.55).clamp(0., 1.));
    t.rotation *= Quat::from_rotation_x(-attack * (1. - attack) * 0.9);
}
#[derive(Component)]
struct Collapse {
    age: f32,
    scale: Option<Vec3>,
}
pub fn die(commands: &mut Commands, entity: Entity) {
    commands.entity(entity).insert(Collapse {
        age: 0.,
        scale: None,
    });
}
pub fn plugin(app: &mut App) {
    app.add_systems(Update, collapse)
        .add_systems(OnExit(Screen::Match), clear);
}
fn collapse(
    mut commands: Commands,
    time: Res<Time>,
    mut q: Query<(Entity, &mut Collapse, &mut Transform)>,
) {
    for (e, mut c, mut t) in &mut q {
        c.age += time.delta_secs();
        if c.age > 0.65 {
            commands.entity(e).despawn();
            continue;
        }
        let scale = *c.scale.get_or_insert(t.scale);
        let k = (c.age / 0.65).clamp(0., 1.);
        t.scale = scale * Vec3::new(1. - k * 0.25, 1. - k * 0.95, 1. - k * 0.25);
        t.rotation = Quat::from_rotation_z(k * 0.8);
    }
}
fn clear(mut commands: Commands, q: Query<Entity, With<Collapse>>) {
    for e in &q {
        commands.entity(e).despawn();
    }
}
