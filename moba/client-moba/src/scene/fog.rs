//! Shared map-space fog mask and per-fragment scenery shading on both desktop cameras.

use crate::{app::Screen, link::Feed};
use bevy::asset::{embedded_asset, RenderAssetUsages};
use bevy::image::ImageSampler;
use bevy::pbr::{ExtendedMaterial, MaterialExtension};
use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, Extent3d, TextureDimension, TextureFormat};
use bevy::shader::ShaderRef;
use tbg_moba_core::{content, types::Side};

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
    world.units.iter().any(|u| {
        u.side == Side::Team(seat.team()) && {
            let p = Vec2::new(u.x as f32, u.y as f32) / 100.;
            p.distance_squared(point) <= (content::sight(u.kind) as f32 / 1000.).powi(2)
        }
    })
}
fn update(
    feed: Res<Feed>,
    screen: Res<State<Screen>>,
    mask: Res<FogTexture>,
    mut images: ResMut<Assets<Image>>,
) {
    let Some(image) = images.get_mut(&mask.0) else {
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
    let pixels_per_unit = RESOLUTION as f32 / tbg_moba_core::map::SIZE as f32;
    for u in world
        .units
        .iter()
        .filter(|u| u.side == Side::Team(seat.team()))
    {
        let centre = Vec2::new(u.x as f32, u.y as f32) / 100. * pixels_per_unit;
        let radius = content::sight(u.kind) as f32 / 1000. * pixels_per_unit;
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
                // Fade only inside the authoritative radius; unseen terrain never becomes bright.
                let alpha = ((radius - distance) / 1.6).clamp(0., 1.);
                let alpha = alpha * alpha * (3. - 2. * alpha);
                let i = y as usize * RESOLUTION + x as usize;
                data[i] = data[i].max((alpha * 255.) as u8);
            }
        }
    }
}
