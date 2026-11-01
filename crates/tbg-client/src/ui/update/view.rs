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
