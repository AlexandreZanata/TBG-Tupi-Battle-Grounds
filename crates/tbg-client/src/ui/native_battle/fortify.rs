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
    Branch(u8),
    Step(u8),
    Effect(u8),
    Cost(u8),
    Note,
}

pub fn plugin(app: &mut App) {
    app.add_systems(OnEnter(GameState::InMatch), spawn)
        .add_systems(
            Update,
            (visibility, readouts, choices, accents).run_if(in_state(GameState::InMatch)),
        );
}

fn spawn(mut commands: Commands, assets: Res<AssetServer>) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: px(16),
                top: px(PLATE_TOP),
                width: px(252),
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
                        title.spawn(kit::heading("FORTIFICATION", 9.));
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
                            width: px(22),
                            height: px(22),
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
                    hp.spawn(kit::muted("Stronghold HP", 11.));
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
                            height: px(24),
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
            option(panel, 0);
            option(panel, 1);
            panel.spawn((
                kit::heading("", 10.),
                Node {
                    width: percent(100),
                    min_height: px(14),
                    ..default()
                },
                Field::Note,
            ));
        });
}

fn option(parent: &mut ChildSpawnerCommands, branch: u8) {
    parent
        .spawn((
            Node {
                width: percent(100),
                min_height: px(55),
                padding: UiRect::all(px(7)),
                border: UiRect::all(px(1)),
                border_radius: BorderRadius::all(px(6)),
                align_items: AlignItems::Center,
                column_gap: px(7),
                ..default()
            },
            BackgroundColor(chrome::INK),
            BorderColor::all(chrome::GOLD),
            Choice(branch),
            UiAction::Disabled,
            kit::buttons::Chrome {
                enabled: false,
                focused: false,
            },
            kit::buttons::Palette {
                idle: chrome::INK,
                hover: Color::srgb_u8(65, 57, 39),
                pressed: Color::srgb_u8(35, 31, 24),
                border: chrome::GOLD,
            },
        ))
        .with_children(|button| {
            button.spawn((
                Node {
                    width: px(20),
                    height: px(20),
                    flex_shrink: 0.,
                    border_radius: BorderRadius::all(percent(50)),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                BackgroundColor(chrome::GOLD),
                Accent,
                children![(
                    Text::new(if branch == 0 { "A" } else { "B" }),
                    TextFont::from_font_size(FontSize::Px(11.)),
                    TextColor(chrome::INK)
                )],
            ));
            button
                .spawn(Node {
                    flex_grow: 1.,
                    flex_basis: px(0),
                    min_width: px(0),
                    flex_direction: FlexDirection::Column,
                    row_gap: px(1),
                    ..default()
                })
                .with_children(|copy| {
                    copy.spawn((label("", 11.), Field::Branch(branch)));
                    copy.spawn((kit::muted("", 10.), Field::Step(branch)));
                    copy.spawn((kit::muted("", 9.5), Field::Effect(branch)));
                });
            button
                .spawn((
                    Node {
                        min_width: px(39),
                        height: px(25),
                        flex_shrink: 0.,
                        padding: UiRect::horizontal(px(5)),
                        border: UiRect::all(px(1)),
                        border_radius: BorderRadius::all(px(4)),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        column_gap: px(5),
                        ..default()
                    },
                    BackgroundColor(Color::srgb_u8(48, 39, 31)),
                    BorderColor::all(chrome::GOLD),
                ))
                .with_children(|price| {
                    price.spawn((
                        Node {
                            width: px(11),
                            height: px(11),
                            border: UiRect::all(px(2)),
                            border_radius: BorderRadius::all(percent(50)),
                            ..default()
                        },
                        BorderColor::all(chrome::CREAM),
                        chrome::gradient(
                            Color::srgb_u8(255, 233, 137),
                            Color::srgb_u8(181, 115, 13),
                        ),
                    ));
                    price.spawn((label("", 13.), Field::Cost(branch)));
                });
        });
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
    view: Res<LatestView>,
    pending: Res<PendingFortify>,
    assets: Res<AssetServer>,
    mut texts: Query<(&Field, &mut Text, &mut TextColor)>,
    mut icons: Query<&mut ImageNode, With<Icon>>,
) {
    if !view.is_changed() && !pending.is_changed() {
        return;
    }
    let Some(v) = &view.0 else { return };
    let Some(def) = definition(v) else { return };
    let color = accent(v);
    for (field, mut text, mut tint) in &mut texts {
        let want = field_text(field, v, def, pending.0.is_some());
        if text.0 != want {
            text.0 = want;
        }
        tint.0 = field_color(field, v, pending.0.is_some(), color);
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

fn field_text(field: &Field, v: &MatchView, def: &Value, pending: bool) -> String {
    let tower = fortify::own(v);
    let branch = usize::from(tower.branch.min(1));
    let tier = usize::from(tower.tier.saturating_sub(1).min(1));
    let stats = if tower.tier == 0 {
        &def["stats"]
    } else {
        &def["branches"][branch]["steps"][tier]["stats"]
    };
    match field {
        Field::Name => STRONGHOLD_KINDS[tower.kind.index()].name.into(),
        Field::Tier => format!(
            "Tier {} / 2{}",
            tower.tier,
            if tower.tier == 0 {
                String::new()
            } else {
                format!(" · {}", value(&def["branches"][branch], "name"))
            }
        ),
        Field::Hp => format!(
            "{} / {}",
            v.stronghold_hp[v.you.index()].max(0),
            tower.max_hp
        ),
        Field::PassiveName => value(&def["passive"], "name"),
        Field::Passive => shell::expand(&value(&def["passive"], "text"), stats),
        Field::Note => note(v, pending),
        _ => option_text(field, v, def, pending),
    }
}

fn option_text(field: &Field, v: &MatchView, def: &Value, pending: bool) -> String {
    let (Field::Branch(branch) | Field::Step(branch) | Field::Effect(branch) | Field::Cost(branch)) =
        field
    else {
        return String::new();
    };
    let tower = fortify::own(v);
    let path = &def["branches"][usize::from(*branch)];
    let tier = usize::from(tower.tier.min(1));
    let step = &path["steps"][tier];
    let closed = fortify::offer(v, *branch, pending) == Offer::Closed;
    match field {
        Field::Branch(_) => value(path, "name"),
        Field::Step(_) if closed => format!(
            "Closed · {} chosen",
            value(&def["branches"][usize::from(tower.branch.min(1))], "name")
        ),
        Field::Step(_) => format!("Tier {} · {}", tier + 1, value(step, "name")),
        Field::Effect(_) if !closed => shell::branch_effect(step),
        Field::Cost(_) if !closed => (FORTIFY_COST[tier] / ENERGY_SCALE).to_string(),
        Field::Cost(_) => "–".into(),
        _ => String::new(),
    }
}

fn value(value: &Value, key: &str) -> String {
    value[key].as_str().unwrap_or_default().into()
}

fn note(v: &MatchView, pending: bool) -> String {
    let tower = fortify::own(v);
    let branch =
        STRONGHOLD_KINDS[tower.kind.index()].branch_names[usize::from(tower.branch.min(1))];
    match fortify::offer(v, tower.branch.min(1), pending) {
        Offer::Max => format!("Fully fortified · {branch}"),
        Offer::Pending => "Fortifying…".into(),
        Offer::Paused => "Battle paused · War Council first".into(),
        Offer::Idle => "Opens when the battle starts".into(),
        Offer::Short => format!(
            "Need {} Energy · you have {:.1}",
            FORTIFY_COST[usize::from(tower.tier.min(1))] / ENERGY_SCALE,
            v.energy as f32 / ENERGY_SCALE as f32
        ),
        _ if tower.tier == 0 => "Tier 1 picks a branch for the rest of the match".into(),
        _ => format!("Tier 2 completes the {branch} branch"),
    }
}

fn field_color(field: &Field, v: &MatchView, pending: bool, color: Color) -> Color {
    match field {
        Field::Name => color,
        Field::PassiveName | Field::Note => chrome::GOLD,
        Field::Cost(branch) if fortify::offer(v, *branch, pending) == Offer::Short => {
            Color::srgb_u8(255, 179, 163)
        }
        Field::Hp | Field::Cost(_) => chrome::CREAM,
        Field::Branch(branch) | Field::Step(branch)
            if fortify::offer(v, *branch, pending) == Offer::Open =>
        {
            chrome::CREAM
        }
        _ => kit::MUTED,
    }
}

fn choices(
    view: Res<LatestView>,
    pending: Res<PendingFortify>,
    mut choices: Query<(
        &Choice,
        &mut Node,
        &mut UiAction,
        &mut kit::buttons::Chrome,
        &mut kit::buttons::Palette,
    )>,
) {
    let Some(v) = &view.0 else { return };
    for (choice, mut node, mut action, mut chrome, mut palette) in &mut choices {
        let state = fortify::offer(v, choice.0, pending.0.is_some());
        node.display = if state == Offer::Max {
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
        palette.idle = if chrome.enabled {
            Color::srgb_u8(48, 42, 30)
        } else {
            Color::srgb_u8(27, 27, 25)
        };
        palette.border = if state == Offer::Closed {
            kit::LINE
        } else {
            Color::srgb_u8(120, 101, 68)
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
