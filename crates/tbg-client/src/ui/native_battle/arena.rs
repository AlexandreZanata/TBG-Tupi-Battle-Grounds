//! Persistent arena chip and dismissible five-second introduction per Linux match.

use super::{
    super::{kit, label, ActionQueue, UiAction},
    chrome,
};
use crate::{i18n::Lang, sync::MatchInfo, GameState};
use bevy::prelude::*;
use tbg_core::{
    content::{ARENA_RULES, ENERGY_SCALE},
    Arena,
};

#[derive(Resource, Default)]
pub struct ArenaBanner {
    until: f64,
}
impl ArenaBanner {
    pub fn show(&mut self, now: f64) {
        self.until = now + 5.;
    }
    pub fn dismiss(&mut self) {
        self.until = 0.;
    }
}
#[derive(Component)]
struct Banner;

pub fn plugin(app: &mut App) {
    app.init_resource::<ArenaBanner>()
        .add_systems(OnEnter(GameState::InMatch), spawn)
        .add_systems(
            Update,
            controls
                .after(super::super::keyboard::navigate)
                .before(super::super::actions::apply)
                .run_if(in_state(GameState::InMatch)),
        )
        .add_systems(
            Update,
            visibility
                .after(controls)
                .run_if(in_state(GameState::InMatch)),
        );
}

fn spawn(
    mut commands: Commands,
    info: Res<MatchInfo>,
    lang: Res<Lang>,
    time: Res<Time<Real>>,
    assets: Res<AssetServer>,
    mut state: ResMut<ArenaBanner>,
) {
    state.show(time.elapsed_secs_f64());
    let accent = accent(info.arena);
    let name = lang.key(&format!("arenas.{}.name", info.arena.slug()));
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: px(92),
                width: percent(100),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            DespawnOnExit(GameState::InMatch),
            ZIndex(3),
        ))
        .with_children(|root| {
            root.spawn((
                Node {
                    height: px(27),
                    padding: UiRect::axes(px(10), px(3)),
                    border: UiRect::all(px(1)),
                    border_radius: BorderRadius::all(px(12)),
                    column_gap: px(6),
                    align_items: AlignItems::Center,
                    ..default()
                },
                BackgroundColor(chrome::INK.with_alpha(0.9)),
                BorderColor::all(accent.with_alpha(0.6)),
                UiAction::ShowArenaBanner,
            ))
            .with_children(|chip| {
                chip.spawn(chrome::icon(&assets, "arenas", 14., accent));
                chip.spawn(label(name.clone(), 14.));
            });
        });
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: px(134),
                width: percent(100),
                justify_content: JustifyContent::Center,
                ..default()
            },
            Banner,
            DespawnOnExit(GameState::InMatch),
            ZIndex(4),
        ))
        .with_children(|root| {
            root.spawn((
                Node {
                    width: px(390),
                    min_height: px(106),
                    padding: UiRect::axes(px(24), px(12)),
                    border: UiRect::all(px(1)),
                    border_radius: BorderRadius::all(px(10)),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    row_gap: px(2),
                    ..default()
                },
                chrome::frame(accent.with_alpha(0.7)),
                UiAction::Disabled,
            ))
            .with_children(|banner| {
                banner.spawn(kit::muted(
                    lang.key("arenas.banner.label").to_uppercase(),
                    9.,
                ));
                banner.spawn((
                    Text::new(name),
                    TextFont::from_font_size(FontSize::Px(28.)).with_font(kit::display_font()),
                    TextColor(accent),
                ));
                banner.spawn((
                    label(effect(*lang, info.arena), 13.),
                    TextLayout::justify(Justify::Center),
                ));
                banner.spawn(kit::muted(lang.key("arenas.symmetric"), 10.));
                banner.spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        top: px(3),
                        right: px(5),
                        width: px(24),
                        height: px(24),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    UiAction::CloseArenaBanner,
                    children![label("×", 22.)],
                ));
            });
        });
}

fn controls(time: Res<Time<Real>>, mut state: ResMut<ArenaBanner>, mut queue: ResMut<ActionQueue>) {
    queue.0.retain(|action| match action {
        UiAction::ShowArenaBanner => {
            state.show(time.elapsed_secs_f64());
            false
        }
        UiAction::CloseArenaBanner => {
            state.dismiss();
            false
        }
        _ => true,
    });
}

fn visibility(
    time: Res<Time<Real>>,
    state: Res<ArenaBanner>,
    mut banners: Query<&mut Visibility, With<Banner>>,
) {
    for mut visible in &mut banners {
        *visible = if time.elapsed_secs_f64() < state.until {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
}

fn effect(lang: Lang, arena: Arena) -> String {
    let rule = ARENA_RULES[arena.index()];
    let (key, value) = match arena {
        Arena::Forest => return lang.key("arenas.effect.none"),
        Arena::Tundra => ("arenas.effect.move", format!("{:+}%", rule.move_bp / 100)),
        Arena::Volcanic => ("arenas.effect.attack", format!("{:+}%", rule.atk_bp / 100)),
        Arena::Ruins => (
            "arenas.effect.evolve",
            (rule.first_evolve_discount / ENERGY_SCALE).to_string(),
        ),
        Arena::Storm => ("arenas.effect.regen", format!("{:+}%", rule.regen_bp / 100)),
    };
    lang.key(key).replace("{value}", &value)
}

fn accent(arena: Arena) -> Color {
    let [r, g, b] = match arena {
        Arena::Forest => [123, 182, 97],
        Arena::Tundra => [159, 211, 236],
        Arena::Volcanic => [255, 138, 61],
        Arena::Ruins => [217, 180, 74],
        Arena::Storm => [143, 168, 255],
    };
    Color::srgb_u8(r, g, b)
}
