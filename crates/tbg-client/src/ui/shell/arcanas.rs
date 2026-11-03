//! Arcanas draft editing with the core's costs/prerequisites and explicit complete saves.

use super::{button, kit, Action, Lang, Shell};
use bevy::prelude::*;
use serde_json::Value;
use std::sync::OnceLock;
use tbg_core::{
    arcana::{BUDGET, COST, MAX, NODES, PREREQUISITES},
    ArcanaPreset,
};

/// Validate every edit with the authoritative content tables, including dependent removals.
pub fn change(draft: &mut ArcanaPreset, index: usize, delta: i8) -> bool {
    if index >= NODES {
        return false;
    }
    let mut next = *draft;
    let Some(rank) = next.ranks[index].checked_add_signed(delta) else {
        return false;
    };
    next.ranks[index] = rank;
    if rank > MAX[index] || !next.ranks_valid() || next.spent() > u32::from(BUDGET) {
        return false;
    }
    *draft = next;
    true
}

pub fn spawn(
    root: &mut ChildSpawnerCommands,
    shell: &Shell,
    assets: &AssetServer,
    lang: Lang,
    compact: bool,
) {
    let data = arcana_data();
    root.spawn(kit::panel()).with_children(|p| {
        p.spawn(kit::heading(lang.key("lobby.tabs.arcanas"), 32.));
        p.spawn(kit::muted(
            format!(
                "{} / {} · {}",
                shell.draft.spent(),
                BUDGET,
                lang.key("native.arcana_points")
            ),
            18.,
        ));
        p.spawn(kit::muted(
            if shell.notice.is_empty() {
                lang.key("native.arcana_scope")
            } else {
                shell.notice.clone()
            },
            15.,
        ));
        p.spawn(kit::row()).with_children(|r| {
            r.spawn(button(
                lang.key("native.default"),
                Action::DefaultArcanas,
                180.,
            ));
            r.spawn(button(lang.key("native.reset"), Action::ResetArcanas, 140.));
        });
    });
    root.spawn(Node {
        flex_direction: if compact && !cfg!(target_os = "android") {
            FlexDirection::Column
        } else {
            FlexDirection::Row
        },
        column_gap: px(16),
        row_gap: px(16),
        width: percent(100),
        ..default()
    })
    .with_children(|columns| {
        for family in data["families"].as_array().into_iter().flatten() {
            let family_id = family["id"].as_str().unwrap_or_default();
            columns
                .spawn(Node {
                    flex_grow: 1.,
                    flex_basis: px(0),
                    min_width: px(if cfg!(target_os = "android") { 0 } else { 300 }),
                    flex_direction: FlexDirection::Column,
                    row_gap: px(12),
                    ..default()
                })
                .with_children(|col| {
                    col.spawn(kit::panel()).with_children(|p| {
                        #[cfg(tbg_native_ui)]
                        p.spawn(super::catalog::banner(
                            assets,
                            format!(
                                "arcana/{}",
                                family["banner"]["display"].as_str().unwrap_or_default()
                            ),
                            2.5,
                        ));
                        #[cfg(not(tbg_native_ui))]
                        p.spawn((
                            ImageNode::new(assets.load(format!(
                                "arcana/{}",
                                family["banner"]["display"].as_str().unwrap_or_default()
                            ))),
                            Node {
                                width: percent(100),
                                height: px(120),
                                ..default()
                            },
                        ));
                        p.spawn(kit::heading(
                            lang.key(&format!("arcana.family.{family_id}.name")),
                            28.,
                        ));
                        p.spawn(kit::muted(
                            lang.key(&format!("arcana.family.{family_id}.description")),
                            15.,
                        ));
                    });
                    for node in data["nodes"]
                        .as_array()
                        .into_iter()
                        .flatten()
                        .filter(|n| n["family"].as_str() == Some(family_id))
                    {
                        let Some(i) = node["index"]
                            .as_u64()
                            .map(|i| i as usize)
                            .filter(|i| *i < NODES)
                        else {
                            continue;
                        };
                        col.spawn(kit::panel()).with_children(|p| {
                            p.spawn(kit::row()).with_children(|r| {
                                r.spawn(kit::image(
                                    assets,
                                    format!(
                                        "arcana/{}",
                                        node["asset"]["display"].as_str().unwrap_or_default()
                                    ),
                                    58.,
                                    58.,
                                ));
                                r.spawn(kit::heading(node_copy(lang, node, "name"), 22.));
                            });
                            for key in ["per_rank", "effect", "eligibility"] {
                                p.spawn(kit::muted(node_copy(lang, node, key), 14.));
                            }
                            if !PREREQUISITES[i].is_empty() {
                                let needs = PREREQUISITES[i]
                                    .iter()
                                    .map(|(n, r)| {
                                        format!(
                                            "{} {r}",
                                            node_copy(lang, &data["nodes"][*n], "name")
                                        )
                                    })
                                    .collect::<Vec<_>>()
                                    .join(" · ");
                                p.spawn(kit::muted(
                                    format!("{}: {needs}", lang.key("native.requires")),
                                    13.,
                                ));
                            }
                            p.spawn(Node {
                                flex_wrap: if cfg!(target_os = "android") {
                                    FlexWrap::Wrap
                                } else {
                                    FlexWrap::NoWrap
                                },
                                row_gap: px(6),
                                ..kit::row()
                            })
                            .with_children(|r| {
                                r.spawn(button("-", Action::Arcana(i, -1), 42.));
                                r.spawn(kit::heading(
                                    format!("{} / {}", shell.draft.ranks[i], MAX[i]),
                                    20.,
                                ));
                                r.spawn(button("+", Action::Arcana(i, 1), 42.));
                                r.spawn(kit::muted(
                                    format!("{} Energy?", COST[i])
                                        .replace("Energy?", &lang.key("native.points")),
                                    13.,
                                ));
                            });
                        });
                    }
                });
        }
    });
}

fn node_copy(lang: Lang, node: &Value, field: &str) -> String {
    let id = node["id"].as_str().unwrap_or_default().replace('-', "_");
    lang.key(&format!("arcana.node.{id}.{field}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn br_ar02_ar03_editor_cannot_exceed_budget_or_break_dependencies() {
        let mut draft = ArcanaPreset::DEFAULT;
        assert!(!change(&mut draft, 6, 1));
        assert!(change(&mut draft, 2, -1));
        assert!(change(&mut draft, 6, 1));
        assert!(change(&mut draft, 0, -1));
        assert!(!change(&mut draft, 0, -1));
        assert!(change(&mut draft, 0, 1));
        assert!(!change(&mut draft, 99, 1));
        assert_eq!(draft.spent(), 12);
        assert!(ArcanaPreset::complete(draft.ranks).is_ok());
    }
}

pub fn arcana_data() -> &'static Value {
    static DATA: OnceLock<Value> = OnceLock::new();
    DATA.get_or_init(|| {
        serde_json::from_str(include_str!("../../../assets/arcana/arcana.json"))
            .expect("bundled Arcana catalog")
    })
}
