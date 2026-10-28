//! Browser review tools: roster, palette, blockouts, named units, and contact sheets.

mod blockout;
mod camera;
mod figures;
mod layout;
mod lineup;
mod palette;
mod settings;
mod title;

use crate::{
    art::{
        anim::{Animator, Clip, CLIPS},
        FigureLibrary,
    },
    config, GameState,
};
use bevy::prelude::*;
use settings::Gallery;

#[derive(Component)]
struct Cycle {
    offset: usize,
    shown: Option<usize>,
}

pub fn plugin(app: &mut App) {
    if config::url_param("gallery").is_none() {
        return;
    }
    app.insert_resource(Gallery::read())
        .add_systems(Startup, |mut next: ResMut<NextState<GameState>>| {
            next.set(GameState::Gallery)
        })
        .add_systems(OnEnter(GameState::Gallery), spawn)
        .add_systems(
            Update,
            (
                cycle.before(crate::art::anim::animate),
                camera::frame,
                layout::place.before(crate::art::anim::animate),
                turn.after(crate::art::anim::animate),
            )
                .run_if(in_state(GameState::Gallery)),
        );
}
fn spawn(
    mut commands: Commands,
    lib: Res<FigureLibrary>,
    options: Res<Gallery>,
    mut meshes: ResMut<Assets<Mesh>>,
    mats: ResMut<Assets<StandardMaterial>>,
    arena: Query<Entity, With<Mesh3d>>,
    window: Single<&Window>,
) {
    if options.stress {
        return;
    }
    if options.hide_arena() {
        for e in &arena {
            commands.entity(e).insert(Visibility::Hidden);
        }
    }
    if options.floor() {
        use crate::art::{mesh, palette};
        commands.spawn((
            Mesh3d(meshes.add(mesh::paint(
                mesh::build(mesh::Shape::Box(60.0, 0.05, 60.0)),
                palette::PARCHMENT,
            ))),
            MeshMaterial3d(lib.material.clone()),
            Transform::from_xyz(8.0, -0.08, -5.0),
        ));
    }
    match options.mode.as_str() {
        "palette" => palette::spawn(&mut commands, meshes, mats),
        "blockout" => blockout::spawn(&mut commands, &lib, mats, window.height() > window.width()),
        _ => figures::spawn(&mut commands, &lib, &options, mats, &window),
    }
}
fn cycle(time: Res<Time>, options: Res<Gallery>, mut figures: Query<(&mut Animator, &mut Cycle)>) {
    let step = (time.elapsed_secs_f64() * f64::from(options.slow) / 1.4) as usize;
    for (mut anim, mut c) in &mut figures {
        let i = options
            .clip
            .map_or((step + c.offset) % CLIPS.len(), |clip| {
                CLIPS.iter().position(|c| *c == clip).unwrap_or(0)
            });
        if c.shown == Some(step) {
            continue;
        }
        c.shown = Some(step);
        match CLIPS[i] {
            base @ (Clip::Idle | Clip::Walk) => anim.set_base(base),
            clip => anim.play(clip),
        }
    }
}
fn turn(
    time: Res<Time>,
    options: Res<Gallery>,
    figures: Query<&Animator>,
    mut transforms: Query<&mut Transform, Without<Animator>>,
) {
    if !options.turn {
        return;
    }
    for figure in &figures {
        if let Ok(mut t) = transforms.get_mut(figure.figure) {
            t.rotation = Quat::from_rotation_y(time.elapsed_secs() * options.slow * 0.6);
        }
    }
}
