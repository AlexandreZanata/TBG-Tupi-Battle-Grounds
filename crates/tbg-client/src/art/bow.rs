//! Shared arrow geometry and bow-string presentation, attached to the OffhandL socket.

use super::{
    anim::{Animator, Clip},
    mesh::{self, Shape::*},
    palette::*,
    rig::{Node, Socket},
    FigureLibrary,
};
use bevy::prelude::*;

pub struct BowAssets {
    pub arrow: [Handle<Mesh>; 2],
    pub triangles: usize,
    thread: Handle<Mesh>,
}
impl BowAssets {
    pub fn new(meshes: &mut Assets<Mesh>) -> Self {
        let arrow = [ALLY, ENEMY].map(|team| {
            let parts = [
                (
                    Box(0.008, 0.008, 0.55),
                    TIMBER,
                    Vec3::new(0.0, 0.0, 0.27),
                    Vec3::ZERO,
                ),
                (
                    Cone(0.022, 0.07),
                    STEEL,
                    Vec3::new(0.0, 0.0, 0.58),
                    Vec3::new(90.0, 0.0, 0.0),
                ),
                (
                    Box(0.004, 0.055, 0.075),
                    team,
                    Vec3::new(0.0, 0.0, 0.035),
                    Vec3::ZERO,
                ),
                (
                    Box(0.055, 0.004, 0.075),
                    team,
                    Vec3::new(0.0, 0.0, 0.035),
                    Vec3::ZERO,
                ),
            ]
            .into_iter()
            .map(|(s, c, p, r)| {
                (
                    mesh::paint(mesh::build(s), c),
                    Transform::from_translation(p).with_rotation(Quat::from_euler(
                        EulerRot::XYZ,
                        r.x.to_radians(),
                        r.y.to_radians(),
                        r.z.to_radians(),
                    )),
                )
            })
            .collect();
            meshes.add(mesh::merge(parts).unwrap_or_else(|| mesh::build(Box(0.008, 0.008, 0.55))))
        });
        let triangles = meshes.get(&arrow[0]).map_or(0, mesh::triangles) + 24;
        Self {
            triangles,
            arrow,
            thread: meshes.add(mesh::paint(mesh::build(Box(0.004, 1.0, 0.004)), LINEN)),
        }
    }
}
#[derive(Component)]
pub struct BowRig {
    pub socket: Entity,
    strings: [Entity; 2],
    arrow: Entity,
    pub own: bool,
    /// Half the string length of this form's bow.
    half: f32,
}
pub fn attach(
    commands: &mut Commands,
    root: Entity,
    animator: &Animator,
    lib: &FigureLibrary,
    material: &Handle<StandardMaterial>,
    own: bool,
    half: f32,
) {
    let socket = animator.joints[Node::Prop(Socket::OffhandL).index()];
    let arrow = commands
        .spawn((
            Mesh3d(lib.bow.arrow[usize::from(!own)].clone()),
            MeshMaterial3d(material.clone()),
            Transform::default(),
            Visibility::Hidden,
        ))
        .id();
    let strings = [0, 1].map(|_| {
        commands
            .spawn((
                Mesh3d(lib.bow.thread.clone()),
                MeshMaterial3d(material.clone()),
                Transform::default(),
            ))
            .id()
    });
    commands
        .entity(socket)
        .add_children(&[arrow, strings[0], strings[1]]);
    commands.entity(root).insert(BowRig {
        socket,
        strings,
        arrow,
        own,
        half,
    });
}
pub fn strings(
    figures: Query<(&Animator, &BowRig)>,
    mut transforms: Query<&mut Transform>,
    mut visibility: Query<&mut Visibility>,
) {
    for (a, bow) in &figures {
        let (clip, t) = a.clip_time();
        let middle = Vec3::new(0.0, 0.08, -0.23 - tension(clip, t) * super::archer_v2::DRAW);
        present_strings(bow, a.archer, middle, &mut transforms, &mut visibility);
        if let Ok(mut tr) = transforms.get_mut(bow.arrow) {
            tr.translation = middle;
        }
        if let Ok(mut v) = visibility.get_mut(bow.arrow) {
            let nocked = a.archer && clip == Clip::AttackRanged && t < 0.18;
            *v = if nocked {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            };
        }
    }
}
fn tension(clip: Clip, t: f32) -> f32 {
    if clip != Clip::AttackRanged {
        0.0
    } else if t < 0.12 {
        (t / 0.12).min(1.0)
    } else if t < 0.18 {
        1.0
    } else {
        (1.0 - (t - 0.18) / 0.04).clamp(0.0, 1.0)
    }
}
fn present_strings(
    bow: &BowRig,
    active: bool,
    middle: Vec3,
    transforms: &mut Query<&mut Transform>,
    visibility: &mut Query<&mut Visibility>,
) {
    for (i, &entity) in bow.strings.iter().enumerate() {
        let tip = Vec3::new(0.0, 0.08 + if i == 0 { bow.half } else { -bow.half }, -0.23);
        if let Ok(mut v) = visibility.get_mut(entity) {
            *v = if active {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            };
        }
        if let Ok(mut tr) = transforms.get_mut(entity) {
            *tr = Transform::from_translation((middle + tip) * 0.5)
                .with_rotation(Quat::from_rotation_arc(Vec3::Y, (tip - middle).normalize()))
                .with_scale(Vec3::new(1.0, (tip - middle).length(), 1.0));
        }
    }
}
