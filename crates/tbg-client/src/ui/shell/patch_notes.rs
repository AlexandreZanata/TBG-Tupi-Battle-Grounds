//! Shared native/browser patch history with a validated online feed and offline snapshot.
use super::{button, data::Data, kit, lobby::chrome, Action, Panel};
use crate::i18n::Lang;
use bevy::prelude::*;
use serde_json::Value;
use std::sync::OnceLock;

pub fn parse(value: Value) -> Option<Value> {
    if value.to_string().len() > 262144 {
        return None;
    }
    let notes = value.get("notes")?.as_array()?;
    if value["schema"] != 1 || notes.is_empty() || notes.len() > 50 {
        return None;
    }
    let mut ids = std::collections::BTreeSet::new();
    for note in notes {
        let id = note.get("id")?.as_str()?;
        if id.is_empty()
            || id.len() > 64
            || !id.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-')
            || !ids.insert(id)
        {
            return None;
        }
        for lang in ["en", "pt-BR"] {
            if note["title"][lang].as_str()?.len() > 180
                || note["summary"][lang].as_str()?.len() > 1600
            {
                return None;
            }
            let sections = note["sections"][lang].as_array()?;
            if sections.len() > 32 {
                return None;
            }
            for section in sections {
                if !bounded_text(&section["title"], 180) || !bounded_text(&section["body"], 1600) {
                    return None;
                }
            }
        }
        let changes = note["changes"].as_array()?;
        if changes.len() > 128 {
            return None;
        }
        for change in changes {
            for field in ["subject", "field", "before", "after", "reason"] {
                let value = &change[field];
                let valid = if value.is_object() {
                    ["en", "pt-BR"]
                        .into_iter()
                        .all(|lang| bounded_text(&value[lang], 1600))
                } else {
                    value.is_number() || bounded_text(value, 1600)
                };
                if !valid {
                    return None;
                }
            }
        }
    }
    Some(value)
}

fn bounded_text(value: &Value, limit: usize) -> bool {
    value
        .as_str()
        .is_some_and(|s| !s.is_empty() && s.len() <= limit)
}

fn snapshot() -> &'static Value {
    static SNAPSHOT: OnceLock<Value> = OnceLock::new();
    SNAPSHOT.get_or_init(|| {
        serde_json::from_str(include_str!("../../../../../docs/patches/catalog.json"))
            .expect("validated patch snapshot")
    })
}

fn local(value: &Value, lang: Lang) -> &str {
    value.as_str().unwrap_or_else(|| {
        value[if lang == Lang::PtBr { "pt-BR" } else { "en" }]
            .as_str()
            .unwrap_or("")
    })
}

fn newest_first(catalog: &Value) -> Vec<&Value> {
    let mut ids = std::collections::BTreeSet::new();
    let mut notes: Vec<_> = catalog["notes"]
        .as_array()
        .into_iter()
        .flatten()
        .chain(snapshot()["notes"].as_array().into_iter().flatten())
        .filter(|note| ids.insert(note["id"].as_str().unwrap_or_default()))
        .collect();
    notes.sort_by(|a, b| {
        b["date"]
            .as_str()
            .cmp(&a["date"].as_str())
            .then_with(|| {
                b["ruleset_version"]
                    .as_u64()
                    .cmp(&a["ruleset_version"].as_u64())
            })
            .then_with(|| {
                let version = |note: &Value| {
                    note["version"]
                        .as_str()
                        .and_then(|s| semver::Version::parse(s).ok())
                };
                match (version(a), version(b)) {
                    (Some(a), Some(b)) => b.cmp_precedence(&a),
                    _ => std::cmp::Ordering::Equal,
                }
            })
    });
    notes
}

