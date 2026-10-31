//! Screens and HUD (`bevy_ui`). Anything tappable carries a `UiAction`;
//! `input.rs` turns taps into actions and `actions.rs` applies them.

use bevy::prelude::*;
use tbg_core::protocol::{InviteToken, RoomCode};
use tbg_core::{Rejection, UnitId};

pub mod account;
mod account_ui;
mod actions;
#[cfg(test)]
mod arena_data_tests;
#[cfg(test)]
mod army_tests;
mod attribution;
#[cfg(test)]
mod augment_data_tests;
mod battle_hud;
#[cfg(test)]
mod battle_wiring_tests;
mod countdown;
mod desktop;
#[cfg(target_arch = "wasm32")]
mod desktop_actions;
mod evolve;
pub(crate) mod fortify;
#[cfg(test)]
mod glyphs;
mod hud;
mod hud_portraits;
#[cfg(not(target_os = "linux"))]
mod hud_top;
mod hud_update;
mod input;
pub(crate) mod inspect;
#[cfg(test)]
mod json_lite;
mod keyboard;
pub(crate) mod kit;
#[cfg(test)]
mod lobby_panels_tests;
mod menu;
#[cfg(target_os = "linux")]
mod native_battle;
#[cfg(target_os = "linux")]
mod native_ranking;
mod queue;
pub mod ranking;
#[cfg(not(target_os = "linux"))]
mod ranking_ui;
mod result;
mod shell;
mod start;
#[cfg(test)]
mod stronghold_data_tests;
pub(crate) mod surrender;
pub mod textform;
mod textform_bevy;
mod tips;
mod toast;

pub use actions::ActionQueue;
pub use toast::{Banner, Toasts};

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub enum UiAction {
    /// Greyed-out button: clicks do nothing.
    Disabled,
    Practice,
    #[cfg(target_os = "linux")]
    PracticeArena(tbg_core::Arena),
    Quick,
    CreateRoom,
    JoinRoom(RoomCode),
    // P39: Quick Challenge by link.
    CreateChallenge,
    JoinChallenge(InviteToken),
    ChallengeInvalid,
    ShareChallenge,
    RequestRematch,
    Cancel,
    CopyLink,
    PlayAgain,
    ToMenu,
    /// P47: watch a listed live match from its id (`?watch=` or the lobby).
    Watch(u64),
    /// P47: stop watching and go back to the menu. The desktop battle bridge
    /// (wasm only) builds it, so native builds never see the constructor.
    #[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
    LeaveWatch,
    Buy(u8),
    #[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
    Reserve {
        offer: u8,
        kind: tbg_core::UnitKind,
    },
    #[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
    Release(u8),
    #[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
    Reroll,
    /// Long-press on a shop card: show the unit details.
    Inspect(u8),
    ToggleSound,
    // P12: ranked queue (accounts only).
    Ranked,
    // P11: accounts (handled in actions.rs + account.rs).
    CreateAccount,
    HaveKey,
    CopyKey,
    DownloadKey,
    KeySaved,
    ClosePanel,
    Logout,
    SelectUnit(UnitId),
    /// Click on your own Stronghold: opens its Fortification panel (like a unit's Evolution).
    SelectTower,
    Evolve {
        unit: UnitId,
        branch: u8,
    },
    CloseEvolve,
    /// Tap on empty ground or Escape: close Evolution and unit status.
    Deselect,
    /// The unit status panel's close button.
    CloseInspect,
    // P13: in-game Ranking panel (ui/ranking.rs).
    OpenRanking,
    RankingPrev,
    RankingNext,
    OpenFullRanking,
    OpenCreator,
    OpenRepository,
    CloseRanking,
    // BR-M06: two-tap surrender in the HUD.
    Surrender,
    #[cfg(target_os = "linux")]
    ShowArenaBanner,
    #[cfg(target_os = "linux")]
    CloseArenaBanner,
    // P41: lobby Settings language switch (DOM selector via data-action).
    #[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))] // built from DOM strings on the web
    SetLang(crate::i18n::Lang),
    /// ARENAS-G: the lobby's Arenas page shows its choice behind the menu.
    #[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
    // built from DOM strings on the web
    PreviewArena(tbg_core::Arena),
    // Owner revision 2026-10-29: War Council actions.
    ChooseAugment(u8),
    #[cfg(target_os = "linux")]
    PickAugment(u8),
    #[cfg(target_os = "linux")]
    ConfirmAugment,
    ForgoAugment,
    /// D-46: buy the next Fortification tier of your own tower on a branch.
    Fortify(u8),
}

/// Shop card waiting for the view that confirms (or rejects) its purchase.
#[derive(Resource, Default)]
pub struct PendingBuy(pub Option<(u8, u32)>);

pub const PANEL: Color = Color::srgba(0.07, 0.11, 0.15, 0.85);
pub const BUTTON: Color = Color::srgb(0.18, 0.42, 0.62);
pub const DISABLED: Color = Color::srgb(0.30, 0.32, 0.35);
pub const TEXT: Color = Color::srgb(0.96, 0.95, 0.90);

pub fn plugin(app: &mut App) {
    #[cfg(target_os = "linux")]
    native_battle::plugin(app);
    #[cfg(target_os = "linux")]
    native_ranking::plugin(app);
    app.init_resource::<PendingBuy>()
        .add_systems(
            Update,
            (input::taps, keyboard::navigate, actions::apply, scale_ui).chain(),
        )
        .add_plugins((
            actions::plugin,
            account::plugin,
            account_ui::plugin,
            textform::plugin,
            queue::plugin,
            desktop::plugin,
            menu::plugin,
            ranking::plugin,
            hud::plugin,
            evolve::plugin,
            result::plugin,
            start::plugin,
            surrender::plugin,
            toast::plugin,
            tips::plugin,
        ))
        .add_plugins((
            battle_hud::plugin,
            inspect::plugin,
            fortify::plugin,
            shell::plugin,
            kit::plugin,
            attribution::plugin,
        ));
}

/// Scale the whole UI with the smaller window side (phones ≈ 0.9, desktop 1.5).
fn scale_ui(window: Single<&Window>, mut scale: ResMut<UiScale>) {
    let s = if crate::config::bevy_ui() {
        kit::scale(window.width(), window.height())
    } else {
        (window.width().min(window.height()) / 420.0).clamp(0.75, 1.5)
    };
    if (scale.0 - s).abs() > 0.01 {
        scale.0 = s;
    }
}

pub fn rejection_text(reason: Rejection, lang: crate::i18n::Lang) -> String {
    crate::i18n::content::rejection(lang, reason).to_string()
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
    kit::button(text.to_owned(), action, width)
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
        padding: UiRect {
            bottom: px(40.0),
            ..UiRect::all(px(12.0))
        },
        row_gap: px(12.0),
        ..default()
    },)
}
