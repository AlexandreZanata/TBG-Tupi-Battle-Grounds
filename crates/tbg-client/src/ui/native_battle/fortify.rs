//! Left Stronghold Fortification panel, aligned with the player's HP plate.

use super::super::{
    fortify::{self, Offer, PendingFortify, TowerOpen},
    kit, label, shell, UiAction,
};
use super::{chrome, score::PLATE_TOP};
use crate::{
    sync::{LatestView, MatchInfo},
    GameState,
};
use bevy::prelude::*;
use serde_json::Value;
use tbg_core::{
    content::{ENERGY_SCALE, FORTIFY_COST, STRONGHOLD_KINDS},
    MatchView,
};

#[derive(Component)]
struct Panel;
#[derive(Component)]
struct Icon;
#[derive(Component)]
struct Accent;
#[derive(Component)]
struct Choice(u8);
#[derive(Component)]
enum Field {
    Name,
    Tier,
    Hp,
    PassiveName,
    Passive,
    EvolutionHeading,
    Evolution(u8),
    Note,
}

pub fn plugin(app: &mut App) {
    app.add_systems(OnEnter(GameState::InMatch), spawn)
        .add_systems(
            Update,
            (visibility, readouts, choices, accents).run_if(in_state(GameState::InMatch)),
        );
}

fn spawn(mut commands: Commands, assets: Res<AssetServer>, lang: Res<crate::i18n::Lang>) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: px(super::SIDE_LEFT),
                top: px(PLATE_TOP),
                width: px(252),
                #[cfg(target_os = "android")]
                max_height: percent(65),
                #[cfg(target_os = "android")]
                overflow: Overflow::scroll_y(),
                padding: UiRect::all(px(10)),
                border: UiRect::all(px(2)),
                border_radius: BorderRadius::all(px(7)),
                flex_direction: FlexDirection::Column,
                row_gap: px(6),
                ..default()
            },
            chrome::frame(chrome::GOLD),
            Panel,
            Visibility::Hidden,
            UiAction::Disabled,
            ZIndex(5),
            DespawnOnExit(GameState::InMatch),
        ))
        .with_children(|panel| {
            panel.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    top: px(-2),
                    left: px(0),
                    right: px(0),
                    height: px(3),
                    ..default()
                },
                BackgroundColor(chrome::GOLD),
                Accent,
            ));
            panel
                .spawn(Node {
                    align_items: AlignItems::Center,
                    column_gap: px(8),
                    ..default()
                })
                .with_children(|head| {
                    head.spawn((
                        kit::image(&assets, "battle/native/tower-bastion.webp", 40., 50.),
                        Icon,
                    ));
                    head.spawn(Node {
                        flex_grow: 1.,
                        min_width: px(0),
                        flex_basis: px(0),
                        flex_direction: FlexDirection::Column,
                        row_gap: px(1),
                        ..default()
                    })
                    .with_children(|title| {
                        title.spawn(kit::heading(lang.key("fortify.heading"), 9.));
                        title.spawn((chrome::title("", 17.), Field::Name));
                        title.spawn((
                            kit::muted("", 10.),
                            TextLayout::no_wrap(),
                            Node {
                                max_width: percent(100),
                                overflow: Overflow::clip(),
                                ..default()
                            },
                            Field::Tier,
                        ));
                    });
                    head.spawn((
                        Node {
                            align_self: AlignSelf::Start,
                            width: px(if cfg!(target_os = "android") { 48 } else { 22 }),
                            height: px(if cfg!(target_os = "android") { 48 } else { 22 }),
                            flex_shrink: 0.,
                            border: UiRect::all(px(1)),
                            border_radius: BorderRadius::all(px(4)),
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                            ..default()
                        },
                        BackgroundColor(chrome::INK),
                        BorderColor::all(chrome::GOLD),
                        UiAction::Deselect,
                        kit::buttons::Chrome {
                            enabled: true,
                            focused: false,
                        },
                        kit::buttons::Palette {
                            idle: chrome::INK,
                            hover: Color::srgb_u8(50, 55, 52),
                            pressed: Color::srgb_u8(28, 33, 34),
                            border: Color::srgb_u8(120, 101, 68),
                        },
                        children![label("×", 17.)],
                    ));
                });
            panel
                .spawn((
                    Node {
                        width: percent(100),
                        padding: UiRect::axes(px(6), px(4)),
                        justify_content: JustifyContent::SpaceBetween,
                        border_radius: BorderRadius::all(px(4)),
                        ..default()
                    },
                    BackgroundColor(Color::BLACK.with_alpha(0.45)),
                ))
                .with_children(|hp| {
                    hp.spawn(kit::muted(lang.key("fortify.hp"), 11.));
                    hp.spawn((label("", 11.), Field::Hp));
                });
            panel
                .spawn(Node {
                    width: percent(100),
                    flex_direction: FlexDirection::Column,
                    row_gap: px(1),
                    ..default()
                })
                .with_children(|passive| {
                    passive.spawn((kit::heading("", 10.), Field::PassiveName));
                    passive
                        .spawn(Node {
                            width: percent(100),
                            height: if cfg!(target_os = "android") {
                                Val::Auto
                            } else {
                                px(24)
                            },
                            overflow: Overflow::clip(),
                            ..default()
                        })
                        .with_children(|clip| {
                            clip.spawn((
                                kit::muted("", 10.),
                                bevy::text::LineHeight::Px(12.),
                                Field::Passive,
                            ));
                        });
                });
        });
    commands
        .spawn((
            chrome::upgrade_node(),
            chrome::frame(chrome::GOLD),
            Panel,
            Visibility::Hidden,
            UiAction::Disabled,
            ZIndex(5),
            DespawnOnExit(GameState::InMatch),
        ))
        .with_children(|panel| {
            panel
                .spawn(chrome::upgrade_header())
                .with_children(|heading| {
                    heading.spawn((kit::heading("", 12.), Field::EvolutionHeading));
                    heading.spawn(chrome::upgrade_close(UiAction::Deselect));
                });
            panel
                .spawn(Node {
                    width: percent(100),
                    column_gap: px(6),
                    ..default()
                })
                .with_children(|row| {
                    option(row, 0);
                    option(row, 1);
                });
            panel.spawn((kit::muted("", 10.), Field::Note));
        });
}

