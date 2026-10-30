//! Selective unit nameplate projection above HP bar for clicked/selected unit.

use super::UnitVisual;
use crate::camera::{self, ArenaCamera};
use bevy::prelude::*;
use tbg_core::UnitKind;

#[derive(Component)]
struct Nameplate(Entity, tbg_core::UnitId);

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Update, spawn).add_systems(
        PostUpdate,
        position.after(bevy::transform::TransformSystems::Propagate),
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
                font_size: FontSize::Px(10.0),
                ..default()
            },
            TextColor(Color::srgb(0.96, 0.94, 0.88)),
            BackgroundColor(Color::srgba(0.06, 0.08, 0.12, 0.88)),
            BorderColor::all(Color::srgba(0.85, 0.72, 0.38, 0.80)),
            Node {
                position_type: PositionType::Absolute,
                width: px(88.0),
                height: px(16.0),
                border_radius: BorderRadius::all(px(3.0)),
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

fn position(
    mut commands: Commands,
    window: Single<&Window>,
    scale: Res<UiScale>,
    camera: Single<(&Camera, &GlobalTransform), With<ArenaCamera>>,
    inspected: Option<Res<crate::ui::inspect::Inspected>>,
    units: Query<&UnitVisual>,
    transforms: Query<&GlobalTransform>,
    mut names: Query<(Entity, &Nameplate, &mut Node, &mut Visibility)>,
) {
    let desktop = camera::desktop(window.width(), window.height());
    let (camera, camera_transform) = *camera;
    let selected_id = inspected.and_then(|i| i.0);

    for (entity, label, mut node, mut visibility) in &mut names {
        let Ok(unit) = units.get(label.0) else {
            commands.entity(entity).despawn();
            continue;
        };
        // Selection rule: names are hidden unless selected by click!
        let is_selected = selected_id == Some(label.1);
        if !is_selected || !desktop || unit.dying {
            *visibility = Visibility::Hidden;
            continue;
        }

        let Some(point) = transforms.get(unit.bar).ok().and_then(|bar| {
            camera
                .world_to_viewport(camera_transform, bar.translation() + Vec3::Y * 0.20)
                .ok()
        }) else {
            *visibility = Visibility::Hidden;
            continue;
        };

        node.left = px(point.x / scale.0 - 44.0);
        node.top = px(point.y / scale.0 - 18.0);
        *visibility = Visibility::Inherited;
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
