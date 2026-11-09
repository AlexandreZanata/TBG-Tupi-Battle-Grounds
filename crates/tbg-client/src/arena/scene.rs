//! ARENAS-G: which arena the scene shows, and switching it without a reload.

use super::{
    blender::{
        self,
        packs::{Packs, Status},
        Model,
    },
    theme::Theme,
};
use crate::sync::MatchInfo;
use crate::GameState;
use bevy::prelude::*;

/// The arena on screen. A match sets it from `MatchStart` (the server's
/// draw, or the Practice choice); the lobby shows the saved choice.
#[derive(Resource, Clone, Copy, PartialEq, Eq)]
pub struct ActiveArena(pub(crate) Theme);

impl ActiveArena {
    pub fn of(arena: tbg_core::Arena) -> Self {
        Self(arena.into())
    }
}

/// Every static scene entity and light, so a switch removes exactly them;
/// Strongholds and troops come from the views and stay.
#[derive(Component)]
pub(crate) struct ArenaScene;

/// The arena the current `ArenaScene` entities were built for.
#[derive(Resource)]
pub(crate) struct Built(pub(crate) Option<Theme>);

pub(super) fn plugin(app: &mut App) {
    let theme = Theme::selected();
    app.insert_resource(ActiveArena(theme))
        .insert_resource(Built(None))
        .add_systems(OnEnter(GameState::Menu), preview)
        .add_systems(OnEnter(GameState::InMatch), follow_match)
        .add_systems(Update, switch);
}

fn preview(mut active: ResMut<ActiveArena>) {
    active.set_if_neq(ActiveArena(Theme::selected()));
}

fn follow_match(info: Res<MatchInfo>, mut active: ResMut<ActiveArena>) {
    active.set_if_neq(ActiveArena::of(info.arena));
}

type SceneAssets<'w> = (
    ResMut<'w, Assets<Mesh>>,
    ResMut<'w, Assets<StandardMaterial>>,
    ResMut<'w, Assets<Image>>,
);

fn switch(
    mut commands: Commands,
    active: Res<ActiveArena>,
    mut built: ResMut<Built>,
    scene: Query<Entity, With<ArenaScene>>,
    mut fog: Query<&mut DistanceFog>,
    assets: SceneAssets,
    mut packs: Packs,
) {
    if Some(active.0) == built.0 {
        return;
    }
    let model = if blender::reference() {
        None
    } else {
        match packs.get(blender::arena(active.0)) {
            Status::Loading => return,
            Status::Ready(bytes) => Model::read(bytes),
            Status::Missing => None,
        }
    };
    let (mut meshes, mut mats, mut images) = assets;
    for entity in &scene {
        commands.entity(entity).despawn();
    }
    if let Some(model) = model {
        blender::scenery::spawn(&mut commands, &mut meshes, &mut mats, active.0, model);
    } else {
        if !blender::reference() {
            warn!("Blender scenery pack unavailable: {}", active.0.name());
        }
        super::build_procedural(&mut commands, &mut meshes, &mut mats, &mut images, active.0);
    }
    for mut fog in &mut fog {
        fog.color = active.0.palette().sky;
        fog.falloff = active.0.fog();
    }
    built.0 = Some(active.0);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn count<F: bevy::ecs::query::QueryFilter>(app: &mut App) -> usize {
        let world = app.world_mut();
        world.query_filtered::<(), F>().iter(world).count()
    }

    #[test]
    fn repeated_switches_keep_exactly_one_scene_and_one_sun() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AssetPlugin::default()))
            .init_asset::<Mesh>()
            .init_asset::<StandardMaterial>()
            .init_asset::<Image>()
            .insert_resource(ActiveArena(Theme::Forest))
            .insert_resource(Built(None))
            .add_systems(Update, switch);
        blender::packs::plugin(&mut app);
        for theme in [
            Theme::Forest,
            Theme::Tundra,
            Theme::Volcanic,
            Theme::Ruins,
            Theme::Storm,
            Theme::Celestial,
            Theme::Clockwork,
            Theme::Desert,
        ] {
            blender::packs::seed(app.world_mut(), blender::arena(theme));
        }
        app.update();
        let forest = count::<With<ArenaScene>>(&mut app);
        let world = count::<()>(&mut app);
        assert!(forest > 0);
        for _ in 0..2 {
            for theme in [
                Theme::Tundra,
                Theme::Volcanic,
                Theme::Ruins,
                Theme::Storm,
                Theme::Celestial,
                Theme::Clockwork,
                Theme::Desert,
                Theme::Forest,
            ] {
                app.world_mut().resource_mut::<ActiveArena>().0 = theme;
                app.update();
                assert_eq!(count::<With<DirectionalLight>>(&mut app), 1, "{theme:?}");
                assert!(count::<With<ArenaScene>>(&mut app) <= forest + 8);
            }
        }
        assert_eq!(count::<With<ArenaScene>>(&mut app), forest);
        assert_eq!(
            count::<()>(&mut app),
            world,
            "nothing leaks outside the scene"
        );
    }
}
