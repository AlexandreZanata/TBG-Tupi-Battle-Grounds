//! Common Arena/side chip and dismissible five-second match introduction.

use super::{
    super::{kit, label, ActionQueue, UiAction},
    chrome,
};
use crate::{i18n::Lang, sync::MatchInfo, GameState};
use bevy::prelude::*;
use tbg_core::Arena;

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
    let (side_name, side_description, side_color) = side_copy(*lang, &info);
    let name = lang.key(&format!("arenas.{}.name", info.arena.slug()));
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: px(if cfg!(target_os = "android") { 58 } else { 92 }),
                width: percent(100),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            DespawnOnExit(GameState::InMatch),
            ZIndex(3),
            #[cfg(target_os = "android")]
            crate::camera::mobile::AboveBattle,
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
                chip.spawn(chrome::icon(&assets, "island", 14., accent));
                chip.spawn(label(name.clone(), 14.));
                chip.spawn(kit::muted("·", 13.));
                chip.spawn(label(side_name.clone(), 13.))
                    .insert(TextColor(side_color));
            });
        });
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: px(if cfg!(target_os = "android") { 90 } else { 134 }),
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
                    min_height: px(146),
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
                banner
                    .spawn(label(side_name.clone(), 17.))
                    .insert(TextColor(side_color));
                banner.spawn((
                    label(side_description.clone(), 12.),
                    TextLayout::justify(Justify::Center),
                ));
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
    let mut focus_arena = false;
    queue.0.retain(|action| match action {
        UiAction::ShowArenaBanner => {
            state.show(time.elapsed_secs_f64());
            focus_arena = true;
            false
        }
        UiAction::CloseArenaBanner => {
            state.dismiss();
            false
        }
        UiAction::Disabled => true,
        _ => {
            state.dismiss();
            true
        }
    });
    if focus_arena {
        queue.0.push(UiAction::Deselect);
    }
}

fn visibility(
    time: Res<Time<Real>>,
    state: Res<ArenaBanner>,
    selected: Res<super::super::evolve::Selected>,
    tower: Res<super::super::fortify::TowerOpen>,
    mut banners: Query<&mut Visibility, With<Banner>>,
) {
    for mut visible in &mut banners {
        *visible = if time.elapsed_secs_f64() < state.until && selected.0.is_none() && !tower.0 {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
}

fn effect(lang: Lang, arena: Arena) -> String {
    crate::ui::arena_effect::describe(arena, lang)
}

fn accent(arena: Arena) -> Color {
    let [r, g, b] = match arena {
        Arena::Forest => [123, 182, 97],
        Arena::Tundra => [159, 211, 236],
        Arena::Volcanic => [255, 138, 61],
        Arena::Ruins => [217, 180, 74],
        Arena::Storm => [143, 168, 255],
        Arena::Celestial => [108, 211, 241],
        Arena::Clockwork => [223, 157, 73],
        Arena::Desert => [231, 192, 111],
    };
    Color::srgb_u8(r, g, b)
}

fn side_copy(lang: Lang, info: &MatchInfo) -> (String, String, Color) {
    if info.spectating {
        return (
            lang.key("battle.side.spectator"),
            lang.key("battle.side.common"),
            chrome::GOLD,
        );
    }
    let blue = info.you.unwrap_or(tbg_core::Seat::A) == tbg_core::Seat::A;
    let key = if blue { "blue" } else { "red" };
    (
        lang.key(&format!("battle.side.{key}")),
        lang.key(&format!("battle.side.{key}_description")),
        if blue {
            crate::art::palette::ALLY
        } else {
            crate::art::palette::ENEMY
        },
    )
}

#[cfg(test)]
mod side_tests {
    use super::*;

    #[test]
    fn side01_intro_identifies_the_real_side_and_neutral_spectator() {
        for (side, expected) in [
            (tbg_core::Seat::A, "Lado azul"),
            (tbg_core::Seat::B, "Lado vermelho"),
        ] {
            let info = MatchInfo {
                you: Some(side),
                ..default()
            };
            let (title, description, _) = side_copy(Lang::PtBr, &info);
            assert_eq!(title, expected);
            assert!(description.contains(if side == tbg_core::Seat::A {
                "fica à esquerda"
            } else {
                "fica à direita"
            }));
            let watch = MatchInfo {
                spectating: true,
                ..info
            };
            assert_eq!(side_copy(Lang::PtBr, &watch).0, "Azul contra Vermelho");
        }
    }
}
