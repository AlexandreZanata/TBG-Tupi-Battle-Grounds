//! Short messages (rejections, connection notes) shown for 1.5 s.

use bevy::prelude::*;

use super::{label, PANEL};
use crate::i18n::{Lang, Msg};

/// What a toast says: text its producer already composed in the active
/// language, or a catalog message resolved when the toast is shown.
enum Say {
    Text(String),
    Msg(Msg, Vec<(&'static str, String)>),
}

#[derive(Resource, Default)]
pub struct Toasts(Vec<(Say, f32)>);

impl Toasts {
    /// Show `text` for `secs` seconds (replaces any toast on screen).
    pub fn push_for(&mut self, text: impl Into<String>, secs: f32) {
        self.0.push((Say::Text(text.into()), secs));
    }

    pub fn push(&mut self, text: impl Into<String>) {
        self.0.push((Say::Text(text.into()), 1.5));
    }

    /// Show a catalog message for 1.5 s.
    pub fn say(&mut self, msg: Msg) {
        self.0.push((Say::Msg(msg, Vec::new()), 1.5));
    }

    /// Show a catalog template filled with `args` for 1.5 s.
    pub fn say_with(&mut self, msg: Msg, args: &[(&'static str, String)]) {
        self.0.push((Say::Msg(msg, args.to_vec()), 1.5));
    }
}

#[derive(Component)]
struct Toast(f32);

/// Persistent notice (e.g. "Reconnecting...") shown while `Some`.
#[derive(Resource, Default)]
pub struct Banner(pub Option<String>);

#[derive(Component)]
struct BannerNode;

pub fn plugin(app: &mut App) {
    app.init_resource::<Toasts>()
        .init_resource::<Banner>()
        .add_systems(
            Update,
            (show, expire, banner.run_if(resource_changed::<Banner>)),
        );
}

fn show(
    mut commands: Commands,
    mut toasts: ResMut<Toasts>,
    lang: Res<Lang>,
    existing: Query<Entity, With<Toast>>,
) {
    let Some((say, secs)) = toasts.0.pop() else {
        return;
    };
    let text = match say {
        Say::Text(text) => text,
        Say::Msg(msg, args) => {
            let args: Vec<(&str, &dyn std::fmt::Display)> = args
                .iter()
                .map(|(name, value)| (*name, value as &dyn std::fmt::Display))
                .collect();
            lang.tf(msg, &args)
        }
    };
    toasts.0.clear();
    for e in &existing {
        commands.entity(e).despawn();
    }
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            top: percent(40.0),
            justify_self: JustifySelf::Center,
            padding: UiRect::axes(px(16.0), px(10.0)),
            border_radius: BorderRadius::all(px(10.0)),
            ..default()
        },
        BackgroundColor(PANEL),
        Toast(secs),
        children![label(text, 18.0)],
    ));
}

fn expire(mut commands: Commands, time: Res<Time>, mut toasts: Query<(Entity, &mut Toast)>) {
    for (e, mut t) in &mut toasts {
        t.0 -= time.delta_secs();
        if t.0 <= 0.0 {
            commands.entity(e).despawn();
        }
    }
}

fn banner(mut commands: Commands, banner: Res<Banner>, nodes: Query<Entity, With<BannerNode>>) {
    for e in &nodes {
        commands.entity(e).despawn();
    }
    let Some(text) = &banner.0 else { return };
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            top: percent(30.0),
            justify_self: JustifySelf::Center,
            padding: UiRect::axes(px(16.0), px(10.0)),
            border_radius: BorderRadius::all(px(10.0)),
            ..default()
        },
        BackgroundColor(PANEL),
        BannerNode,
        children![label(text.clone(), 20.0)],
    ));
}
