//! Selective unit nameplate projection above HP bar for clicked/selected unit.

use super::UnitVisual;
use crate::camera::{self, ArenaCamera};
use bevy::prelude::*;
use tbg_core::UnitKind;

/// Wide enough for the longest name ("Antidote Brewer") at 12 px.
const TAG_W: f32 = 124.0;

#[derive(Component)]
struct Nameplate(Entity, tbg_core::UnitId);

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Update, spawn).add_systems(
        PostUpdate,
        (show, position)
            .chain()
            .after(bevy::transform::TransformSystems::Propagate),
    );
}

fn spawn(
    mut commands: Commands,
    units: Query<(Entity, &UnitVisual, &crate::ui::UiAction), Added<UnitVisual>>,
) {
    for (entity, unit, action) in &units {
        let crate::ui::UiAction::SelectUnit(id) = action else {
            continue;
        };
        commands.spawn((
            Nameplate(entity, *id),
            Text::new(name(unit.form.0)),
            TextFont {
                font_size: FontSize::Px(12.0),
                ..default()
            },
            TextColor(Color::srgb(0.96, 0.94, 0.88)),
            BackgroundColor(Color::srgba(0.06, 0.08, 0.12, 0.88)),
            BorderColor::all(Color::srgba(0.85, 0.72, 0.38, 0.80)),
            Node {
                position_type: PositionType::Absolute,
                width: px(TAG_W),
                height: px(20.0),
                border_radius: BorderRadius::all(px(4.0)),
                border: UiRect::all(px(1.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            TextLayout::justify(Justify::Center),
            Visibility::Hidden,
        ));
    }
}

/// Selection rule: a name shows only for the inspected unit, while it lives.
/// A label whose unit is gone (removed, or the match ended) despawns.
fn show(
    mut commands: Commands,
    inspected: Option<Res<crate::ui::inspect::Inspected>>,
    units: Query<&UnitVisual>,
    mut names: Query<(Entity, &Nameplate, &mut Visibility)>,
) {
    let selected_id = inspected.and_then(|i| i.0);
    for (entity, label, mut visibility) in &mut names {
        let Ok(unit) = units.get(label.0) else {
            commands.entity(entity).despawn();
            continue;
        };
        *visibility = if selected_id == Some(label.1) && !unit.dying {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
}

/// Places the shown names above their units. Desktop only: elsewhere a name
/// stays hidden, and so does one whose unit is off screen.
fn position(
    (window, scale): (Single<&Window>, Res<UiScale>),
    camera: Single<(&Camera, &GlobalTransform), With<ArenaCamera>>,
    units: Query<&UnitVisual>,
    transforms: Query<&GlobalTransform>,
    mut names: Query<(&Nameplate, &mut Node, &mut Visibility)>,
) {
    let desktop = camera::desktop(window.width(), window.height());
    let (camera, camera_transform) = *camera;
    for (label, mut node, mut visibility) in &mut names {
        if *visibility == Visibility::Hidden {
            continue;
        }
        let point = units
            .get(label.0)
            .ok()
            .filter(|_| desktop)
            .and_then(|unit| transforms.get(unit.bar).ok())
            .and_then(|bar| {
                camera
                    .world_to_viewport(camera_transform, bar.translation() + Vec3::Y * 0.20)
                    .ok()
            });
        let Some(point) = point else {
            *visibility = Visibility::Hidden;
            continue;
        };
        node.left = px(point.x / scale.0 - TAG_W / 2.0);
        node.top = px(point.y / scale.0 - 24.0);
    }
}

fn name(kind: UnitKind) -> &'static str {
    match kind {
        UnitKind::Shieldbearer => "Shieldbearer",
        UnitKind::Archer => "Archer",
        UnitKind::BoneBrute => "Bone Brute",
        UnitKind::Hexer => "Hexer",
        UnitKind::Sawbot => "Sawknight",
        UnitKind::SiegeWalker => "Ogre Archer",
        UnitKind::Bloomkeeper => "Bloomkeeper",
        UnitKind::Rootcaller => "Rootcaller",
        UnitKind::Nester => "Nester",
        UnitKind::Rampart => "Rampart",
        UnitKind::Beetling => "Beetling",
        // P43: staged kinds show their content name until named art lands.
        #[allow(unreachable_patterns)]
        _ => tbg_core::content::def(kind).name,
    }
}

#[cfg(test)]
mod tests {
    use tbg_core::{Archetype, Faction, UnitId, UnitKind};

    use super::*;
    use crate::ui::inspect::Inspected;

    /// A unit visual as `spawn_unit_visual` leaves it; `dying` once it falls.
    fn unit(dying: bool) -> UnitVisual {
        UnitVisual {
            figure: Entity::PLACEHOLDER,
            joints: [Entity::PLACEHOLDER; crate::art::rig::NODE_COUNT],
            ring: Entity::PLACEHOLDER,
            ring_base: 1.0,
            tiers: [Entity::PLACEHOLDER; 2],
            bar: Entity::PLACEHOLDER,
            bar_fill: Entity::PLACEHOLDER,
            own: true,
            archetype: Archetype::Dps,
            faction: Faction::Humans,
            splash: 0.0,
            form: (UnitKind::Archer, 0, 0),
            yaw: 0.0,
            tint: None,
            fx: 0,
            dying,
            dead_for: 0.0,
            last_x: 0.0,
            still: 0.0,
            glow: Vec::new(),
            aim: None,
            interval: 1.0,
            hold: 0.0,
            range: 1.0,
        }
    }

    /// A unit and its nameplate, spawned as `spawn` does; returns the unit.
    fn spawn_unit(world: &mut World, id: UnitId) -> Entity {
        let unit = world.spawn(unit(false)).id();
        world.spawn((Nameplate(unit, id), Visibility::Hidden));
        unit
    }

    /// Runs `show` once and returns the ids of the units whose name shows.
    fn shown(world: &mut World) -> Vec<UnitId> {
        let mut schedule = Schedule::default();
        schedule.add_systems(show);
        schedule.run(world);
        let mut ids: Vec<UnitId> = world
            .query::<(&Nameplate, &Visibility)>()
            .iter(world)
            .filter(|(_, visibility)| **visibility == Visibility::Inherited)
            .map(|(label, _)| label.1)
            .collect();
        ids.sort_unstable();
        ids
    }

    #[test]
    fn only_the_selected_unit_shows_its_name_and_selection_moves_it() {
        let mut world = World::new();
        world.insert_resource(Inspected(Some(1)));
        spawn_unit(&mut world, 1);
        spawn_unit(&mut world, 2);
        assert_eq!(shown(&mut world), vec![1]);

        world.insert_resource(Inspected(Some(2)));
        assert_eq!(shown(&mut world), vec![2], "the name follows the selection");
    }

    #[test]
    fn deselecting_hides_the_name() {
        let mut world = World::new();
        world.insert_resource(Inspected(Some(1)));
        spawn_unit(&mut world, 1);
        spawn_unit(&mut world, 2);
        assert_eq!(shown(&mut world), vec![1]);

        world.insert_resource(Inspected(None));
        assert!(shown(&mut world).is_empty());
    }

    #[test]
    fn a_dying_unit_hides_its_name_even_while_selected() {
        let mut world = World::new();
        world.insert_resource(Inspected(Some(1)));
        let unit = spawn_unit(&mut world, 1);
        assert_eq!(shown(&mut world), vec![1]);

        world.get_mut::<UnitVisual>(unit).unwrap().dying = true;
        assert!(shown(&mut world).is_empty());
    }

    #[test]
    fn names_despawn_with_their_units_at_match_end() {
        let mut world = World::new();
        world.insert_resource(Inspected(Some(1)));
        spawn_unit(&mut world, 1);
        spawn_unit(&mut world, 2);
        let units: Vec<Entity> = world
            .query_filtered::<Entity, With<UnitVisual>>()
            .iter(&world)
            .collect();
        for unit in units {
            world.despawn(unit);
        }
        assert!(shown(&mut world).is_empty());
        assert_eq!(world.query::<&Nameplate>().iter(&world).count(), 0);
    }
}
