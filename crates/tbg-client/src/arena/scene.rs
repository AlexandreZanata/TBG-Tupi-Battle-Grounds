//! ARENAS-G: which arena the scene shows, and switching it without a reload.

use super::theme::Theme;
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
struct Built(Theme);

pub(super) fn plugin(app: &mut App) {
    let theme = Theme::selected();
    app.insert_resource(ActiveArena(theme))
        .insert_resource(Built(theme))
        .add_systems(Startup, spawn)
        .add_systems(OnEnter(GameState::Menu), preview)
        .add_systems(OnEnter(GameState::InMatch), follow_match)
        .add_systems(Update, switch);
}

fn spawn(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut mats: ResMut<Assets<StandardMaterial>>,
    built: Res<Built>,
) {
    super::build(&mut commands, &mut meshes, &mut mats, built.0);
}

fn preview(mut active: ResMut<ActiveArena>) {
    active.set_if_neq(ActiveArena(Theme::selected()));
}

fn follow_match(info: Res<MatchInfo>, mut active: ResMut<ActiveArena>) {
    active.set_if_neq(ActiveArena::of(info.arena));
}

fn switch(
    mut commands: Commands,
    active: Res<ActiveArena>,
    mut built: ResMut<Built>,
    scene: Query<Entity, With<ArenaScene>>,
    mut fog: Query<&mut DistanceFog>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut mats: ResMut<Assets<StandardMaterial>>,
) {
    if active.0 == built.0 {
        return;
    }
    for entity in &scene {
        commands.entity(entity).despawn();
    }
    super::build(&mut commands, &mut meshes, &mut mats, active.0);
    for mut fog in &mut fog {
        fog.color = active.0.palette().sky;
    }
    built.0 = active.0;
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
            .insert_resource(ActiveArena(Theme::Forest))
            .insert_resource(Built(Theme::Forest))
            .add_systems(Startup, spawn)
            .add_systems(Update, switch);
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
                Theme::Forest,
            ] {
                app.world_mut().resource_mut::<ActiveArena>().0 = theme;
                app.update();
                assert_eq!(count::<With<DirectionalLight>>(&mut app), 1, "{theme:?}");
                assert!(count::<With<ArenaScene>>(&mut app) <= forest + 8);
            }
        }
        assert_eq!(count::<With<ArenaScene>>(&mut app), forest);
        assert_eq!(count::<()>(&mut app), world, "nothing leaks outside the scene");
    }
}
