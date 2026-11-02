//! The update overlay (P66 §3.1/§3.2): one modal at a time, only in the
//! idle lobby. The pending "what's new" receipt comes first; then the
//! available update, download progress, the staged restart or a localized
//! error. Text is plain (`Text` nodes), never markup, and every string
//! comes from the catalog in the player's language.

use bevy::prelude::*;
use serde_json::Value;

use super::{UpdState, Updater};
use crate::i18n::Lang;
use crate::ui::{kit, UiAction};

#[derive(Component)]
pub(super) struct UpdateOverlay;

fn fill(template: String, pairs: &[(&str, String)]) -> String {
    pairs
        .iter()
        .fold(template, |t, (k, v)| t.replace(&format!("{{{k}}}"), v))
}

fn mib(bytes: u64) -> String {
    format!("{:.1} MiB", bytes as f64 / (1024.0 * 1024.0))
}

/// Locale field of a notes value (`{"en": .., "pt-BR": ..}`).
fn localized(v: &Value, lang: Lang) -> Option<&Value> {
    let code = if lang == Lang::PtBr { "pt-BR" } else { "en" };
    v.get(code).or_else(|| v.get("en"))
}

/// What the modal shows: heading, lines, buttons.
type Modal = (String, Vec<String>, Vec<(String, UiAction)>);

fn notes_modal(n: &Value, lang: Lang) -> Modal {
    if n.get("kind").and_then(Value::as_str) == Some("recovered") {
        let body = lang.key("update.recovered.body");
        return (
            lang.key("update.recovered.title"),
            vec![body],
            vec![(lang.key("update.notes.ok"), UiAction::UpdateNotesOk)],
        );
    }
    let notes = n.get("notes").cloned().unwrap_or(Value::Null);
    let title = notes
        .get("title")
        .and_then(|t| localized(t, lang))
        .and_then(Value::as_str)
        .map_or_else(|| lang.key("update.notes.title"), String::from);
    let mut lines = Vec::new();
    for section in notes
        .get("sections")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .take(4)
    {
        let kind = section
            .get("kind")
            .and_then(Value::as_str)
            .unwrap_or("highlights");
        lines.push(lang.key(&format!("update.section.{kind}")));
        let items = section
            .get("items")
            .and_then(|i| localized(i, lang))
            .and_then(Value::as_array);
        for item in items
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
            .take(40)
        {
            lines.push(format!("• {item}"));
        }
    }
    (
        title,
        lines,
        vec![(lang.key("update.notes.ok"), UiAction::UpdateNotesOk)],
    )
}

fn state_modal(state: &UpdState, lang: Lang) -> Option<Modal> {
    Some(match state {
        UpdState::Available {
            version,
            size,
            required,
            ..
        } => {
            let mut lines = vec![fill(
                lang.key("update.available.body"),
                &[("version", version.clone()), ("size", mib(*size))],
            )];
            if *required {
                lines.push(lang.key("update.required.body"));
            }
            let buttons = vec![
                (lang.key("update.now"), UiAction::UpdateNow),
                (lang.key("update.later"), UiAction::UpdateLater),
            ];
            (lang.key("update.available.title"), lines, buttons)
        }
        UpdState::Downloading { done, total, .. } => {
            let line = fill(
                lang.key("update.downloading"),
                &[("done", mib(*done)), ("total", mib(*total))],
            );
            (
                lang.key("update.available.title"),
                vec![line],
                vec![(lang.key("native.cancel"), UiAction::UpdateCancel)],
            )
        }
        UpdState::Staged { version } => {
            let body = fill(
                lang.key("update.staged.body"),
                &[("version", version.clone())],
            );
            let buttons = vec![
                (lang.key("update.restart"), UiAction::UpdateRestart),
                (lang.key("update.later"), UiAction::UpdateLater),
            ];
            (lang.key("update.staged.title"), vec![body], buttons)
        }
        UpdState::Error(code) => {
            let key = match code.as_str() {
                "unavailable" | "download" => "update.error.offline",
                "verification" => "update.error.verification",
                "disk" => "update.error.disk",
                "setup" => "update.setup_required",
                _ => "update.error.other",
            };
            (
                lang.key("update.available.title"),
                vec![lang.key(key)],
                vec![(lang.key("update.notes.ok"), UiAction::UpdateLater)],
            )
        }
        UpdState::Off | UpdState::Checking | UpdState::Applying => return None,
    })
}

