//! Illustrated Linux War Council with local selection and authoritative confirmation.

use super::{
    super::{kit, label, ActionQueue, UiAction},
    chrome,
};
use crate::{
    sync::{LatestView, MatchInfo},
    GameState,
};
use bevy::prelude::*;
use serde_json::Value;
use std::sync::OnceLock;
use tbg_core::{
    content::{augment_def, AugmentRarity, AUGMENT_WINDOW_TICKS, ENERGY_SCALE, TICK_HZ},
    view::RoundView,
    MatchView,
};

mod layout;
use layout::spawn;

const CARD_W: f32 = 233.;
const CARD_H: f32 = CARD_W * 1180. / 820.;

#[derive(Component)]
pub(in crate::ui) struct CouncilControl;
#[derive(Resource, Default)]
struct Selection {
    round: Option<u32>,
    picked: Option<u8>,
    forgone: bool,
}
#[derive(Component)]
struct Root;
#[derive(Component)]
struct OptionCard(usize);
#[derive(Component)]
enum Picture {
    Illustration(usize),
    Frame(usize),
}
#[derive(Component)]
enum Field {
    Time,
    Waiting,
    Rarity(usize),
    Name(usize),
    Description(usize),
    Cost(usize),
}
#[derive(Component)]
enum Layout {
    Timer,
    Actions,
    Waiting,
}
#[derive(Component)]
struct Confirm;
#[derive(Component)]
struct Forgo;

pub fn plugin(app: &mut App) {
    app.init_resource::<Selection>()
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
            (visibility, options, pictures, texts, update_layout, buttons)
                .after(controls)
                .run_if(in_state(GameState::InMatch)),
        );
}

fn affordable(v: &MatchView, index: u8) -> bool {
    v.augment_round.as_ref().is_some_and(|r| {
        !r.resolved
            && r.ticks_left > 0
            && r.costs
                .get(usize::from(index))
                .is_some_and(|cost| v.energy >= *cost)
    })
}

fn controls(
    view: Res<LatestView>,
    mut selection: ResMut<Selection>,
    mut actions: ResMut<ActionQueue>,
) {
    let current = view
        .0
        .as_ref()
        .and_then(|v| v.augment_round.as_ref())
        .map(|r| r.round);
    if selection.round != current {
        *selection = Selection {
            round: current,
            ..default()
        };
    }
    let Some(v) = &view.0 else { return };
    if selection.picked.is_some_and(|index| !affordable(v, index))
        && v.augment_round.as_ref().is_some_and(|r| !r.resolved)
    {
        selection.picked = None;
    }
    for action in std::mem::take(&mut actions.0) {
        match action {
            UiAction::PickAugment(index) => {
                if affordable(v, index) {
                    selection.picked = Some(index);
                }
            }
            UiAction::ConfirmAugment => {
                if let Some(index) = selection.picked.filter(|index| affordable(v, *index)) {
                    selection.forgone = false;
                    actions.0.push(UiAction::ChooseAugment(index));
                }
            }
            UiAction::ForgoAugment => {
                selection.forgone = true;
                actions.0.push(action);
            }
            _ => actions.0.push(action),
        }
    }
}