pub fn spawn(root: &mut ChildSpawnerCommands, data: &Data, assets: &AssetServer, lang: Lang) {
    root.spawn(kit::panel()).with_children(|panel| {
        panel
            .spawn(Node {
                column_gap: px(24),
                align_items: AlignItems::Center,
                ..default()
            })
            .with_children(|hero| {
                hero.spawn(kit::image(
                    assets,
                    "lobby/news/patch-notes.webp",
                    330.,
                    110.,
                ));
                hero.spawn(Node {
                    flex_grow: 1.,
                    row_gap: px(10),
                    ..chrome::column()
                })
                .with_children(|copy| {
                    copy.spawn(kit::heading(lang.key("patches.title"), 32.));
                    copy.spawn(kit::muted(
                        lang.key(if data.patch_notes.is_null() {
                            "patches.offline"
                        } else {
                            "patches.intro"
                        }),
                        15.,
                    ));
                });
                hero.spawn(button(
                    lang.key("lobby.tabs.lobby"),
                    Action::Open(Panel::Lobby),
                    160.,
                ));
            });
    });
    let catalog = if data.patch_notes.is_null() {
        snapshot()
    } else {
        &data.patch_notes
    };
    for note in newest_first(catalog) {
        root.spawn(kit::panel()).with_children(|panel| {
            panel.spawn(kit::muted(
                format!(
                    "{} · {} · {} {}",
                    note["version"].as_str().unwrap_or(""),
                    note["date"].as_str().unwrap_or(""),
                    lang.key("patches.ruleset"),
                    note["ruleset_version"]
                ),
                13.,
            ));
            panel.spawn(kit::heading(local(&note["title"], lang), 27.));
            panel.spawn(kit::muted(local(&note["summary"], lang), 16.));
            if note["status"] != "released" {
                panel.spawn(kit::muted(lang.key("patches.preview"), 13.));
            }
            for change in note["changes"].as_array().into_iter().flatten() {
                panel
                    .spawn(Node {
                        padding: UiRect::all(px(12)),
                        row_gap: px(6),
                        border: UiRect::left(px(2)),
                        ..chrome::column()
                    })
                    .insert((
                        BackgroundColor(chrome::METAL),
                        BorderColor::all(chrome::EDGE),
                    ))
                    .with_children(|row| {
                        row.spawn(kit::heading(local(&change["subject"], lang), 18.));
                        let display = |value: &Value| {
                            if value.is_object() {
                                local(value, lang).to_owned()
                            } else {
                                value
                                    .as_str()
                                    .map(str::to_owned)
                                    .unwrap_or_else(|| value.to_string())
                            }
                        };
                        row.spawn(kit::muted(
                            format!(
                                "{}: {} -> {}",
                                local(&change["field"], lang),
                                display(&change["before"]),
                                display(&change["after"])
                            ),
                            15.,
                        ));
                        row.spawn(kit::muted(local(&change["reason"], lang), 14.));
                    });
            }
            for section in note["sections"][if lang == Lang::PtBr { "pt-BR" } else { "en" }]
                .as_array()
                .into_iter()
                .flatten()
            {
                panel.spawn(kit::heading(section["title"].as_str().unwrap_or(""), 20.));
                panel.spawn(kit::muted(section["body"].as_str().unwrap_or(""), 15.));
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn patch_history_is_newest_first_even_when_the_feed_appends_new_releases() {
        let feed = serde_json::json!({"notes":[
            {"id":"middle", "date":"9999-12-31", "ruleset_version":15,"version":"0.1.4"},
            {"id":"oldest", "date":"0001-01-01", "ruleset_version":14,"version":"0.1.3"},
            {"id":"newest", "date":"9999-12-31", "ruleset_version":16,"version":"0.1.5"}
        ]});
        let notes = newest_first(&feed);
        assert_eq!(notes.first().unwrap()["id"], "newest");
        assert_eq!(notes.last().unwrap()["id"], "oldest");
        assert_eq!(
            notes.len(),
            snapshot()["notes"].as_array().unwrap().len() + 3
        );
        assert!(notes
            .windows(2)
            .all(|pair| pair[0]["date"].as_str() >= pair[1]["date"].as_str()));
    }
    #[test]
    fn compiled_notes_are_valid_and_malformed_feeds_are_rejected() {
        assert!(parse(snapshot().clone()).is_some());
        let mut oversized = snapshot().clone();
        oversized["notes"][0]["sections"]["en"][0]["body"] = Value::String("x".repeat(1601));
        assert!(parse(oversized).is_none());
        let mut duplicated = snapshot().clone();
        duplicated["notes"][1]["id"] = duplicated["notes"][0]["id"].clone();
        assert!(parse(duplicated).is_none());
        assert!(parse(serde_json::json!({"schema":99,"notes":[]})).is_none());
        assert!(parse(serde_json::json!({"schema":1,"notes":[{"id":"../private"}]})).is_none());
    }
}
