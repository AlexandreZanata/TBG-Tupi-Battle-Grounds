//! THE SEAM: the only module that knows how a unit looks. P3 uses
//! placeholder shapes (faction-colored capsule, archetype marker, team
//! ring, tier cubes, HP bar). P6 replaces the internals, not the API.

use bevy::prelude::*;
use tbg_core::content::def;
use tbg_core::{Archetype, SlotCount, UnitId, UnitKind, UnitView};

use crate::ui::UiAction;

pub mod assets;
mod cues;

pub use assets::VisualAssets;
pub use cues::{play_cue, spawn_projectile, VisualCue};

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
    pub body: Entity,
    ring: Entity,
    tiers: [Entity; 2],
    bar: Entity,
    bar_fill: Entity,
    pub own: bool,
    pub archetype: Archetype,
    faction: usize,
    body_y: f32,
    anim: Option<(VisualCue, f32)>,
    tint: Option<(Handle<StandardMaterial>, f32)>,
    pub dying: bool,
}

pub fn plugin(app: &mut App) {
    app.add_systems(PreStartup, assets::init)
        .add_systems(Update, (cues::animate, cues::billboards, cues::fly));
}

pub fn spawn_unit_visual(
    commands: &mut Commands,
    a: &VisualAssets,
    look: UnitLook,
    pos: Vec3,
) -> Entity {
    let d = def(look.kind);
    let big = usize::from(d.slots == SlotCount::Two);
    let height = [0.8, 1.2][big];
    let faction = d.faction.index();
    let mesh = |m: &Handle<Mesh>, mat: &Handle<StandardMaterial>, t: Transform| {
        (Mesh3d(m.clone()), MeshMaterial3d(mat.clone()), t)
    };
    let body_y = height / 2.0;
    let body = commands
        .spawn(mesh(
            &a.bodies[big],
            &a.faction[faction],
            Transform::from_xyz(0.0, body_y, 0.0),
        ))
        .id();
    let marker_t = Transform::from_xyz(0.0, height + 0.15, 0.0);
    let marker = commands
        .spawn(mesh(&a.markers[d.archetype as usize], &a.marker, marker_t))
        .id();
    let ring_mat = &a.team[usize::from(!look.own)];
    let ring = commands
        .spawn(mesh(
            &a.ring,
            ring_mat,
            Transform::from_xyz(0.0, 0.02, 0.0).with_scale(Vec3::splat([1.0, 1.4][big])),
        ))
        .id();
    let tier_mat = if look.branch == 0 { &a.gold } else { &a.silver };
    let tiers = [(0u8, -0.12), (1, 0.12)].map(|(i, x)| {
        let t = Transform::from_xyz(x, height + 0.42, 0.0);
        let vis = if look.tier > i {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        commands.spawn((mesh(&a.tier_cube, tier_mat, t), vis)).id()
    });
    let bar_fill = commands
        .spawn(mesh(
            &a.bar,
            &a.bar_fill[0],
            Transform::from_xyz(0.0, 0.0, 0.01),
        ))
        .id();
    let bar_bg = commands
        .spawn(mesh(
            &a.bar,
            &a.bar_bg,
            Transform::from_scale(Vec3::new(1.04, 1.4, 1.0)),
        ))
        .id();
    let bar = commands
        .spawn((
            Transform::from_xyz(0.0, height + 0.7, 0.0),
            Visibility::default(),
        ))
        .add_children(&[bar_bg, bar_fill])
        .id();
    let children = [body, marker, ring, tiers[0], tiers[1], bar];
    let visual = UnitVisual {
        body,
        ring,
        tiers,
        bar,
        bar_fill,
        own: look.own,
        archetype: d.archetype,
        faction,
        body_y,
        anim: Some((VisualCue::Spawn, 0.0)),
        tint: None,
        dying: false,
    };
    let root = commands
        .spawn((
            Transform::from_translation(pos),
            Visibility::default(),
            visual,
        ))
        .add_children(&children)
        .id();
    if look.own {
        // Tapping near an own unit opens its evolution panel (ui/input.rs).
        commands.entity(root).insert(UiAction::SelectUnit(look.id));
    }
    root
}

/// Apply evolution, HP, and status from the latest view.
pub fn update_unit_visual(commands: &mut Commands, a: &VisualAssets, v: &UnitVisual, u: &UnitView) {
    for (i, cube) in v.tiers.iter().enumerate() {
        let shown = usize::from(u.tier) > i;
        let mat = if u.branch == 0 { &a.gold } else { &a.silver };
        let vis = if shown {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        commands
            .entity(*cube)
            .insert((vis, MeshMaterial3d(mat.clone())));
    }
    let ratio = (u.hp as f32 / u.max_hp.max(1) as f32).clamp(0.0, 1.0);
    let fill = &a.bar_fill[usize::from(ratio < 0.6) + usize::from(ratio < 0.3)];
    let t =
        Transform::from_xyz(-(1.0 - ratio) * 0.4, 0.0, 0.01).with_scale(Vec3::new(ratio, 1.0, 1.0));
    commands
        .entity(v.bar_fill)
        .insert((t, MeshMaterial3d(fill.clone())));
    let ring = match u.status {
        1 => &a.slowed,
        2 => &a.rooted,
        _ => &a.team[usize::from(!v.own)],
    };
    commands.entity(v.ring).insert(MeshMaterial3d(ring.clone()));
}