fn visibility(
    view: Res<LatestView>,
    info: Res<MatchInfo>,
    mut roots: Query<&mut Visibility, With<Root>>,
) {
    let show = !info.spectating && view.0.as_ref().is_some_and(|v| v.augment_round.is_some());
    for mut visible in &mut roots {
        *visible = if show {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
}

fn options(
    view: Res<LatestView>,
    selection: Res<Selection>,
    mut cards: Query<(
        &OptionCard,
        &mut UiAction,
        &mut kit::buttons::Chrome,
        &mut kit::buttons::Palette,
        &mut BoxShadow,
    )>,
) {
    if !view.is_changed() && !selection.is_changed() {
        return;
    }
    let Some(v) = &view.0 else { return };
    for (card, mut action, mut chrome, mut palette, mut shadow) in &mut cards {
        let selected = selection.picked == Some(card.0 as u8);
        chrome.enabled = affordable(v, card.0 as u8);
        *action = if chrome.enabled {
            UiAction::PickAugment(card.0 as u8)
        } else {
            UiAction::Disabled
        };
        palette.border = if selected {
            Color::srgb_u8(155, 232, 106)
        } else {
            Color::NONE
        };
        shadow.0 = vec![ShadowStyle {
            color: if selected {
                Color::srgb_u8(116, 186, 84).with_alpha(0.4)
            } else {
                Color::BLACK.with_alpha(0.5)
            },
            blur_radius: px(16),
            ..default()
        }];
    }
}

fn catalog() -> &'static Value {
    static DATA: OnceLock<Value> = OnceLock::new();
    DATA.get_or_init(|| {
        serde_json::from_str(include_str!("../../../assets/augments/augments.json"))
            .expect("bundled Augment art catalog")
    })
}

fn pictures(
    view: Res<LatestView>,
    selection: Res<Selection>,
    assets: Res<AssetServer>,
    images: Res<Assets<Image>>,
    mut pictures: Query<(&Picture, &mut ImageNode)>,
) {
    if !view.is_changed() && !selection.is_changed() && !images.is_changed() {
        return;
    }
    let Some(v) = &view.0 else { return };
    let Some(r) = &v.augment_round else { return };
    for (picture, mut image) in &mut pictures {
        let (Picture::Frame(i) | Picture::Illustration(i)) = picture;
        let id = r.options[*i];
        let Some(def) = augment_def(id) else { continue };
        let art = &catalog()["augments"][usize::from(id)];
        let filename = match picture {
            Picture::Illustration(_) => art["asset"]["display"].as_str(),
            Picture::Frame(_) => {
                catalog()["rarities"][def.rarity.index()]["frame"]["display"].as_str()
            }
        };
        if let Some(filename) = filename {
            image.image = assets.load(format!("augments/{filename}"));
        }
        image.rect = match picture {
            Picture::Illustration(_) => images.get(&image.image).map(cover_rect),
            Picture::Frame(_) => None,
        };
        image.color = if (!r.resolved && v.energy < r.costs[*i])
            || (r.resolved && selection.picked != Some(*i as u8))
        {
            Color::srgb(0.5, 0.5, 0.5)
        } else {
            Color::WHITE
        };
    }
}

fn cover_rect(image: &Image) -> Rect {
    let size = image.size().as_vec2();
    let ratio = (CARD_W * 0.8) / (CARD_H * 0.45);
    let crop = if size.x / size.y > ratio {
        Vec2::new(size.y * ratio, size.y)
    } else {
        Vec2::new(size.x, size.x / ratio)
    };
    Rect::from_center_size(size * 0.5, crop)
}

fn texts(
    view: Res<LatestView>,
    selection: Res<Selection>,
    mut texts: Query<(&Field, &mut Text, &mut TextColor)>,
) {
    let Some(v) = &view.0 else { return };
    let Some(r) = &v.augment_round else { return };
    for (field, mut text, mut color) in &mut texts {
        let want = match field {
            Field::Time => {
                let seconds = r.ticks_left.div_ceil(TICK_HZ);
                format!("{}:{:02}", seconds / 60, seconds % 60)
            }
            Field::Waiting => if selection.forgone {
                "Round forgone. Waiting for opponent…"
            } else {
                "Augment selected! Waiting for opponent…"
            }
            .into(),
            _ => card_text(field, r),
        };
        if text.0 != want {
            text.0 = want;
        }
        if let Field::Rarity(i) = field {
            if let Some(def) = augment_def(r.options[*i]) {
                color.0 = rarity_color(def.rarity);
            }
        }
    }
}

fn card_text(field: &Field, r: &RoundView) -> String {
    let (Field::Rarity(i) | Field::Name(i) | Field::Description(i) | Field::Cost(i)) = field else {
        return String::new();
    };
    let Some(def) = augment_def(r.options[*i]) else {
        return String::new();
    };
    match field {
        Field::Rarity(_) => format!("{:?}", def.rarity).to_uppercase(),
        Field::Name(_) => def.name.into(),
        Field::Description(_) => def.text.into(),
        Field::Cost(_) => (r.costs[*i] / ENERGY_SCALE).to_string(),
        _ => String::new(),
    }
}

fn rarity_color(rarity: AugmentRarity) -> Color {
    let [r, g, b] = match rarity {
        AugmentRarity::Bronze => [215, 137, 105],
        AugmentRarity::Silver => [143, 198, 232],
        AugmentRarity::Gold => [232, 188, 104],
        AugmentRarity::Diamond => [240, 223, 173],
    };
    Color::srgb_u8(r, g, b)
}

fn update_layout(view: Res<LatestView>, mut nodes: Query<(&Layout, &mut Node)>) {
    let Some(r) = view.0.as_ref().and_then(|v| v.augment_round.as_ref()) else {
        return;
    };
    for (part, mut node) in &mut nodes {
        match part {
            Layout::Timer => {
                node.width = percent(
                    100. * (r.ticks_left as f32 / AUGMENT_WINDOW_TICKS as f32).clamp(0., 1.),
                )
            }
            Layout::Actions | Layout::Waiting => {
                let show = matches!(part, Layout::Waiting) == r.resolved;
                node.display = if show { Display::Flex } else { Display::None };
            }
        }
    }
}

fn buttons(
    view: Res<LatestView>,
    selection: Res<Selection>,
    mut buttons: Query<(
        &mut UiAction,
        &mut kit::buttons::Chrome,
        Option<&Confirm>,
        Option<&Forgo>,
        &mut kit::buttons::Palette,
        &Children,
    )>,
    mut labels: Query<&mut TextColor>,
) {
    let Some(v) = &view.0 else { return };
    let open = v
        .augment_round
        .as_ref()
        .is_some_and(|r| !r.resolved && r.ticks_left > 0);
    for (mut action, mut chrome, confirm, forgo, mut palette, children) in &mut buttons {
        if confirm.is_none() && forgo.is_none() {
            continue;
        }
        if confirm.is_some() {
            chrome.enabled = selection.picked.is_some_and(|i| affordable(v, i));
            *action = if chrome.enabled {
                UiAction::ConfirmAugment
            } else {
                UiAction::Disabled
            };
        }
        if forgo.is_some() {
            chrome.enabled = open;
            *action = if open {
                UiAction::ForgoAugment
            } else {
                UiAction::Disabled
            };
        }
        palette.border = if chrome.enabled {
            chrome::GOLD
        } else {
            kit::LINE
        };
        for child in children.iter() {
            if let Ok(mut text) = labels.get_mut(child) {
                text.0 = super::super::TEXT.with_alpha(if chrome.enabled { 1. } else { 0.4 });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn br_ag05_ag11_picking_is_local_and_confirming_queues_one_intention() {
        let mut state = tbg_core::MatchState::new(6);
        while state
            .view_for(tbg_core::Seat::A, &[])
            .augment_round
            .is_none()
        {
            state.step();
        }
        let view = state.view_for(tbg_core::Seat::A, &[]);
        let energy = view.energy;
        let round = view.augment_round.as_ref().unwrap().round;
        let mut app = App::new();
        app.insert_resource(LatestView(Some(view)))
            .init_resource::<Selection>()
            .init_resource::<ActionQueue>()
            .add_systems(Update, controls);
        app.world_mut()
            .resource_mut::<ActionQueue>()
            .0
            .push(UiAction::PickAugment(0));
        app.update();
        assert!(
            app.world().resource::<ActionQueue>().0.is_empty(),
            "BR-AG11: picking sends no intention"
        );
        assert_eq!(
            app.world()
                .resource::<LatestView>()
                .0
                .as_ref()
                .unwrap()
                .energy,
            energy
        );
        app.world_mut()
            .resource_mut::<ActionQueue>()
            .0
            .push(UiAction::ConfirmAugment);
        app.update();
        assert_eq!(
            app.world_mut().resource_mut::<ActionQueue>().0.pop(),
            Some(UiAction::ChooseAugment(0))
        );
        state
            .apply_intention(
                tbg_core::Seat::A,
                1,
                tbg_core::PlayerIntention::ChooseAugment { round, option: 0 },
            )
            .unwrap();
        app.world_mut().resource_mut::<LatestView>().0 =
            Some(state.view_for(tbg_core::Seat::A, &[]));
        app.world_mut()
            .resource_mut::<ActionQueue>()
            .0
            .push(UiAction::ConfirmAugment);
        app.update();
        assert!(
            app.world().resource::<ActionQueue>().0.is_empty(),
            "BR-AG05: resolved choice cannot be confirmed again"
        );
    }
}
