//! The in-match HUD, laid out on a 1920×1080 design grid scaled to the window.

use bevy::prelude::*;
use bevy::window::WindowResized;

use crate::app::Screen;
use crate::i18n::{Fonts, Lang};
use crate::input::Blocks;
use crate::palette;

pub mod chrome;
mod minimap;
mod notices;
pub use notices::button as notices_button;
mod roster;
mod shop;
mod top;

/// Root of everything spawned for one match screen.
#[derive(Component)]
pub struct HudRoot;

pub fn plugin(app: &mut App) {
    app.add_systems(Update, scale_ui)
        .add_systems(OnEnter(Screen::Match), spawn)
        .add_systems(OnExit(Screen::Match), despawn::<HudRoot>);
    top::plugin(app);
    roster::plugin(app);
    shop::plugin(app);
    minimap::plugin(app);
    notices::plugin(app);
}

/// One logical design pixel per 1/1080 of the window height.
fn scale_ui(
    mut resized: MessageReader<WindowResized>,
    window: Single<&Window>,
    mut scale: ResMut<UiScale>,
    mut done: Local<bool>,
) {
    if resized.read().count() == 0 && *done {
        return;
    }
    *done = true;
    scale.0 = (window.height() / 1080.0).clamp(0.55, 1.6);
}

pub fn despawn<T: Component>(mut commands: Commands, roots: Query<Entity, With<T>>) {
    for e in &roots {
        commands.entity(e).despawn();
    }
}

fn spawn(mut commands: Commands, lang: Res<Lang>, fonts: Res<Fonts>, assets: Res<AssetServer>) {
    let ctx = Ctx {
        lang: *lang,
        fonts: fonts.clone(),
        assets: assets.clone(),
    };
    commands
        .spawn((
            HudRoot,
            Node {
                width: percent(100),
                height: percent(100),
                position_type: PositionType::Absolute,
                ..default()
            },
        ))
        .with_children(|root| {
            top::spawn(root, &ctx);
            roster::spawn(root, &ctx);
            shop::spawn(root, &ctx);
            minimap::spawn(root);
            notices::spawn(root, &ctx);
        });
}

/// What every HUD part needs while spawning.
pub struct Ctx {
    pub lang: Lang,
    pub fonts: Fonts,
    pub assets: AssetServer,
}

pub fn text(value: impl Into<String>, size: f32, color: Color) -> impl Bundle {
    (
        Text::new(value),
        TextFont::from_font_size(FontSize::Px(size)),
        TextColor(color),
    )
}

pub fn title(value: impl Into<String>, size: f32, color: Color, fonts: &Fonts) -> impl Bundle {
    (
        Text::new(value),
        TextFont::from_font_size(FontSize::Px(size)).with_font(fonts.display.clone()),
        TextColor(color),
    )
}

/// A dark framed panel that blocks world clicks.
pub fn panel(node: Node, border: Color) -> impl Bundle {
    (
        Node {
            border: UiRect::all(px(2)),
            border_radius: BorderRadius::all(px(10)),
            ..node
        },
        BackgroundColor(palette::PANEL),
        BorderColor::all(border),
        Blocks,
    )
}

/// `18400` → `18.4k`.
pub fn short(value: u32) -> String {
    if value >= 1000 {
        format!("{:.1}k", value as f32 / 1000.0)
    } else {
        value.to_string()
    }
}

/// `m:ss`, like the 1v1 match clock.
pub fn clock(tick: u32) -> String {
    let s = tick / tbg_moba_core::content::TICK_HZ;
    format!("{}:{:02}", s / 60, s % 60)
}
