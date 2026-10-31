//! Localized, nonblocking Result page/accessibility controls shared by browser and native Bevy UI.

use super::{range, total, Presentation};
use crate::{
    i18n::{Lang, Msg},
    sync::MatchInfo,
    ui::{button, label, UiAction},
    GameState,
};
use bevy::prelude::*;

#[derive(Component)]
struct Range;
#[derive(Component)]
struct Toggle;
#[derive(Component)]
struct Motion;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Update, update.run_if(in_state(GameState::Result)));
}

pub(crate) fn spawn(parent: &mut ChildSpawnerCommands, info: &MatchInfo, lang: Lang) {
    let Some(total) = total(info) else {
        return;
    };
    if total == 0 {
        parent.spawn(label(lang.t(Msg::CelebrationEmpty), 15.));
    }
    parent
        .spawn(Node {
            flex_wrap: FlexWrap::Wrap,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            column_gap: px(8),
            row_gap: px(5),
            ..default()
        })
        .with_children(|row| {
            row.spawn(button("<", UiAction::CelebrationPrev, 48.));
            row.spawn((
                label("", 14.),
                Node {
                    width: px(190),
                    ..default()
                },
                Range,
            ));
            row.spawn(button(">", UiAction::CelebrationNext, 48.));
            row.spawn(button(
                lang.t(Msg::CelebrationHide),
                UiAction::CelebrationHide,
                145.,
            ))
            .insert(Toggle);
            row.spawn(button(
                lang.t(Msg::CelebrationMotion),
                UiAction::CelebrationMotion,
                165.,
            ))
            .insert(Motion);
        });
    parent.spawn(label(lang.t(Msg::CelebrationShortcuts), 11.));
}

#[allow(clippy::type_complexity)]
fn update(
    info: Res<MatchInfo>,
    state: Res<Presentation>,
    lang: Res<Lang>,
    mut range_labels: Query<&mut Text, With<Range>>,
    mut buttons: Query<
        (
            &mut UiAction,
            Option<&Children>,
            Option<&Toggle>,
            Option<&Motion>,
        ),
        Without<Range>,
    >,
    mut texts: Query<&mut Text, Without<Range>>,
) {
    if !info.is_changed() && !state.is_changed() && !lang.is_changed() {
        return;
    }
    let Some(total) = total(&info) else {
        return;
    };
    let (first, last) = range(total, state.page);
    for mut text in &mut range_labels {
        text.0 = lang
            .t(Msg::CelebrationRange)
            .replace(
                "{from}",
                &if total == 0 { 0 } else { first + 1 }.to_string(),
            )
            .replace("{to}", &last.to_string())
            .replace("{total}", &total.to_string());
    }
    for (mut action, children, toggle, motion) in &mut buttons {
        let key = if toggle.is_some() {
            Some(if state.hidden {
                Msg::CelebrationShow
            } else {
                Msg::CelebrationHide
            })
        } else if motion.is_some() {
            Some(if state.reduced {
                Msg::CelebrationMotionOff
            } else {
                Msg::CelebrationMotion
            })
        } else {
            None
        };
        if let Some(key) = key {
            if let Some(children) = children {
                for child in children {
                    if let Ok(mut text) = texts.get_mut(*child) {
                        text.0 = lang.t(key).to_string();
                    }
                }
            }
        }
        // Controls always stay clickable; their bounded actions are no-ops at the ends.
        let _ = &mut action;
    }
}
