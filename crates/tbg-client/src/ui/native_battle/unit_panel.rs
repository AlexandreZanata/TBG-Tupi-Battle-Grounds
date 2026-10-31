//! Bottom-left champion status and Evolution controls using the existing reports.

use super::super::{
    evolve::{self, Selected},
    inspect::{self, Inspected, Report},
    kit, label, UiAction,
};
use super::{chrome, SHOP_BOTTOM, SIDE_LEFT};
use crate::i18n::Lang;
use crate::{
    sync::{LatestView, MatchInfo},
    GameState,
};
use bevy::prelude::*;
use tbg_core::{
    content::{form_of, ENERGY_SCALE},
    MatchView,
};

#[derive(Component)]
struct Panel;
#[derive(Component)]
struct Portrait;
#[derive(Component)]
struct Health;
#[derive(Component)]
struct Accent;
#[derive(Component)]
struct Upgrades;
#[derive(Component)]
struct Choice(u8);
#[derive(Clone, Copy)]
enum Stat {
    Attack,
    PerSecond,
    Every,
    Range,
    Speed,
    Splash,
    OnHit,
    Status,
    Synergy,
}
#[derive(Component)]
enum Field {
    Name,
    Form,
    Tags,
    Hp,
    AttackLabel,
    Stat(Stat),
    Evolution(u8),
}

pub fn plugin(app: &mut App) {
    app.add_systems(OnEnter(GameState::InMatch), spawn)
        .add_systems(
            Update,
            (update, upgrade_visibility, choices).run_if(in_state(GameState::InMatch)),
        );
}

fn spawn(mut commands: Commands, assets: Res<AssetServer>) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: px(SIDE_LEFT),
                bottom: px(SHOP_BOTTOM),
                width: px(278),
                padding: UiRect::all(px(12)),
                border: UiRect::all(px(1)),
                border_radius: BorderRadius::all(px(6)),
                flex_direction: FlexDirection::Column,
                row_gap: px(7),
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
                    left: px(-1),
                    top: px(0),
                    bottom: px(0),
                    width: px(3),
                    border_radius: BorderRadius::all(px(3)),
                    ..default()
                },
                BackgroundColor(Color::srgb_u8(47, 180, 255)),
                Accent,
            ));
            header(panel, &assets);
            panel
                .spawn((
                    Node {
                        width: percent(100),
                        height: px(18),
                        border: UiRect::all(px(1)),
                        border_radius: BorderRadius::all(px(3)),
                        overflow: Overflow::clip(),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    BackgroundColor(chrome::INK),
                    BorderColor::all(chrome::GOLD),
                ))
                .with_children(|hp| {
                    hp.spawn((
                        Node {
                            position_type: PositionType::Absolute,
                            left: px(0),
                            width: percent(100),
                            height: percent(100),
                            ..default()
                        },
                        chrome::gradient(
                            Color::srgb_u8(95, 212, 255),
                            Color::srgb_u8(10, 111, 180),
                        ),
                        Health,
                    ));
                    hp.spawn((
                        label("", 11.),
                        TextShadow {
                            offset: Vec2::new(0., 1.),
                            color: Color::BLACK,
                        },
                        Field::Hp,
                    ));
                });
            panel
                .spawn(Node {
                    width: percent(100),
                    flex_wrap: FlexWrap::Wrap,
                    column_gap: px(10),
                    row_gap: px(4),
                    ..default()
                })
                .with_children(|stats| {
                    for (name, kind) in [
                        ("Damage", Stat::Attack),
                        ("Per second", Stat::PerSecond),
                        ("Attack every", Stat::Every),
                        ("Range", Stat::Range),
                        ("Speed", Stat::Speed),
                        ("Splash", Stat::Splash),
                        ("On hit", Stat::OnHit),
                        ("Status", Stat::Status),
                        ("Synergy", Stat::Synergy),
                    ] {
                        stat(stats, name, kind);
                    }
                });
            panel
                .spawn((
                    Node {
                        width: percent(100),
                        flex_direction: FlexDirection::Column,
                        row_gap: px(5),
                        ..default()
                    },
                    Upgrades,
                ))
                .with_children(|options| {
                    options
                        .spawn(Node {
                            width: percent(100),
                            justify_content: JustifyContent::SpaceBetween,
                            align_items: AlignItems::Center,
                            ..default()
                        })
                        .with_children(|heading| {
                            heading.spawn(kit::heading("Evolution", 12.));
                            heading.spawn((
                                Node {
                                    width: px(20),
                                    height: px(20),
                                    align_items: AlignItems::Center,
                                    justify_content: JustifyContent::Center,
                                    ..default()
                                },
                                UiAction::CloseEvolve,
                                children![kit::heading("×", 14.)],
                            ));
                        });
                    for branch in 0..2 {
                        options.spawn((
                            Node {
                                width: percent(100),
                                min_height: px(42),
                                padding: UiRect::all(px(7)),
                                border: UiRect::all(px(1)),
                                border_radius: BorderRadius::all(px(5)),
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            BackgroundColor(chrome::INK),
                            BorderColor::all(chrome::GOLD),
                            UiAction::Disabled,
                            Choice(branch),
                            kit::buttons::Chrome {
                                enabled: false,
                                focused: false,
                            },
                            kit::buttons::Palette {
                                idle: chrome::INK,
                                hover: Color::srgb_u8(45, 57, 60),
                                pressed: Color::srgb_u8(28, 38, 43),
                                border: chrome::GOLD,
                            },
                            children![(label("", 10.), Field::Evolution(branch))],
                        ));
                    }
                });
        });
}

