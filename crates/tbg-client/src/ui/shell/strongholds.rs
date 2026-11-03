//! Stronghold selection, canonical statistics and Fortification branch details.

use super::{button, kit, Action, Lang};
use bevy::prelude::*;
use serde_json::Value;
use std::sync::OnceLock;

#[cfg(tbg_native_ui)]
mod native;
#[cfg(tbg_native_ui)]
pub use native::spawn;

pub fn stronghold_data() -> &'static Value {
    static DATA: OnceLock<Value> = OnceLock::new();
    DATA.get_or_init(|| {
        serde_json::from_str(include_str!("../../../assets/stronghold/strongholds.json"))
            .expect("bundled Stronghold catalog")
    })
}
fn text(v: &Value, key: &str) -> String {
    v[key].as_str().unwrap_or_default().into()
}

#[cfg(not(tbg_native_ui))]
pub fn spawn(root: &mut ChildSpawnerCommands, assets: &AssetServer, lang: Lang) {
    let data = stronghold_data();
    root.spawn(kit::panel()).with_children(|p| {
        p.spawn(kit::heading(lang.key("lobby.tabs.stronghold"), 32.));
        p.spawn(kit::muted(text(&data["copy"], "intro"), 18.));
        p.spawn(kit::muted(text(&data["copy"], "scope"), 15.));
    });
    root.spawn(Node {
        flex_wrap: FlexWrap::Wrap,
        column_gap: px(16),
        row_gap: px(16),
        ..default()
    })
    .with_children(|grid| {
        for v in data["kinds"].as_array().into_iter().flatten() {
            let Some(kind) = crate::config::parse_stronghold(&text(v, "kind")) else {
                continue;
            };
            grid.spawn(kit::panel()).with_children(|p| {
                p.spawn(kit::image(
                    assets,
                    format!("stronghold/{}", text(&v["banner"], "display")),
                    310.,
                    150.,
                ));
                details(p, v, data, kind, lang);
            });
        }
    });
}

fn details(
    p: &mut ChildSpawnerCommands,
    v: &Value,
    data: &Value,
    kind: tbg_core::StrongholdKind,
    lang: Lang,
) {
    let slug = format!("{kind:?}").to_lowercase();
    p.spawn(kit::heading(
        crate::i18n::content::stronghold_name(lang, kind),
        if cfg!(target_os = "android") {
            20.
        } else {
            26.
        },
    ));
    p.spawn(kit::muted(
        lang.key(&format!("stronghold.{slug}.specialty")),
        16.,
    ));
    p.spawn((
        super::super::label(lang.key(&format!("stronghold.{slug}.summary")), 15.),
        detail_node(),
    ));
    let stats = &v["stats"];
    p.spawn(super::super::label(
        format!(
            "HP {} · {} {} · {} {}",
            stats["maxHp"],
            lang.key("native.damage"),
            stats["damage"],
            lang.key("native.range"),
            stats["range"].as_u64().unwrap_or(0) / 1000
        ),
        15.,
    ));
    p.spawn(kit::heading(
        lang.key(&format!(
            "stronghold.{}.passive.name",
            format!("{kind:?}").to_lowercase()
        )),
        17.,
    ));
    p.spawn((kit::muted(passive(lang, kind, stats), 14.), detail_node()));
    for (heading, field, key) in [
        ("+", "advantages", "advantage"),
        ("-", "disadvantages", "disadvantage"),
    ] {
        for (index, _line) in v[field]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
            .enumerate()
        {
            p.spawn((
                kit::muted(
                    format!(
                        "{heading} {}",
                        expand(
                            &lang.key(&format!("stronghold.{slug}.{key}.{index}")),
                            stats
                        )
                    ),
                    13.,
                ),
                detail_node(),
            ));
        }
    }
    for (branch_id, branch) in v["branches"].as_array().into_iter().flatten().enumerate() {
        p.spawn(kit::heading(
            crate::i18n::content::stronghold_branch(lang, kind, branch_id as u8),
            17.,
        ));
        for (i, step) in branch["steps"].as_array().into_iter().flatten().enumerate() {
            let s = &step["stats"];
            p.spawn((
                kit::muted(
                    format!(
                        "{} · {} Energy · HP {} · {} {}",
                        crate::i18n::content::stronghold_step(
                            lang,
                            kind,
                            branch_id as u8,
                            (i + 1) as u8
                        ),
                        data["fortifyCost"][i.min(1)].as_u64().unwrap_or(0)
                            / data["energyScale"].as_u64().unwrap_or(600),
                        s["maxHp"],
                        lang.key("native.damage"),
                        s["damage"]
                    ),
                    13.,
                ),
                detail_node(),
            ));
            p.spawn((kit::muted(branch_effect(step), 13.), detail_node()));
        }
    }
    #[cfg(tbg_native_ui)]
    p.spawn(Node {
        flex_grow: 1.,
        min_height: px(8),
        ..default()
    });
    let chosen = crate::config::load_stronghold() == kind;
    let control = button(
        lang.key(if chosen {
            "arenas.chosen"
        } else {
            "arenas.choose"
        }),
        Action::Stronghold(kind),
        290.,
    );
    #[cfg(tbg_native_ui)]
    p.spawn(control).insert(super::catalog::button_node());
    #[cfg(not(tbg_native_ui))]
    p.spawn(control);
}

