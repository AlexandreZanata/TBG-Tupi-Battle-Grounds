//! Keyboard navigation for the existing queue, result and battle actions.

use super::{ActionQueue, UiAction};
static TABBED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
pub(super) fn tabbed() -> bool {
    TABBED.load(std::sync::atomic::Ordering::Relaxed)
}
use crate::GameState;
use bevy::prelude::*;

#[cfg(not(tbg_desktop))]
type Buttons<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static UiAction,
        &'static ComputedNode,
        &'static UiGlobalTransform,
        &'static InheritedVisibility,
        Option<&'static bevy::ui::CalculatedClip>,
    ),
>;

#[cfg(tbg_desktop)]
type Buttons<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static UiAction,
        &'static ComputedNode,
        &'static UiGlobalTransform,
        &'static InheritedVisibility,
        Option<&'static bevy::ui::CalculatedClip>,
        Option<&'static super::native_battle::CouncilControl>,
    ),
>;

#[allow(clippy::too_many_arguments)] // Bevy declares independent resources and component filters in the system signature.
pub(super) fn navigate(
    keys: Res<ButtonInput<KeyCode>>,
    state: Res<State<GameState>>,
    form: NonSend<super::textform::ActiveForm>,
    buttons: Buttons,
    mut focus: Local<usize>,
    mut match_id: Local<Option<[u8; 16]>>,
    info: Res<crate::sync::MatchInfo>,
    chat: Res<super::match_chat::Chat>,
    mut button_focus: ResMut<super::kit::buttons::Focus>,
    mut queue: ResMut<ActionQueue>,
    #[cfg(tbg_desktop)] view: Res<crate::sync::LatestView>,
) {
    if *match_id != info.result_id {
        *match_id = info.result_id;
        TABBED.store(false, std::sync::atomic::Ordering::Relaxed);
    }
    if super::p75::modal() || chat.focused || super::match_chat::consumed() {
        return;
    }
    if *state.get() == GameState::InMatch && keys.just_pressed(KeyCode::Enter) && !tabbed() {
        return;
    }
    if *state.get() == GameState::Menu || super::textform::is_open(&form) {
        return;
    }
    if keys.just_pressed(KeyCode::Escape) {
        queue.0.push(match state.get() {
            GameState::Queue => UiAction::Cancel,
            GameState::Result => UiAction::ToMenu,
            _ => UiAction::Deselect,
        });
    }
    if !keys.just_pressed(KeyCode::Tab) && !keys.just_pressed(KeyCode::Enter) {
        return;
    }
    let mut visible: Vec<_> = buttons
        .iter()
        .filter(|entry| {
            let (_, a, n, t, v, c, ..) = entry;
            #[cfg(tbg_desktop)]
            if view.0.as_ref().is_some_and(|v| v.augment_round.is_some()) && entry.6.is_none() {
                return false;
            }
            **a != UiAction::Disabled
                && v.get()
                && n.size.x > 0.
                && c.is_none_or(|c| c.contains_point(t.translation))
        })
        .collect();
    visible.sort_by_key(|(_, _, _, t, ..)| {
        (
            (t.translation.y * 10.) as i32,
            (t.translation.x * 10.) as i32,
        )
    });
    if visible.is_empty() {
        return;
    }
    if keys.just_pressed(KeyCode::Tab) {
        TABBED.store(true, std::sync::atomic::Ordering::Relaxed);
        *focus = if keys.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]) {
            (*focus + visible.len() - 1) % visible.len()
        } else {
            (*focus + 1) % visible.len()
        };
    }
    button_focus.0 = Some(visible[*focus % visible.len()].0);
    if keys.just_pressed(KeyCode::Enter) && !keys.any_pressed([KeyCode::AltLeft, KeyCode::AltRight])
    {
        queue.0.push(*visible[*focus % visible.len()].1);
    }
}

#[cfg(test)]mod tests{
use super::*;
#[test]fn ch11_enter_enter_without_tab_never_queues_surrender(){
    let mut app=App::new();
    app.insert_resource(State::new(GameState::InMatch)).insert_resource(ButtonInput::<KeyCode>::default()).insert_resource(ActionQueue::default()).insert_resource(crate::sync::MatchInfo{result_id:Some([1;16]),..default()}).insert_resource(super::super::match_chat::Chat::default()).insert_resource(super::super::kit::buttons::Focus::default()).insert_non_send(super::super::textform::ActiveForm::default());
    #[cfg(tbg_desktop)]app.insert_resource(crate::sync::LatestView::default());
    app.world_mut().spawn((UiAction::Surrender,ComputedNode{size:Vec2::splat(100.),..default()},UiGlobalTransform::default(),InheritedVisibility::VISIBLE));
    app.add_systems(Update,navigate);
    for _ in 0..2{app.world_mut().resource_mut::<ButtonInput<KeyCode>>().reset_all();app.world_mut().resource_mut::<ButtonInput<KeyCode>>().press(KeyCode::Enter);app.update();assert!(app.world().resource::<ActionQueue>().0.is_empty(),"CH-11: implicit focus must never surrender");}
    app.world_mut().resource_mut::<ButtonInput<KeyCode>>().reset_all();app.world_mut().resource_mut::<ButtonInput<KeyCode>>().press(KeyCode::Tab);app.update();
    app.world_mut().resource_mut::<ButtonInput<KeyCode>>().reset_all();app.world_mut().resource_mut::<ButtonInput<KeyCode>>().press(KeyCode::Enter);app.update();
    assert_eq!(app.world().resource::<ActionQueue>().0,vec![UiAction::Surrender],"explicit Tab focus keeps keyboard activation usable");
    TABBED.store(false,std::sync::atomic::Ordering::Relaxed);
}}
