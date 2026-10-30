//! Desktop unit nameplates projected above HP bars; small screens keep their existing UI.

use super::UnitVisual;
use crate::{
    art::palette::INK,
    camera::{self, ArenaCamera},
};
use bevy::prelude::*;
use tbg_core::UnitKind;

#[derive(Component)]
struct Nameplate(Entity);

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Update, spawn).add_systems(
        PostUpdate,
        position.after(bevy::transform::TransformSystems::Propagate),
    );
}

fn spawn(mut commands: Commands, units: Query<(Entity, &UnitVisual), Added<UnitVisual>>) {
    for (entity, unit) in &units {
        commands.spawn((
            Nameplate(entity),
            Text::new(name(unit.form.0)),
            TextFont {
                font_size: FontSize::Px(9.0),
                ..default()
            },
            TextColor(INK),
            BackgroundColor(Color::srgba(0.94, 0.90, 0.78, 0.88)),
            Node {
                position_type: PositionType::Absolute,
                width: px(84.0),
                height: px(13.0),
                border_radius: BorderRadius::all(px(2.0)),
                justify_content: JustifyContent::Center,
                ..default()
            },
            TextLayout::justify(Justify::Center),
            Visibility::Hidden,
        ));
    }
}

#[allow(clippy::too_many_arguments)] // Projection and a reused collision buffer share one layout pass.
fn position(
    mut commands: Commands,
    window: Single<&Window>,
    scale: Res<UiScale>,
    camera: Single<(&Camera, &GlobalTransform), With<ArenaCamera>>,
    units: Query<&UnitVisual>,
    transforms: Query<&GlobalTransform>,
    mut names: Query<(Entity, &Nameplate, &mut Node, &mut Visibility)>,
    mut placed: Local<Vec<Vec4>>,
) {
    placed.clear();
    let desktop = camera::desktop(window.width(), window.height());
    let (camera, camera_transform) = *camera;
    for (entity, label, mut node, mut visibility) in &mut names {
        let Ok(unit) = units.get(label.0) else {
            commands.entity(entity).despawn();
            continue;
        };
        *visibility = Visibility::Hidden;
        if !desktop || unit.dying {
            continue;
        }
        let Some(point) = transforms.get(unit.bar).ok().and_then(|bar| {
            camera
                .world_to_viewport(camera_transform, bar.translation() + Vec3::Y * 0.18)
                .ok()
        }) else {
            continue;
        };
        let rect = clear_rect(
            Vec4::new(
                point.x / scale.0 - 42.0,
                point.y / scale.0 - 14.0,
                84.0,
                13.0,
            ),
            &placed,
        );
        placed.push(rect);
        node.left = px(rect.x);
        node.top = px(rect.y);
        *visibility = Visibility::Inherited;
    }
}

fn clear_rect(mut rect: Vec4, placed: &[Vec4]) -> Vec4 {
    for _ in 0..12 {
        let overlap = placed.iter().any(|r| {
            rect.x < r.x + r.z + 2.0
                && rect.x + rect.z + 2.0 > r.x
                && rect.y < r.y + r.w + 2.0
                && rect.y + rect.w + 2.0 > r.y
        });
        if !overlap {
            break;
        }
        rect.y -= 16.0;
    }
    rect
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
        _ => tbg_core::content::def(kind).name,
    }
}
