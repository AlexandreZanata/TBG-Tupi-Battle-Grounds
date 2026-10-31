//! Two-tap surrender (BR-M06, Proposed): the first tap arms for four
//! seconds, the second sends `ClientMsg::Surrender` (or leaves practice).
//! One resource holds the state; the button text updates only on change.

use bevy::prelude::*;

use super::{label, UiAction, DISABLED};
use crate::GameState;

pub(super) const ARMED_SECS: f64 = 4.0;
const SURRENDER_BG: Color = Color::srgb(0.45, 0.18, 0.20);

#[derive(Resource, Default)]
pub struct Surrender {
    pub armed_until: Option<f64>,
    pub sent: bool,
    pub ranked: bool,
}

#[derive(Component)]
struct SurrenderButton;

pub fn plugin(app: &mut App) {
    app.init_resource::<Surrender>()
        .add_systems(Update, refresh.run_if(in_state(GameState::InMatch)));
}

/// Called on every `MatchStart` (a new match is not armed).
pub fn reset(state: &mut Surrender, ranked: bool) {
    *state = Surrender {
        ranked,
        ..default()
    };
}

pub(super) fn button() -> impl Bundle {
    (
        Node {
            width: px(230.0),
            min_height: px(44.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border_radius: BorderRadius::all(px(10.0)),
            ..default()
        },
        BackgroundColor(SURRENDER_BG),
        UiAction::Surrender,
        SurrenderButton,
        children![label("", 12.0)],
    )
}

fn refresh(
    time: Res<Time>,
    mut state: ResMut<Surrender>,
    lang: Res<crate::i18n::Lang>,
    mut buttons: Query<(&mut UiAction, &mut BackgroundColor, &Children), With<SurrenderButton>>,
    mut texts: Query<&mut Text>,
) {
    let now = time.elapsed_secs_f64();
    if state.armed_until.is_some_and(|until| now >= until) {
        state.armed_until = None;
    }
    use crate::i18n::Msg;
    let want = lang.t(if state.sent {
        Msg::SurrenderSent
    } else if state.armed_until.is_some() {
        if state.ranked {
            Msg::SurrenderAgainLoss
        } else {
            Msg::SurrenderAgain
        }
    } else {
        Msg::BattleSurrender
    });
    let bg = if state.sent { DISABLED } else { SURRENDER_BG };
    for (mut action, mut color, children) in &mut buttons {
        let target = if state.sent {
            UiAction::Disabled
        } else {
            UiAction::Surrender
        };
        if *action != target {
            *action = target;
        }
        if color.0 != bg {
            color.0 = bg;
        }
        for child in children.iter() {
            if let Ok(mut text) = texts.get_mut(child) {
                if text.0 != want {
                    text.0 = want.into();
                }
            }
        }
    }
}