fn header(parent: &mut ChildSpawnerCommands, assets: &AssetServer) {
    parent
        .spawn(Node {
            column_gap: px(9),
            align_items: AlignItems::Center,
            ..default()
        })
        .with_children(|head| {
            head.spawn((
                Node {
                    width: px(50),
                    height: px(50),
                    flex_shrink: 0.,
                    border: UiRect::all(px(1)),
                    border_radius: BorderRadius::all(px(5)),
                    overflow: Overflow::clip(),
                    ..default()
                },
                ImageNode::new(assets.load("battle/portraits/hexer.webp")),
                BorderColor::all(chrome::GOLD),
                Portrait,
            ));
            head.spawn(Node {
                flex_grow: 1.,
                flex_basis: px(0),
                min_width: px(0),
                flex_direction: FlexDirection::Column,
                row_gap: px(2),
                ..default()
            })
            .with_children(|copy| {
                copy.spawn(Node {
                    width: percent(100),
                    height: px(21),
                    overflow: Overflow::clip(),
                    ..default()
                })
                .with_children(|clip| {
                    clip.spawn((chrome::title("", 17.), TextLayout::no_wrap(), Field::Name));
                });
                copy.spawn((kit::heading("", 11.), Field::Form));
                copy.spawn((kit::muted("", 10.), Field::Tags));
            });
            head.spawn((
                Node {
                    width: px(20),
                    height: px(20),
                    align_self: AlignSelf::Start,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                UiAction::CloseInspect,
                children![kit::heading("×", 16.)],
            ));
        });
}

fn stat(parent: &mut ChildSpawnerCommands, name: &str, kind: Stat) {
    parent
        .spawn((
            Node {
                width: percent(if matches!(kind, Stat::Synergy) {
                    100.
                } else {
                    48.
                }),
                justify_content: JustifyContent::SpaceBetween,
                padding: UiRect::bottom(px(3)),
                border: UiRect::bottom(px(1)),
                column_gap: px(4),
                ..default()
            },
            BorderColor::all(Color::WHITE.with_alpha(0.09)),
        ))
        .with_children(|cell| {
            let mut title = cell.spawn(kit::muted(name.to_owned(), 10.));
            if matches!(kind, Stat::Attack) {
                title.insert(Field::AttackLabel);
            }
            cell.spawn((label("", 10.), Field::Stat(kind)));
        });
}

fn current(
    view: &LatestView,
    inspected: &Inspected,
    info: &MatchInfo,
    lang: Lang,
) -> Option<Report> {
    inspect::report(view.0.as_ref()?, inspected.0?, &info.arcanas, lang)
}

#[allow(clippy::too_many_arguments)] // Bevy declares independent resources and component filters in the system signature.
fn update(
    view: Res<LatestView>,
    inspected: Res<Inspected>,
    info: Res<MatchInfo>,
    lang: Res<Lang>,
    assets: Res<AssetServer>,
    mut panels: Query<&mut Visibility, With<Panel>>,
    mut texts: Query<(&Field, &mut Text)>,
    mut hp: Query<(&mut Node, &mut BackgroundGradient), With<Health>>,
    mut portraits: Query<&mut ImageNode, With<Portrait>>,
    mut accents: Query<&mut BackgroundColor, With<Accent>>,
) {
    let report = current(&view, &inspected, &info, *lang);
    for mut visible in &mut panels {
        *visible = if report.is_some() {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
    let Some(r) = report else { return };
    for (field, mut text) in &mut texts {
        text.0 = field_text(field, &r, view.0.as_ref(), inspected.0, *lang);
    }
    let (top, bottom) = if r.own {
        (Color::srgb_u8(95, 212, 255), Color::srgb_u8(10, 111, 180))
    } else {
        (Color::srgb_u8(255, 128, 133), Color::srgb_u8(180, 20, 44))
    };
    for (mut node, mut gradient) in &mut hp {
        node.width = percent(100. * (r.hp as f32 / r.max_hp as f32).clamp(0., 1.));
        *gradient = chrome::gradient(top, bottom);
    }
    for mut image in &mut portraits {
        image.image = assets.load(format!(
            "battle/portraits/{}.webp",
            r.kind.to_ascii_lowercase()
        ));
    }
    for mut color in &mut accents {
        color.0 = if r.own {
            Color::srgb_u8(47, 180, 255)
        } else {
            Color::srgb_u8(255, 65, 87)
        };
    }
}

fn field_text(
    field: &Field,
    r: &Report,
    view: Option<&MatchView>,
    id: Option<tbg_core::UnitId>,
    lang: Lang,
) -> String {
    match field {
        Field::Name => r.name.into(),
        Field::Form => r.form.clone(),
        Field::Tags => format!(
            "{} · {} · {}",
            if r.own { "Yours" } else { "Enemy" },
            r.faction,
            r.role
        ),
        Field::Hp => format!("{} / {}", r.hp, r.max_hp),
        Field::AttackLabel => r.attack.0.into(),
        Field::Stat(kind) => stat_text(*kind, r),
        Field::Evolution(branch) => evolution_text(view, id, *branch, lang),
    }
}

fn stat_text(kind: Stat, r: &Report) -> String {
    match kind {
        Stat::Attack => r.attack.1.to_string(),
        Stat::PerSecond => r.per_second.clone(),
        Stat::Every => r.every.clone(),
        Stat::Range => format!("{} tiles", r.range),
        Stat::Speed => format!("{} tiles/s", r.speed),
        Stat::Splash => {
            if r.splash == "-" {
                r.splash.clone()
            } else {
                format!("{} tiles", r.splash)
            }
        }
        Stat::OnHit => r.on_hit.clone(),
        Stat::Status => r.status_label.into(),
        Stat::Synergy => r.synergy.clone(),
    }
}

fn evolution_text(
    view: Option<&MatchView>,
    id: Option<tbg_core::UnitId>,
    branch: u8,
    lang: Lang,
) -> String {
    let Some((v, u)) = view.and_then(|v| Some((v, v.units.iter().find(|u| Some(u.id) == id)?)))
    else {
        return String::new();
    };
    let Some(form) = form_of(u.kind, u.tier.saturating_add(1), branch) else {
        return String::new();
    };
    format!(
        "{} · {} Energy\n{}",
        form.name,
        v.evolve_costs[usize::from(u.tier.min(1))] / ENERGY_SCALE,
        evolve::describe(form, lang)
    )
}

fn upgrade_visibility(
    view: Res<LatestView>,
    inspected: Res<Inspected>,
    selected: Res<Selected>,
    mut roots: Query<&mut Node, With<Upgrades>>,
) {
    let show = view.0.as_ref().is_some_and(|v| {
        v.units.iter().any(|u| {
            Some(u.id) == inspected.0
                && Some(u.id) == selected.0
                && u.seat == v.you
                && !u.is_summon()
                && u.tier < 2
        })
    });
    for mut node in &mut roots {
        node.display = if show { Display::Flex } else { Display::None };
    }
}

fn choices(
    view: Res<LatestView>,
    selected: Res<Selected>,
    mut choices: Query<(&Choice, &mut Node, &mut UiAction, &mut kit::buttons::Chrome)>,
) {
    let Some(v) = &view.0 else { return };
    let unit = selected
        .0
        .and_then(|id| v.units.iter().find(|u| u.id == id));
    for (choice, mut node, mut action, mut chrome) in &mut choices {
        let show = unit.is_some_and(|u| u.tier == 0 || (u.tier == 1 && u.branch == choice.0));
        node.display = if show { Display::Flex } else { Display::None };
        *action = match unit {
            Some(u) if v.augment_round.is_none() => evolve::choice_action(v, u.id, choice.0),
            _ => UiAction::Disabled,
        };
        chrome.enabled = *action != UiAction::Disabled;
    }
}
