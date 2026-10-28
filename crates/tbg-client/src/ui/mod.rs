//! Screens and HUD (`bevy_ui`). Anything clickable carries a `UiAction`;
//! one observer turns clicks into actions and one system applies them.

use bevy::prelude::*;
use tbg_core::protocol::ClientMsg;
use tbg_core::{PlayerIntention, Rejection, UnitId};

use crate::link::MatchLink;
use crate::local::LocalMatch;
use crate::GameState;

mod evolve;
mod hud;
mod hud_update;
mod input;
mod menu;
mod result;
mod toast;

pub use toast::Toasts;

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub enum UiAction {
    /// Greyed-out button: clicks do nothing.
    Disabled,
    Practice,
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

#[derive(Resource, Default)]
struct ActionQueue(Vec<UiAction>);

#[derive(Resource, Default)]
struct IntentSeq(u32);

/// Shop card waiting for the view that confirms (or rejects) its purchase.
#[derive(Resource, Default)]
pub struct PendingBuy(pub Option<(u8, u32)>);

pub const PANEL: Color = Color::srgba(0.07, 0.11, 0.15, 0.85);
pub const BUTTON: Color = Color::srgb(0.18, 0.42, 0.62);
pub const DISABLED: Color = Color::srgb(0.30, 0.32, 0.35);
pub const TEXT: Color = Color::srgb(0.96, 0.95, 0.90);

pub fn plugin(app: &mut App) {
    app.init_resource::<ActionQueue>()
        .init_resource::<IntentSeq>()
        .init_resource::<PendingBuy>()
        .add_systems(Update, (input::taps, apply_actions, scale_ui).chain())
        .add_plugins((
            menu::plugin,
            hud::plugin,
            evolve::plugin,
            result::plugin,
            toast::plugin,
        ));
}

fn apply_actions(
    mut queue: ResMut<ActionQueue>,
    mut link: NonSendMut<MatchLink>,
    mut next: ResMut<NextState<GameState>>,
    mut seq: ResMut<IntentSeq>,
    mut pending: ResMut<PendingBuy>,
    mut selected: ResMut<evolve::Selected>,
    time: Res<Time>,
) {
    for action in std::mem::take(&mut queue.0) {
        let mut send = |intent| {
            seq.0 += 1;
            link.send(ClientMsg::Intent { seq: seq.0, intent });
            seq.0
        };
        match action {
            UiAction::Practice | UiAction::PlayAgain => {
                let seed = time.elapsed().as_nanos() as u64 ^ 0x9e37_79b9_7f4a_7c15;
                *link = MatchLink::Local(Box::new(LocalMatch::new(seed)));
                pending.0 = None;
                selected.0 = None;
            }
            UiAction::ToMenu => {
                *link = MatchLink::Idle;
                next.set(GameState::Menu);
            }
            UiAction::Buy(offer) => {
                let s = send(PlayerIntention::Purchase { offer });
                pending.0 = Some((offer, s));
            }
            UiAction::SelectUnit(id) => selected.0 = Some(id),
            UiAction::Evolve { unit, branch } => {
                send(PlayerIntention::Evolve { unit, branch });
                selected.0 = None;
            }
            UiAction::CloseEvolve => selected.0 = None,
            UiAction::Disabled => {}
        }
    }
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