/// The modal for the current state, if the lobby should show one.
fn modal(updater: &Updater, lang: Lang) -> Option<Modal> {
    if updater.opened && matches!(updater.state, UpdState::Off | UpdState::Checking) {
        return Some((
            lang.key("native.downloads"),
            vec![lang.key(if updater.state == UpdState::Checking {
                "update.checking"
            } else {
                "update.current"
            })],
            vec![
                (lang.key("update.check"), UiAction::UpdateCheck),
                (
                    lang.key(if updater.automatic {
                        "update.auto_on"
                    } else {
                        "update.auto_off"
                    }),
                    UiAction::UpdateAutomatic,
                ),
                (lang.key("common.close"), UiAction::UpdateLater),
            ],
        ));
    }
    if let Some(n) = &updater.notes {
        return Some(notes_modal(n, lang));
    }
    let snoozed = |id: &str| updater.snoozed.as_deref() == Some(id);
    match &updater.state {
        UpdState::Available { build, .. } if snoozed(build) => None,
        UpdState::Staged { version } if snoozed(version) => None,
        other => state_modal(other, lang),
    }
}

pub(super) fn overlay(
    mut commands: Commands,
    updater: Res<Updater>,
    lang: Res<Lang>,
    roots: Query<Entity, With<UpdateOverlay>>,
    mut shown: Local<String>,
) {
    let wanted = updater.idle.then(|| modal(&updater, *lang)).flatten();
    let signature = format!("{wanted:?}");
    if signature == *shown {
        return;
    }
    *shown = signature;
    for e in &roots {
        commands.entity(e).despawn();
    }
    let Some((heading, lines, buttons)) = wanted else {
        return;
    };
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: percent(100),
                height: percent(100),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0., 0., 0., 0.72)),
            GlobalZIndex(90),
            UpdateOverlay,
        ))
        .with_children(|root| {
            root.spawn(kit::panel()).with_children(|p| {
                p.spawn(kit::heading(heading, 24.));
                for line in lines {
                    p.spawn((
                        crate::ui::label(line, 16.),
                        Node {
                            max_width: px(560),
                            ..default()
                        },
                    ));
                }
                p.spawn(kit::row()).with_children(|row| {
                    for (text, action) in buttons {
                        row.spawn(kit::button(text, action, 170.));
                    }
                });
            });
        });
}

#[derive(Component)]
pub(super) struct UpdateBadge;

/// Later leaves an accessible non-modal way back to the verified update.
pub(super) fn badge(
    mut commands: Commands,
    updater: Res<Updater>,
    lang: Res<Lang>,
    roots: Query<Entity, With<UpdateBadge>>,
    mut shown: Local<String>,
) {
    let wanted = updater.idle
        && updater.snoozed.is_some()
        && matches!(
            updater.state,
            UpdState::Available { .. } | UpdState::Staged { .. }
        );
    let signature = format!("{wanted}:{}", lang.key("update.open"));
    if signature == *shown {
        return;
    }
    *shown = signature;
    for entity in &roots {
        commands.entity(entity).despawn();
    }
    if wanted {
        commands
            .spawn((
                Node {
                    position_type: PositionType::Absolute,
                    top: px(118.),
                    justify_self: JustifySelf::Center,
                    ..default()
                },
                GlobalZIndex(80),
                UpdateBadge,
            ))
            .with_children(|root| {
                root.spawn(kit::button(
                    lang.key("update.open"),
                    UiAction::UpdateOpen,
                    280.,
                ));
            });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn br_up01_settings_opens_an_in_app_modal_for_checking_and_current_builds() {
        let mut updater = Updater {
            opened: true,
            state: UpdState::Checking,
            ..Default::default()
        };
        for lang in [Lang::En, Lang::PtBr] {
            let (_, lines, buttons) = modal(&updater, lang).unwrap();
            assert_eq!(lines, vec![lang.key("update.checking")]);
            assert!(buttons
                .iter()
                .any(|(_, action)| *action == UiAction::UpdateAutomatic));
        }
        updater.state = UpdState::Off;
        assert!(modal(&updater, Lang::PtBr).is_some());
        updater.opened = false;
        assert!(modal(&updater, Lang::PtBr).is_none());
    }
}