fn option(parent: &mut ChildSpawnerCommands, branch: u8) {
    parent.spawn((
        chrome::upgrade_choice(),
        Choice(branch),
        children![(label("", 10.), Field::Evolution(branch))],
    ));
}

fn definition(v: &MatchView) -> Option<&'static Value> {
    let kind = STRONGHOLD_KINDS[fortify::own(v).kind.index()].name;
    shell::stronghold_data()["kinds"]
        .as_array()?
        .iter()
        .find(|def| def["kind"].as_str() == Some(kind))
}

fn visibility(
    view: Res<LatestView>,
    open: Res<TowerOpen>,
    info: Res<MatchInfo>,
    mut panels: Query<&mut Visibility, With<Panel>>,
) {
    let show = open.0 && !info.spectating && view.0.as_ref().and_then(definition).is_some();
    for mut visible in &mut panels {
        *visible = if show {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
}

fn readouts(
    lang: Res<crate::i18n::Lang>,
    view: Res<LatestView>,
    pending: Res<PendingFortify>,
    assets: Res<AssetServer>,
    mut texts: Query<(&Field, &mut Text, &mut TextColor, &mut Node)>,
    mut icons: Query<&mut ImageNode, With<Icon>>,
) {
    if !view.is_changed() && !pending.is_changed() && !lang.is_changed() {
        return;
    }
    let Some(v) = &view.0 else { return };
    let Some(def) = definition(v) else { return };
    let color = accent(v);
    for (field, mut text, mut tint, mut node) in &mut texts {
        let want = field_text(field, v, def, pending.0.is_some(), *lang);
        if text.0 != want {
            text.0 = want;
        }
        tint.0 = field_color(field, color);
        if matches!(field, Field::Note) {
            node.display = if text.0.is_empty() {
                Display::None
            } else {
                Display::Flex
            };
        }
    }
    let path = format!(
        "battle/native/tower-{}.webp",
        STRONGHOLD_KINDS[fortify::own(v).kind.index()]
            .name
            .to_ascii_lowercase()
    );
    for mut image in &mut icons {
        image.image = assets.load(path.clone());
    }
}

fn field_text(
    field: &Field,
    v: &MatchView,
    def: &Value,
    pending: bool,
    lang: crate::i18n::Lang,
) -> String {
    let tower = fortify::own(v);
    let branch = usize::from(tower.branch.min(1));
    let tier = usize::from(tower.tier.saturating_sub(1).min(1));
    let stats = if tower.tier == 0 {
        &def["stats"]
    } else {
        &def["branches"][branch]["steps"][tier]["stats"]
    };
    match field {
        Field::Name => crate::i18n::content::stronghold_name(lang, tower.kind).into(),
        Field::Tier => format!(
            "{}{}",
            lang.key("fortify.tier")
                .replace("{tier}", &tower.tier.to_string()),
            if tower.tier == 0 {
                String::new()
            } else {
                format!(
                    " · {}",
                    crate::i18n::content::stronghold_branch(lang, tower.kind, branch as u8)
                )
            }
        ),
        Field::Hp => format!(
            "{} / {}",
            v.stronghold_hp[v.you.index()].max(0),
            tower.max_hp
        ),
        Field::PassiveName => lang.key(&format!(
            "stronghold.{}.passive.name",
            format!("{:?}", tower.kind).to_lowercase()
        )),
        Field::Passive => shell::passive(lang, tower.kind, stats),
        Field::EvolutionHeading => format!(
            "{} · {}",
            crate::i18n::content::stronghold_name(lang, tower.kind),
            lang.key("fortify.heading")
        ),
        Field::Note => note(v, pending, lang),
        Field::Evolution(branch) => evolution_text(v, def, *branch, lang),
    }
}

fn evolution_text(v: &MatchView, def: &Value, branch: u8, lang: crate::i18n::Lang) -> String {
    let tower = fortify::own(v);
    if tower.tier >= 2 || (tower.tier == 1 && branch != tower.branch) {
        return String::new();
    }
    let tier = usize::from(tower.tier);
    let step = &def["branches"][usize::from(branch)]["steps"][tier];
    format!(
        "{} · {} {}\n{}\n{}",
        crate::i18n::content::stronghold_branch(lang, tower.kind, branch),
        FORTIFY_COST[tier] / ENERGY_SCALE,
        lang.key("mobile.energy"),
        lang.tf(
            crate::i18n::Msg::FortifyStep,
            &[
                ("tier", &(tier + 1)),
                (
                    "step",
                    &crate::i18n::content::stronghold_step(
                        lang,
                        tower.kind,
                        branch,
                        (tier + 1) as u8
                    )
                ),
            ]
        ),
        shell::branch_effect(step, lang),
    )
}

fn note(v: &MatchView, pending: bool, lang: crate::i18n::Lang) -> String {
    let tower = fortify::own(v);
    let branch = crate::i18n::content::stronghold_branch(lang, tower.kind, tower.branch.min(1));
    match fortify::offer(v, tower.branch.min(1), pending) {
        Offer::Max => lang.key("fortify.note.full").replace("{branch}", &branch),
        Offer::Pending => lang.key("fortify.note.pending"),
        Offer::Paused => lang.key("fortify.note.paused"),
        Offer::Idle => lang.key("fortify.note.idle"),
        Offer::Short => lang
            .key("fortify.note.short")
            .replace(
                "{cost}",
                &(FORTIFY_COST[usize::from(tower.tier.min(1))] / ENERGY_SCALE).to_string(),
            )
            .replace(
                "{have}",
                &lang.decimal(v.energy as f64 / ENERGY_SCALE as f64, 1),
            ),
        _ => String::new(),
    }
}

fn field_color(field: &Field, color: Color) -> Color {
    match field {
        Field::Name => color,
        Field::PassiveName | Field::Note => chrome::GOLD,
        Field::EvolutionHeading => kit::GOLD,
        Field::Evolution(_) => super::super::TEXT,
        Field::Hp => chrome::CREAM,
        _ => kit::MUTED,
    }
}

fn choices(
    view: Res<LatestView>,
    pending: Res<PendingFortify>,
    mut choices: Query<(&Choice, &mut Node, &mut UiAction, &mut kit::buttons::Chrome)>,
) {
    let Some(v) = &view.0 else { return };
    for (choice, mut node, mut action, mut chrome) in &mut choices {
        let state = fortify::offer(v, choice.0, pending.0.is_some());
        node.display = if matches!(state, Offer::Max | Offer::Closed) {
            Display::None
        } else {
            Display::Flex
        };
        chrome.enabled = state == Offer::Open;
        *action = if chrome.enabled {
            UiAction::Fortify(choice.0)
        } else {
            UiAction::Disabled
        };
    }
}

fn accent(v: &MatchView) -> Color {
    use tbg_core::StrongholdKind::*;
    let [r, g, b] = match fortify::own(v).kind {
        Treasury => [227, 181, 74],
        Command => [224, 87, 90],
        Artillery => [233, 138, 79],
        _ => [127, 176, 220],
    };
    Color::srgb_u8(r, g, b)
}

fn accents(view: Res<LatestView>, mut bars: Query<&mut BackgroundColor, With<Accent>>) {
    let Some(v) = &view.0 else { return };
    for mut bar in &mut bars {
        bar.0 = accent(v);
    }
}
