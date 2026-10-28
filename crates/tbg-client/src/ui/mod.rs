//! Screens and HUD (`bevy_ui`). Anything tappable carries a `UiAction`;
//! `input.rs` turns taps into actions and `actions.rs` applies them.

use bevy::prelude::*;
use tbg_core::protocol::RoomCode;
use tbg_core::{Rejection, UnitId};

mod actions;
mod evolve;
mod hud;
mod hud_update;
mod input;
mod menu;
mod queue;
mod result;
mod toast;

pub use actions::ActionQueue;
pub use toast::{Banner, Toasts};

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub enum UiAction {
    /// Greyed-out button: clicks do nothing.
    Disabled,
    Practice,
    Quick,
    CreateRoom,
    JoinRoom(RoomCode),
    Cancel,
    CopyLink,
    PlayAgain,
    ToMenu,
    Buy(u8),
    SelectUnit(UnitId),
    Evolve {
        unit: UnitId,
        branch: u8,
    },
    CloseEvolve,
}

/// Shop card waiting for the view that confirms (or rejects) its purchase.
#[derive(Resource, Default)]
pub struct PendingBuy(pub Option<(u8, u32)>);

pub const PANEL: Color = Color::srgba(0.07, 0.11, 0.15, 0.85);
pub const BUTTON: Color = Color::srgb(0.18, 0.42, 0.62);
pub const DISABLED: Color = Color::srgb(0.30, 0.32, 0.35);
pub const TEXT: Color = Color::srgb(0.96, 0.95, 0.90);

pub fn plugin(app: &mut App) {
    app.init_resource::<PendingBuy>()
        .add_systems(Update, (input::taps, actions::apply, scale_ui).chain())
        .add_plugins((
            actions::plugin,
            queue::plugin,
            menu::plugin,
            hud::plugin,
            evolve::plugin,
            result::plugin,
            toast::plugin,
        ));
}

/// Scale the whole UI with the smaller window side (phones ≈ 0.9, desktop 1.5).
fn scale_ui(window: Single<&Window>, mut scale: ResMut<UiScale>) {
    let s = (window.width().min(window.height()) / 420.0).clamp(0.75, 1.5);
    if (scale.0 - s).abs() > 0.01 {
        scale.0 = s;
    }
}

pub fn rejection_text(reason: Rejection) -> String {
    match reason {
        Rejection::NotEnoughEnergy => "Not enough Energy",
        Rejection::SlotCapReached => "Army is full (8 slots)",
        Rejection::NotRunning => "The match is not running",
        _ => "Action not possible",
    }
    .to_string()
}

/// Text bundle that never blocks clicks on what is behind it.
pub fn label(text: impl Into<String>, size: f32) -> impl Bundle {
    (
        Text::new(text),
        TextFont {
            font_size: FontSize::Px(size),
            ..default()
        },
        TextColor(TEXT),
    )
}

/// A clickable button; `UiAction::Disabled` greys it out.
pub fn button(text: &str, action: UiAction, width: f32) -> impl Bundle + use<> {
    let text = text.to_string();
    let color = if action == UiAction::Disabled {
        DISABLED
    } else {
        BUTTON
    };
    (
        Node {
            width: px(width),
            padding: UiRect::axes(px(12.0), px(10.0)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border_radius: BorderRadius::all(px(10.0)),
            ..default()
        },
        BackgroundColor(color),
        action,
        children![label(text, 18.0)],
    )
}

/// Full-screen container that lets clicks pass through to the arena.
pub fn screen(justify: JustifyContent) -> impl Bundle {
    (Node {
        width: percent(100.0),
        height: percent(100.0),
        position_type: PositionType::Absolute,
        flex_direction: FlexDirection::Column,
        justify_content: justify,
        align_items: AlignItems::Center,
        padding: UiRect::all(px(12.0)),
        row_gap: px(12.0),
        ..default()
    },)
}
