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

pub mod aim;
pub mod assets;
mod attacks;
mod bars;
mod cues;
mod growth;
#[cfg(test)]
pub mod layout;
pub mod mount;
mod names;
mod overhead;
mod shell;
mod status_badges;
mod update;

pub use assets::VisualAssets;
#[cfg(test)]
pub use cues::CORPSE_SECS;
pub use cues::{play_cue, spawn_projectile, VisualCue};
pub use update::update_unit_visual;

pub struct UnitLook {
    pub id: UnitId,
    pub kind: UnitKind,
    pub tier: u8,
    pub branch: u8,
    pub own: bool,
}

/// What a unit aims at: another unit's visual, or a fixed point (a
/// Stronghold, a review dummy).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AimTarget {
    Unit(Entity),
    Point(Vec3),
}

/// Stronghold shots of this frame, (own tower, target), for the towers to
/// aim and fire their own projectiles (`arena::strongholds`).
#[derive(Resource, Default)]
pub struct TowerOrders(pub Vec<(bool, AimTarget)>);

/// Root of one unit's visual; child entities are listed for updates.
#[derive(Component)]
pub struct UnitVisual {
    pub figure: Entity,
    joints: [Entity; crate::art::rig::NODE_COUNT],
    ring: Entity,
    status_ring: Entity,
    /// Rest scale of the team ring (the inspected unit's ring pulses above it).
    ring_base: f32,
    tiers: [Entity; 2],
    bar: Entity,
    bar_fill: Entity,
    bleed_badge: Entity,
    shield_outline: Option<Entity>,
    mark_badge: Option<Entity>,
    weaken_badge: Option<Entity>,
    venom_badge: Option<Entity>,
    control_badge: Option<Entity>,
    pub own: bool,
    pub archetype: Archetype,
    pub faction: Faction,
    /// Splash radius in tiles (0 = none), for the splash-ring effect.
    pub splash: f32,
    form: (UnitKind, u8, u8),
    pub yaw: f32,
    /// Tint material, seconds left, already applied to the joints.
    tint: Option<(Handle<StandardMaterial>, f32, bool)>,
    pub dying: bool,
    fx: u16,
    /// Derived from public caster marks; never feeds a gameplay decision.
    pub marked: bool,
    /// One server-selected cosmetic offhand destination, consumed by the real joint emitter.
    pub secondary_shot: Option<Vec3>,
    dead_for: f32,
    last_x: f32,
    /// Seconds without moving (walk/idle hysteresis).
    still: f32,
    /// Unlit glow children of the current form.
    glow: Vec<Entity>,
    /// The engaged target and seconds left before the aim lets go.
    aim: Option<(AimTarget, f32)>,
    /// Seconds between attacks, root duration, and range (tiles) of the form.
    interval: f32,
    pub hold: f32,
    pub range: f32,
}

/// Seconds a unit keeps aiming after its last attack started, beyond its
/// attack interval.
const AIM_GRACE: f32 = 0.7;

impl UnitVisual {
    /// Turn toward `target` until the next attack interval has passed.
    pub fn engage(&mut self, target: AimTarget) {
        self.aim = Some((target, self.interval + AIM_GRACE));
    }
    pub fn kind(&self) -> UnitKind {
        self.form.0
    }
    /// The installed form may precede the animator refresh within the same snapshot.
    pub(crate) fn form(&self) -> (u8, u8) {
        (self.form.1, self.form.2)
    }
    /// The unit fires or strikes through its own art kit.
    pub fn has_kit(&self) -> bool {
        self.kit().is_some()
    }
    pub fn kit(&self) -> Option<&'static crate::art::kits::Kit> {
        crate::art::kits::kit(self.form.0)
    }
    fn refresh_stats(&mut self) {
        let (kind, tier, branch) = self.form;
        let s = effective_stats(kind, tier, branch, 0);
        self.splash = s.splash as f32 / 1000.0;
        self.interval = f32::from(s.interval) / tbg_core::content::TICK_HZ as f32;
        self.range = s.range as f32 / 1000.0;
        self.hold = match s.on_hit {
            Some(tbg_core::OnHit::Root { ticks }) => {
                f32::from(ticks) / tbg_core::content::TICK_HZ as f32
            }
            _ => 0.0,
        };
    }
}

