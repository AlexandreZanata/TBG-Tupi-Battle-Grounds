//! Pointer and keyboard → HUD actions, Commander orders and camera jumps.

use bevy::prelude::*;
use bevy::ui::{ComputedStackIndex, UiGlobalTransform};
use tbg_moba_core::geom::Pos;
use tbg_moba_core::{Destination, MobaIntention, Role};

use crate::app::Screen;
use crate::camera::{MainCamera, MinimapRect, Rig, MAX_DISTANCE, MIN_DISTANCE};
use crate::link::{Feed, Outbox};
use crate::scene::to_map;
use crate::units::{Marker, Spawned, UnitVisual};

/// Everything a click or hotkey can do.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    /// Buy the offer in this Shop slot and send it to the Destination.
    BuySlot(usize),
    ToggleLock(usize),
    Reroll,
    SetDestination(usize),
    Upgrade,
    Auto,
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
                world_orders.run_if(in_state(Screen::Match)),
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
        (KeyCode::KeyA, Action::Auto),
        (KeyCode::Space, Action::Center),
        (KeyCode::Escape, Action::ToMenu),
    ];
    actions.0.extend(
        map.into_iter()
            .filter(|(k, _)| keys.just_pressed(*k))
            .map(|(_, a)| a),
    );
}

/// Right click on the ground (or the minimap) moves the Commander; left
/// click on the minimap moves the camera.
#[allow(clippy::too_many_arguments)] // world picking needs camera, window, minimap and HUD state
fn world_orders(
    window: Single<&Window>,
    mouse: Res<ButtonInput<MouseButton>>,
    camera: Single<(&Camera, &GlobalTransform), With<MainCamera>>,
    minimap: Res<MinimapRect>,
    over: Res<PointerOverUi>,
    mut rig: ResMut<Rig>,
    mut outbox: ResMut<Outbox>,
    mut markers: MessageWriter<Marker>,
) {
    let Some(cursor) = window.cursor_position() else {
        return;
    };
    if minimap.contains(cursor) {
        let at = minimap.point_on_map(cursor);
        if mouse.pressed(MouseButton::Left) {
            rig.focus = at;
        }
        if mouse.just_pressed(MouseButton::Right) {
            order(&mut outbox, &mut markers, at);
        }
        return;
    }
    if over.0 || !mouse.just_pressed(MouseButton::Right) {
        return;
    }
    let (camera, transform) = *camera;
    let Ok(ray) = camera.viewport_to_world(transform, cursor) else {
        return;
    };
    let Some(distance) = ray.intersect_plane(Vec3::ZERO, InfinitePlane3d::new(Vec3::Y)) else {
        return;
    };
    order(&mut outbox, &mut markers, to_map(ray.get_point(distance)));
}

fn order(outbox: &mut Outbox, markers: &mut MessageWriter<Marker>, at: Vec2) {
    let to = Pos {
        x: (at.x * 1000.0) as i32,
        y: (at.y * 1000.0) as i32,
    }
    .clamp_to_map();
    outbox.0.push(MobaIntention::MoveCommander { to });
    markers.write(Marker { at });
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
    spawned: Res<Spawned>,
    units: Query<&UnitVisual>,
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
            Action::Auto => outbox.0.push(MobaIntention::CommanderAuto),
            Action::Center => center(&feed, &spawned, &units, &mut rig),
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

/// Whether the camera already jumped to the player's Commander this match.
#[derive(Resource, Default)]
pub struct Centered(pub bool);

fn first_center(
    mut centered: ResMut<Centered>,
    feed: Res<Feed>,
    spawned: Res<Spawned>,
    units: Query<&UnitVisual>,
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
        center(&feed, &spawned, &units, &mut rig);
    }
}

/// Jumps the camera to the player's Commander (or base while it is dead).
pub fn center(feed: &Feed, spawned: &Spawned, units: &Query<&UnitVisual>, rig: &mut Rig) {
    let Some(world) = &feed.curr else { return };
    let Some(seat) = feed.seat else { return };
    let commander = world.seats.get(seat.0 as usize).map_or(0, |s| s.commander);
    let at = spawned
        .by_id
        .get(&commander)
        .and_then(|e| units.get(*e).ok())
        .map(|v| v.curr)
        .unwrap_or_else(|| {
            let base = tbg_moba_core::map::spawn_point(seat.team());
            Vec2::new(base.x as f32, base.y as f32) / 1000.0
        });
    rig.focus = at;
}
