//! Strategy-only Shop/Destination actions and camera navigation; no unit movement orders.

use bevy::prelude::*;
use bevy::ui::{ComputedStackIndex, UiGlobalTransform};
use tbg_moba_core::{Destination, MobaIntention, Role};

use crate::app::Screen;
use crate::camera::{CameraDrag, MinimapRect, Rig, MAX_DISTANCE, MIN_DISTANCE};
use crate::link::{Feed, Outbox};

/// Everything a click or hotkey can do.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    /// Buy the offer in this Shop slot and send it to the Destination.
    BuySlot(usize),
    ToggleLock(usize),
    Reroll,
    SetDestination(usize),
    Upgrade,
    Center,
    ZoomIn,
    ZoomOut,
    PickRole(usize),
    PlayOffline,
    PlayOnline,
    PlayAgain,
    ToMenu,
    Quit,
}

/// UI panels that swallow clicks and wheel so the world under them is safe.
#[derive(Component)]
pub struct Blocks;

#[derive(Resource, Default)]
pub struct PointerOverUi(pub bool);

/// The Destination new Troops are sent to.
#[derive(Resource, Clone, Copy)]
pub struct Selection {
    pub destination: usize,
}

impl Selection {
    pub fn for_role(role: Role) -> Self {
        let destination = match role {
            Role::Top => 0,
            Role::Mid => 1,
            Role::Bot | Role::Support => 2,
            Role::Jungle => 3,
        };
        Self { destination }
    }
}

#[derive(Resource, Default)]
pub struct Actions(pub Vec<Action>);

pub fn plugin(app: &mut App) {
    app.init_resource::<PointerOverUi>()
        .init_resource::<Actions>()
        .insert_resource(Selection::for_role(Role::Support))
        .add_systems(PreUpdate, hover.after(bevy::ui::UiSystems::Layout))
        .init_resource::<Centered>()
        .add_systems(
            Update,
            (
                clicks,
                hotkeys.run_if(in_state(Screen::Match)),
                minimap_camera.run_if(in_state(Screen::Match)),
                first_center.run_if(in_state(Screen::Match)),
                apply,
            )
                .chain(),
        )
        .add_systems(
            OnEnter(Screen::Match),
            |launch: Res<crate::config::Launch>,
             mut sel: ResMut<Selection>,
             mut centered: ResMut<Centered>| {
                *sel = Selection::for_role(launch.role);
                centered.0 = false;
            },
        );
}

type Nodes<'w, 's, T> = Query<
    'w,
    's,
    (
        &'static T,
        &'static ComputedNode,
        &'static UiGlobalTransform,
        &'static InheritedVisibility,
        &'static ComputedStackIndex,
    ),
>;

fn hit<'a, T: Component>(nodes: &'a Nodes<T>, physical: Vec2) -> Option<&'a T> {
    nodes
        .iter()
        .filter(|(_, node, t, vis, _)| vis.get() && node.contains_point(**t, physical))
        .max_by_key(|(.., stack)| stack.0)
        .map(|(a, ..)| a)
}

fn hover(
    window: Single<&Window>,
    blocks: Nodes<Blocks>,
    minimap: Res<MinimapRect>,
    mut over: ResMut<PointerOverUi>,
) {
    over.0 = window
        .cursor_position()
        .is_some_and(|p| minimap.contains(p) || hit(&blocks, p * window.scale_factor()).is_some());
}

fn clicks(
    window: Single<&Window>,
    mouse: Res<ButtonInput<MouseButton>>,
    buttons: Nodes<Action>,
    mut actions: ResMut<Actions>,
) {
    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }
    if let Some(p) = window.cursor_position() {
        if let Some(action) = hit(&buttons, p * window.scale_factor()) {
            actions.0.push(*action);
        }
    }
}

fn hotkeys(keys: Res<ButtonInput<KeyCode>>, mut actions: ResMut<Actions>) {
    let map = [
        (KeyCode::Digit1, Action::BuySlot(0)),
        (KeyCode::Digit2, Action::BuySlot(1)),
        (KeyCode::Digit3, Action::BuySlot(2)),
        (KeyCode::KeyZ, Action::SetDestination(0)),
        (KeyCode::KeyX, Action::SetDestination(1)),
        (KeyCode::KeyC, Action::SetDestination(2)),
        (KeyCode::KeyV, Action::SetDestination(3)),
        (KeyCode::KeyR, Action::Reroll),
        (KeyCode::KeyU, Action::Upgrade),
        (KeyCode::Space, Action::Center),
        (KeyCode::Escape, Action::ToMenu),
    ];
    actions.0.extend(
        map.into_iter()
            .filter(|(k, _)| keys.just_pressed(*k))
            .map(|(_, a)| a),
    );
}