pub fn plugin(app: &mut App) {
    app.add_plugins((names::plugin, shell::plugin, growth::plugin))
        .init_resource::<TowerOrders>()
        .add_systems(PreStartup, assets::init)
        .add_systems(
            Update,
            (
                aim::update_aim,
                cues::animate,
                cues::fly,
                highlight_inspected,
            ),
        )
        .add_systems(
            PostUpdate,
            bars::billboards
                .after(crate::art::anim::animate)
                .before(bevy::transform::TransformSystems::Propagate),
        )
        .add_systems(
            PostUpdate,
            (attacks::release, attacks::stomp).after(bevy::transform::TransformSystems::Propagate),
        );
}

/// The selected unit's ground ring pulses as a luminous base halo. The
/// figure itself is not lifted: `aim` owns its translation every frame.
fn highlight_inspected(
    time: Res<Time>,
    inspected: Option<Res<crate::ui::inspect::Inspected>>,
    units: Query<(&UnitVisual, &UiAction)>,
    mut transforms: Query<&mut Transform>,
) {
    let target = inspected.and_then(|i| i.0);
    let pulse = 1.40 + 0.08 * (time.elapsed_secs() * 5.0).sin();
    for (v, action) in &units {
        let UiAction::SelectUnit(id) = action else {
            continue;
        };
        let is_selected = target == Some(*id);
        let scale = v.ring_base * if is_selected { pulse } else { 1.0 };
        if let Ok(mut t) = transforms.get_mut(v.ring) {
            if (t.scale.x - scale).abs() > 1e-4 {
                t.scale = Vec3::splat(scale);
            }
        }
        if let Ok(mut t) = transforms.get_mut(v.figure) {
            let target_y = if is_selected { 0.07 } else { 0.0 };
            if (t.translation.y - target_y).abs() > 1e-4 {
                t.translation.y = target_y;
            }
        }
    }
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
    spawn_form_visual(commands, a, lib, look, pos, false)
}

pub fn spawn_training_dummy(
    commands: &mut Commands,
    a: &VisualAssets,
    lib: &FigureLibrary,
    look: UnitLook,
    pos: Vec3,
) -> Entity {
    spawn_form_visual(commands, a, lib, look, pos, true)
}

