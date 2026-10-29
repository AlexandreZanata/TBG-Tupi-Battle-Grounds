//! THE SEAM: the only module that knows how a unit looks. Since P6 a unit
//! is a code-built figurine (`art`) with a team ring, rank pips, and an HP
//! bar; cues drive the shared animation library.

use bevy::{light::NotShadowCaster, prelude::*};
use tbg_core::content::def;
use tbg_core::content::effective_stats;
use tbg_core::{Archetype, Faction, UnitId, UnitKind};

use crate::art::anim::Animator;
use crate::art::{figure, FigureLibrary};
use crate::ui::UiAction;

pub mod assets;
mod cues;
mod names;
mod update;

pub use assets::VisualAssets;
pub use cues::{play_cue, spawn_projectile, VisualCue};
pub use update::update_unit_visual;

pub struct UnitLook {
    pub id: UnitId,
    pub kind: UnitKind,
    pub tier: u8,
    pub branch: u8,
    pub own: bool,
}

/// Root of one unit's visual; child entities are listed for updates.
#[derive(Component)]
pub struct UnitVisual {
    figure: Entity,
    joints: [Entity; crate::art::rig::NODE_COUNT],
    ring: Entity,
    tiers: [Entity; 2],
    bar: Entity,
    bar_fill: Entity,
    pub own: bool,
    pub archetype: Archetype,
    pub faction: Faction,
    /// Splash radius in tiles (0 = none), for the splash-ring effect.
    pub splash: f32,
    form: (UnitKind, u8, u8),
    yaw: f32,
    /// Tint material, seconds left, already applied to the joints.
    tint: Option<(Handle<StandardMaterial>, f32, bool)>,
    pub dying: bool,
    dead_for: f32,
    last_x: f32,
}

pub fn plugin(app: &mut App) {
    app.add_plugins(names::plugin)
        .add_systems(PreStartup, assets::init)
        .add_systems(Update, (cues::animate, cues::billboards, cues::fly));
}

/// Figures turn 25° toward the camera from facing straight down the lane.
fn yaw(own: bool) -> f32 {
    if own {
        65f32.to_radians()
    } else {
        -65f32.to_radians()
    }
}

pub fn spawn_unit_visual(
    commands: &mut Commands,
    a: &VisualAssets,
    lib: &FigureLibrary,
    look: UnitLook,
    pos: Vec3,
) -> Entity {
    let d = def(look.kind);
    let form = lib.get(look.kind, look.tier, look.branch);
    let height = 1.05 * form.plan.scale;
    let mesh = |m: &Handle<Mesh>, mat: &Handle<StandardMaterial>, t: Transform| {
        (Mesh3d(m.clone()), MeshMaterial3d(mat.clone()), t)
    };
    let rot = Quat::from_rotation_y(yaw(look.own));
    let figure_root = commands
        .spawn((
            Transform::from_rotation(rot).with_scale(Vec3::splat(form.plan.scale)),
            Visibility::default(),
        ))
        .id();
    let joints = figure::spawn_joints(
        commands,
        figure_root,
        &form.plan,
        form.meshes(look.own),
        &lib.material,
    );
    let ring_mat = &a.team[usize::from(!look.own)];
    let ring_scale = Vec3::splat(0.85 * form.plan.scale);
    let ring = commands
        .spawn(mesh(
            &a.ring,
            ring_mat,
            Transform::from_xyz(0.0, 0.02, 0.0).with_scale(ring_scale),
        ))
        .insert(NotShadowCaster)
        .id();
    let (bar, bar_fill, tiers) = spawn_bar(commands, a, height, &look);
    let visual = UnitVisual {
        figure: figure_root,
        joints,
        ring,
        tiers,
        bar,
        bar_fill,
        own: look.own,
        archetype: d.archetype,
        faction: d.faction,
        splash: effective_stats(look.kind, look.tier, look.branch, 0).splash as f32 / 1000.0,
        form: (look.kind, look.tier, look.branch),
        yaw: yaw(look.own),
        tint: None,
        dying: false,
        dead_for: 0.0,
        last_x: pos.x,
    };
    let mut animator = Animator::new(joints, form.plan, figure_root, look.kind, look.id);
    animator.native = form.native;
    animator.archer = form.native && look.kind == UnitKind::Archer;
    animator.play(crate::art::anim::Clip::Spawn);
    let root = commands.spawn_empty().id();
    if animator.archer {
        crate::art::bow::attach(
            commands,
            root,
            &animator,
            lib,
            &lib.material,
            look.own,
            form.bow_half,
        );
    }
    commands
        .entity(root)
        .insert((
            Transform::from_translation(pos),
            Visibility::default(),
            visual,
            animator,
        ))
        .add_children(&[figure_root, ring, bar]);
    if look.own {
        // Tapping near an own unit opens its evolution panel (ui/input.rs).
        commands.entity(root).insert(UiAction::SelectUnit(look.id));
    }
    root
}

fn spawn_bar(
    commands: &mut Commands,
    a: &VisualAssets,
    height: f32,
    look: &UnitLook,
) -> (Entity, Entity, [Entity; 2]) {
    let mesh = |m: &Handle<Mesh>, mat: &Handle<StandardMaterial>, t: Transform| {
        (Mesh3d(m.clone()), MeshMaterial3d(mat.clone()), t)
    };
    let tier_mat = if look.branch == 0 { &a.gold } else { &a.silver };
    let tiers = [(0u8, -0.08), (1, 0.08)].map(|(i, x)| {
        let vis = if look.tier > i {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        commands
            .spawn((
                mesh(&a.tier_cube, tier_mat, Transform::from_xyz(x, 0.1, 0.0)),
                vis,
                NotShadowCaster,
            ))
            .id()
    });
    let bar_fill = commands
        .spawn(mesh(
            &a.bar,
            &a.bar_fill[0],
            Transform::from_xyz(0.0, 0.0, 0.01),
        ))
        .insert(NotShadowCaster)
        .id();
    let bar_bg = commands
        .spawn(mesh(
            &a.bar,
            &a.bar_bg,
            Transform::from_scale(Vec3::new(1.04, 1.4, 1.0)),
        ))
        .insert(NotShadowCaster)
        .id();
    let bar = commands
        .spawn((
            Transform::from_xyz(0.0, height + 0.25, 0.0),
            Visibility::default(),
        ))
        .add_children(&[bar_bg, bar_fill, tiers[0], tiers[1]])
        .id();
    (bar, bar_fill, tiers)
}
