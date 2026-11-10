//! Shared map-space fog mask and per-fragment scenery shading on both desktop cameras.

use crate::{app::Screen, link::Feed};
use bevy::asset::{embedded_asset, RenderAssetUsages};
use bevy::image::ImageSampler;
use bevy::pbr::{ExtendedMaterial, MaterialExtension};
use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, Extent3d, TextureDimension, TextureFormat};
use bevy::shader::ShaderRef;
use tbg_moba_core::{
    content,
    geom::{Pos, MM},
    map,
    protocol::{UnitView, WorldView},
    types::Side,
    vision::{home_half, invading_scout, SightCircle, Territory},
    Team, UnitKind,
};

const RESOLUTION: usize = 256;
pub type FogMaterial = ExtendedMaterial<StandardMaterial, FogMask>;
#[derive(Asset, AsBindGroup, Reflect, Debug, Clone)]
pub struct FogMask {
    #[texture(100)]
    #[sampler(101)]
    pub texture: Handle<Image>,
}
impl MaterialExtension for FogMask {
    fn fragment_shader() -> ShaderRef {
        "embedded://tbg_moba/scene/fog.wesl".into()
    }
}
#[derive(Resource)]
pub struct FogTexture(pub Handle<Image>);

pub fn plugin(app: &mut App) {
    embedded_asset!(app, "fog.wesl");
    app.add_plugins(MaterialPlugin::<FogMaterial>::default())
        .add_systems(Startup, setup.before(super::setup_material))
        .add_systems(Update, update);
}
pub fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let mut image = Image::new_fill(
        Extent3d {
            width: RESOLUTION as u32,
            height: RESOLUTION as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &[255],
        TextureFormat::R8Unorm,
        RenderAssetUsages::default(),
    );
    image.sampler = ImageSampler::linear();
    commands.insert_resource(FogTexture(images.add(image)));
}

/// Cosmetic use only. Online visibility always comes from the Team's received frame.
pub fn point_visible(feed: &Feed, point: Vec2) -> bool {
    let (Some(world), Some(seat)) = (&feed.curr, feed.seat) else {
        return false;
    };
    let point = Pos {
        x: (point.x * MM as f32) as i32,
        y: (point.y * MM as f32) as i32,
    };
    let team = seat.team();
    home_half(team, point)
        || territory(world, team).contains(point)
        || world
            .units
            .iter()
            .any(|u| scout(u, team) && position(u).within(point, content::sight(u.kind)))
}

fn position(unit: &UnitView) -> Pos {
    Pos {
        x: i32::from(unit.x) * 10,
        y: i32::from(unit.y) * 10,
    }
}
fn scout(unit: &UnitView, team: Team) -> bool {
    invading_scout(team, unit.side, unit.kind, position(unit))
}
fn territory(world: &WorldView, team: Team) -> Territory {
    let standing: Vec<_> = world
        .units
        .iter()
        .filter(|u| u.side == Side::Team(team.enemy()) && matches!(u.kind, UnitKind::Tower(..)))
        .map(position)
        .collect();
    Territory::new(team, &standing)
}
fn update(
    feed: Res<Feed>,
    screen: Res<State<Screen>>,
    mask: Res<FogTexture>,
    mut images: ResMut<Assets<Image>>,
) {
    let Some(mut image) = images.get_mut(&mask.0) else {
        return;
    };
    let Some(data) = image.data.as_mut() else {
        return;
    };
    if *screen.get() != Screen::Match {
        data.fill(255);
        return;
    }
    data.fill(0);
    let (Some(world), Some(seat)) = (&feed.curr, feed.seat) else {
        return;
    };
    let team = seat.team();
    paint_home(data, team);
    for area in territory(world, team).conquered {
        stamp(data, area);
    }
    for unit in world.units.iter().filter(|u| scout(u, team)) {
        stamp(
            data,
            SightCircle {
                centre: position(unit),
                radius: content::sight(unit.kind),
            },
        );
    }
}
fn paint_home(data: &mut [u8], team: Team) {
    let map_mm = map::SIZE * MM;
    for y in 0..RESOLUTION {
        for x in 0..RESOLUTION {
            let centre = Pos {
                x: (2 * x as i32 + 1) * map_mm / (2 * RESOLUTION as i32),
                y: (2 * y as i32 + 1) * map_mm / (2 * RESOLUTION as i32),
            };
            if home_half(team, centre) {
                data[y * RESOLUTION + x] = 255;
            }
        }
    }
}
fn stamp(data: &mut [u8], area: SightCircle) {
    let pixels_per_mm = RESOLUTION as f32 / (map::SIZE * MM) as f32;
    let centre = Vec2::new(area.centre.x as f32, area.centre.y as f32) * pixels_per_mm;
    let radius = area.radius as f32 * pixels_per_mm;
    let min = (centre - Vec2::splat(radius))
        .floor()
        .max(Vec2::ZERO)
        .as_uvec2();
    let max = (centre + Vec2::splat(radius))
        .ceil()
        .min(Vec2::splat(RESOLUTION as f32 - 1.))
        .as_uvec2();
    for y in min.y..=max.y {
        for x in min.x..=max.x {
            let distance = Vec2::new(x as f32 + 0.5, y as f32 + 0.5).distance(centre);
            // The feather stays inside the authoritative circle, with no extra scouting radius.
            let alpha = ((radius - distance) / 1.6).clamp(0., 1.);
            let alpha = alpha * alpha * (3. - 2. * alpha);
            let i = y as usize * RESOLUTION + x as usize;
            data[i] = data[i].max((alpha * 255.) as u8);
        }
    }
}