fn spawn_form_visual(
    commands: &mut Commands,
    a: &VisualAssets,
    lib: &FigureLibrary,
    look: UnitLook,
    pos: Vec3,
    dummy: bool,
) -> Entity {
    let d = def(look.kind);
    let form = if dummy {
        &lib.training_dummy
    } else {
        lib.get(look.kind, look.tier, look.branch)
    };
    let height = if dummy {
        1.75 * form.plan.scale + 0.25
    } else {
        form.overhead
    };
    let mesh = |m: &Handle<Mesh>, mat: &Handle<StandardMaterial>, t: Transform| {
        (Mesh3d(m.clone()), MeshMaterial3d(mat.clone()), t)
    };
    let base_yaw = if matches!(
        look.kind,
        UnitKind::Bonehorn | UnitKind::Drillnest | UnitKind::Weaver
    ) {
        if look.own {
            std::f32::consts::FRAC_PI_2
        } else {
            -std::f32::consts::FRAC_PI_2
        }
    } else {
        yaw(look.own)
    };
    let rot = Quat::from_rotation_y(base_yaw);
    let figure_root = commands
        .spawn((
            Transform::from_rotation(rot).with_scale(Vec3::splat(form.plan.scale)),
            Visibility::default(),
        ))
        .id();
    let joints = figure::spawn_joints(
        commands,
        figure_root,
        look.kind,
        &form.plan,
        form.meshes(look.own),
        &lib.material,
    );
    let ring_mat = &a.team[usize::from(!look.own)];
    let ring_scale = Vec3::splat(0.85 * form.plan.scale);
    let ring = commands
        .spawn(mesh(
            if look.own { &a.ring } else { &a.enemy_ring },
            ring_mat,
            Transform::from_xyz(0.0, 0.02, 0.0).with_scale(ring_scale),
        ))
        .insert(NotShadowCaster)
        .id();
    let status_ring = commands
        .spawn((
            mesh(
                &a.shield_outline,
                &a.slowed,
                Transform::from_xyz(0.0, 0.04, 0.0).with_scale(Vec3::splat(form.plan.scale)),
            ),
            Visibility::Hidden,
            NotShadowCaster,
        ))
        .id();
    let (bar, bar_fill, tiers) = spawn_bar(commands, a, height, &look);
    let bleed_badge = status_badges::spawn_bleed(commands, a, bar);
    let glow = figure::spawn_glow(commands, &joints, &form.glow, &lib.glow_material);
    let mut visual = UnitVisual {
        figure: figure_root,
        joints,
        ring,
        status_ring,
        ring_base: ring_scale.x,
        tiers,
        bar,
        bar_fill,
        bleed_badge,
        shield_outline: None,
        mark_badge: None,
        weaken_badge: None,
        venom_badge: None,
        control_badge: None,
        own: look.own,
        archetype: d.archetype,
        faction: d.faction,
        splash: 0.0,
        form: (look.kind, look.tier, look.branch),
        yaw: base_yaw,
        tint: None,
        dying: false,
        fx: if look.kind == UnitKind::Shatterback {
            4 << 8
        } else {
            0
        },
        marked: false,
        secondary_shot: None,
        dead_for: 0.0,
        last_x: pos.x,
        still: 1.0,
        glow,
        aim: None,
        interval: 1.0,
        hold: 0.0,
        range: 1.0,
    };
    visual.refresh_stats();
    let mut animator = Animator::new(joints, form.plan, figure_root, look.kind, look.id);
    animator.native = form.native;
    animator.form = (look.tier, look.branch);
    if dummy {
        animator.kit = None;
        animator.preview = Some((crate::art::anim::Clip::Idle, 0.));
    } else {
        animator.play(crate::art::anim::Clip::Spawn);
    }
    let root = commands.spawn_empty().id();
    commands
        .entity(root)
        .insert((
            Transform::from_translation(pos),
            Visibility::default(),
            visual,
            animator,
        ))
        .add_children(&[figure_root, ring, status_ring, bar]);
    // Tapping near any unit opens its status panel; own units also get the
    // Evolution panel (ui/input.rs, ui/actions.rs).
    commands.entity(root).insert(UiAction::SelectUnit(look.id));
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
    let tier_mat = &a.star_fill;
    let tiers = [
        (0u8, overhead::marker_x(look.tier, 0)),
        (1, overhead::marker_x(look.tier, 1)),
    ]
    .map(|(i, x)| {
        let vis = if look.tier > i {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        let outline = commands
            .spawn((
                mesh(
                    &a.tier_star,
                    &a.star_edge,
                    Transform::from_xyz(0.0, 0.0, -0.01).with_scale(Vec3::splat(1.18)),
                ),
                NotShadowCaster,
            ))
            .id();
        commands
            .spawn((
                mesh(&a.tier_star, tier_mat, Transform::from_xyz(x, 0.19, 0.025)),
                vis,
                NotShadowCaster,
            ))
            .add_child(outline)
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
        .spawn((Transform::from_xyz(0.0, height, 0.0), Visibility::default()))
        .add_children(&[bar_bg, bar_fill, tiers[0], tiers[1]])
        .id();
    (bar, bar_fill, tiers)
}