/// A left click/drag that starts inside the minimap moves only the camera.
pub fn minimap_camera(
    window: Single<&Window>,
    mouse: Res<ButtonInput<MouseButton>>,
    minimap: Res<MinimapRect>,
    drag: Res<CameraDrag>,
    mut rig: ResMut<Rig>,
    mut active: Local<bool>,
) {
    if !window.focused || !mouse.pressed(MouseButton::Left) {
        *active = false;
        return;
    }
    let Some(cursor) = window.cursor_position() else {
        return;
    };
    if mouse.just_pressed(MouseButton::Left) {
        *active = minimap.contains(cursor) && !drag.active;
    }
    if *active {
        rig.focus = minimap
            .point_on_map(cursor)
            .clamp(Vec2::ZERO, Vec2::splat(tbg_moba_core::map::SIZE as f32));
    }
}

/// Applies queued actions; screen changes and restarts go through commands.
#[allow(clippy::too_many_arguments)] // the one place every action lands
pub fn apply(
    mut commands: Commands,
    mut actions: ResMut<Actions>,
    mut selection: ResMut<Selection>,
    mut outbox: ResMut<Outbox>,
    mut rig: ResMut<Rig>,
    mut launch: ResMut<crate::config::Launch>,
    mut next: ResMut<NextState<Screen>>,
    feed: Res<Feed>,
    mut exit: MessageWriter<AppExit>,
) {
    for action in actions.0.drain(..) {
        match action {
            Action::SetDestination(i) => selection.destination = i.min(3),
            Action::BuySlot(slot) => outbox.0.push(MobaIntention::BuyTroop {
                slot: slot as u8,
                to: Destination::ALL[selection.destination],
            }),
            Action::ToggleLock(slot) => outbox
                .0
                .push(MobaIntention::ToggleLock { slot: slot as u8 }),
            Action::Reroll => outbox.0.push(MobaIntention::Reroll),
            Action::Upgrade => outbox.0.push(MobaIntention::Upgrade),
            Action::Center => center(&feed, &mut rig),
            Action::ZoomIn => rig.distance = (rig.distance * 0.8).max(MIN_DISTANCE),
            Action::ZoomOut => rig.distance = (rig.distance * 1.25).min(MAX_DISTANCE),
            Action::PickRole(i) => launch.role = Role::ALL[i.min(4)],
            Action::PlayOffline | Action::PlayOnline => {
                launch.online = action == Action::PlayOnline;
                next.set(Screen::Match);
            }
            Action::PlayAgain => commands.queue(crate::link::restart_local),
            Action::ToMenu => next.set(Screen::Menu),
            Action::Quit => {
                exit.write(AppExit::Success);
            }
        }
    }
}

/// Whether the camera already jumped to the player's base this match.
#[derive(Resource, Default)]
pub struct Centered(pub bool);

fn first_center(
    mut centered: ResMut<Centered>,
    feed: Res<Feed>,
    launch: Res<crate::config::Launch>,
    mut rig: ResMut<Rig>,
    mut selection: ResMut<Selection>,
) {
    if centered.0 || feed.seat.is_none() || feed.curr.is_none() {
        return;
    }
    centered.0 = true;
    // Online the server picks the seat, so the default destination follows it.
    if let Some(seat) = feed.seat {
        *selection = Selection::for_role(seat.role());
    }
    if launch.overview {
        *rig = Rig::default();
    } else {
        center(&feed, &mut rig);
    }
}

/// Jumps the camera to the player's team base; Seats have no avatar.
pub fn center(feed: &Feed, rig: &mut Rig) {
    let Some(seat) = feed.seat else { return };
    let base = tbg_moba_core::map::spawn_point(seat.team());
    rig.focus = Vec2::new(base.x as f32, base.y as f32) / 1000.;
}