fn detail_node() -> Node {
    Node {
        width: if cfg!(tbg_native_ui) {
            percent(100)
        } else {
            Val::Auto
        },
        max_width: if cfg!(tbg_native_ui) {
            percent(100)
        } else {
            px(310)
        },
        ..default()
    }
}
pub(in crate::ui) fn expand(raw: &str, stats: &Value) -> String {
    let data = stronghold_data();
    let seconds =
        data["windowTicks"].as_f64().unwrap_or(0.) / data["tickHz"].as_f64().unwrap_or(20.);
    let mut value = raw.replace("{window}", &format!("{} s", number(seconds)));
    if let Some(map) = stats.as_object() {
        for (key, n) in map {
            let raw = n.as_f64().unwrap_or(0.);
            let formatted = match key.as_str() {
                "spoils" | "spoilsCap" => {
                    number(raw / data["energyScale"].as_f64().unwrap_or(600.))
                }
                "armorBp" | "splashBp" | "auraBp" => format!("{}%", number(raw / 100.)),
                "splashRadius" | "range" | "auraRange" => format!("{} tiles", number(raw / 1000.)),
                "interval" => format!("{:.2} s", raw / data["tickHz"].as_f64().unwrap_or(20.)),
                _ => n.to_string(),
            };
            value = value.replace(&format!("{{{key}}}"), &formatted);
        }
    }
    value
}
fn number(value: f64) -> String {
    format!("{value:.2}")
        .trim_end_matches('0')
        .trim_end_matches('.')
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn banner_percent_and_milli_tiles_use_the_same_units_as_the_html_picker() {
        let value = serde_json::json!({"auraBp":500,"auraRange":4000,"spoils":90});
        assert_eq!(
            expand("+{auraBp} within {auraRange}; {spoils} Energy", &value),
            "+5% within 4 tiles; 0.15 Energy"
        );
    }
}

pub(in crate::ui) fn branch_effect(step: &Value) -> String {
    let stats = &step["stats"];
    let delta = &step["delta"];
    let mut parts = Vec::new();
    for (key, label, stat) in [
        ("hpBp", "max HP", "maxHp"),
        ("dmgBp", "damage", "damage"),
        ("rangeAdd", "range", "range"),
        ("interval", "attack interval", "interval"),
        ("spoilsAdd", "Spoils", "spoils"),
        ("spoilsCapAdd", "Spoils cap", "spoilsCap"),
        ("auraBpAdd", "War Banner", "auraBp"),
        ("auraRangeAdd", "War Banner radius", "auraRange"),
    ] {
        if delta[key].as_i64().is_some_and(|n| n != 0) {
            parts.push(format!(
                "{label}: {}",
                expand(&format!("{{{stat}}}"), stats)
            ));
        }
    }
    parts.join(" · ")
}

/// The same localized passive copy and authoritative catalog values feed every desktop target.
pub(in crate::ui) fn passive(lang: Lang, kind: tbg_core::StrongholdKind, stats: &Value) -> String {
    let key = format!(
        "stronghold.{}.passive.text",
        format!("{kind:?}").to_lowercase()
    );
    expand(&lang.key(&key), stats).replace(
        " tiles",
        if lang == Lang::PtBr {
            " casas"
        } else {
            " tiles"
        },
    )
}
