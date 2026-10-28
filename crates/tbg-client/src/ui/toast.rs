//! Short messages (rejections, connection notes) shown for 1.5 s.

use bevy::prelude::*;

use super::{label, PANEL};

#[derive(Resource, Default)]
pub struct Toasts(Vec<String>);

impl Toasts {
    pub fn push(&mut self, text: impl Into<String>) {
        self.0.push(text.into());
    }
}

#[derive(Component)]
struct Toast(f32);

pub fn plugin(app: &mut App) {
    app.init_resource::<Toasts>()
        .add_systems(Update, (show, expire));
}

fn show(mut commands: Commands, mut toasts: ResMut<Toasts>, existing: Query<Entity, With<Toast>>) {
    let Some(text) = toasts.0.pop() else { return };
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
        Toast(1.5),
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
