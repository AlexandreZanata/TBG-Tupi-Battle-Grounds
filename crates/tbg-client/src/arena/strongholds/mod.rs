//! Stronghold towers drawn from the authoritative view: kind, Fortification
//! tier and branch, and the HP damage state. A tower's pieces are rebuilt
//! only when one of those four changes, always from the current view, so a
//! new tier never brings back parts that damage already took.

mod aim;
mod artillery;
mod aura;
mod bastion;
mod classic;
mod command;
mod gallery;
mod gunners;
mod motion;
mod parts;
#[cfg(test)]
mod tests;
mod treasury;

use bevy::prelude::*;
use tbg_core::content::LANE_LEN;
use tbg_core::{StrongholdKind, StrongholdView};

use crate::sync::LatestView;
use crate::visuals::AimTarget;
use motion::{Gun, Moving};
use parts::{Form, Piece, StrongholdArt};

/// World distance from a Stronghold's lane x back to its tower's center.
const SETBACK: f32 = 0.9;

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<StrongholdArt>()
        .add_systems(Startup, spawn_towers)
        .add_systems(
            Update,
            (
                sync_looks,
                aim::orders,
                motion::fire,
                aim::track,
                motion::animate,
            )
                .chain(),
        )
        .add_systems(Update, gallery::spawn.run_if(gallery::wanted));
}

/// Everything a tower's shape depends on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Look {
    pub kind: StrongholdKind,
    pub tier: u8,
    pub branch: u8,
    /// 0 above 66 % HP, 1 down to 33 %, 2 below.
    pub damage: u8,
}

impl Look {
    const CLASSIC: Look = Look {
        kind: StrongholdKind::Classic,
        tier: 0,
        branch: 0,
        damage: 0,
    };

    pub fn from_view(view: &StrongholdView, hp: i32) -> Look {
        let tier = view.tier.min(2);
        Look {
            kind: view.kind,
            tier,
            // Before tier 1 no branch is chosen; ignore it so it cannot rebuild.
            branch: if tier == 0 { 0 } else { view.branch.min(1) },
            damage: damage_level(hp, view.max_hp),
        }
    }

    fn form(self) -> Form {
        Form::of(self.tier, self.branch)
    }
}

/// The three damage states: 2 at <= 1/3 of `max_hp`, 1 at <= 2/3, else 0.
/// `max_hp` is the kind's own, after Fortification.
pub(crate) fn damage_level(hp: i32, max_hp: i32) -> u8 {
    let (hp, max) = (i64::from(hp), i64::from(max_hp.max(1)));
    if 3 * hp <= max {
        2
    } else if 3 * hp <= 2 * max {
        1
    } else {
        0
    }
}

/// Every piece of a kind and form, before damage.
fn pieces(kind: StrongholdKind, form: Form) -> Vec<Piece> {
    let mut p = match kind {
        StrongholdKind::Classic => classic::pieces(),
        StrongholdKind::Treasury => treasury::pieces(form),
        StrongholdKind::Bastion => bastion::pieces(form),
        StrongholdKind::Artillery => artillery::pieces(form),
        StrongholdKind::Command => command::pieces(form),
    };
    p.extend(gunners::pieces(kind));
    p
}

/// The pieces a look shows: those its damage has not taken yet.
fn visible(look: Look) -> impl Iterator<Item = Piece> {
    pieces(look.kind, look.form())
        .into_iter()
        .filter(move |p| p.lost > look.damage)
}

/// A tower root; its children are the pieces of `look`.
#[derive(Component)]
pub(crate) struct Tower {
    pub own: bool,
    /// Follows the authoritative view (false for gallery towers).
    live: bool,
    look: Option<Look>,
    gun: Option<Gun>,
    /// The aimed gun of the look, if any (`aim.rs`).
    turret: Option<aim::Turret>,
    /// Seconds since the last shot, and shots so far.
    since: f32,
    shots: u32,
    /// What the tower aims at, seconds since its last order, and how long
    /// an ordered shot has waited for the turret.
    target: Option<AimTarget>,
    idle: f32,
    pending: Option<f32>,
    /// Current turret yaw and absolute elevation (radians).
    yaw: f32,
    pitch: f32,
}

impl Tower {
    fn new(own: bool, live: bool) -> Self {
        Self {
            own,
            live,
            look: None,
            gun: None,
            turret: None,
            since: f32::MAX,
            shots: 0,
            target: None,
            idle: 0.0,
            pending: None,
            yaw: 0.0,
            pitch: 0.0,
        }
    }

    fn kind(&self) -> StrongholdKind {
        self.look.map_or(StrongholdKind::Classic, |l| l.kind)
    }
}

fn spawn_towers(
    mut commands: Commands,
    mut art: ResMut<StrongholdArt>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let far = LANE_LEN as f32 / 1000.0 + SETBACK;
    for (own, x) in [(true, -SETBACK), (false, far)] {
        let mut tower = Tower::new(own, true);
        let root = commands
            .spawn((
                Name::new(if own {
                    "own Stronghold"
                } else {
                    "enemy Stronghold"
                }),
                Transform::from_xyz(x, 0.0, 0.0),
                Visibility::default(),
            ))
            .id();
        if own {
            // Clicking it opens the Fortification panel (ui/input.rs).
            commands
                .entity(root)
                .insert(crate::ui::UiAction::SelectTower);
        }
        dress(
            &mut commands,
            &mut art,
            &mut meshes,
            root,
            &mut tower,
            Look::CLASSIC,
        );
        commands.entity(root).insert(tower);
    }
}

/// Rebuild a live tower whenever its look in the latest view changes.
fn sync_looks(
    mut commands: Commands,
    view: Res<LatestView>,
    mut art: ResMut<StrongholdArt>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut towers: Query<(Entity, &mut Tower)>,
) {
    if !view.is_changed() {
        return;
    }
    let Some(v) = view.0.as_ref() else {
        return;
    };
    let me = v.you.index();
    for (root, mut tower) in &mut towers {
        let i = if tower.own { me } else { 1 - me };
        let look = Look::from_view(&v.strongholds[i], v.stronghold_hp[i]);
        if tower.live && tower.look != Some(look) {
            dress(&mut commands, &mut art, &mut meshes, root, &mut tower, look);
        }
    }
}

/// Replace a tower's pieces with those of `look`, reusing cached meshes.
fn dress(
    commands: &mut Commands,
    art: &mut StrongholdArt,
    meshes: &mut Assets<Mesh>,
    root: Entity,
    tower: &mut Tower,
    look: Look,
) {
    commands.entity(root).despawn_related::<Children>();
    for piece in visible(look) {
        let at = piece.placed(tower.own);
        let mut child = commands.spawn((
            Mesh3d(art.mesh(piece.def, tower.own, meshes)),
            MeshMaterial3d(art.material.clone()),
            at,
            ChildOf(root),
        ));
        if let Some(motion) = piece.motion {
            child.insert(Moving { motion, rest: at });
        }
    }
    if look.kind == StrongholdKind::Command {
        aura::spawn(commands, art, meshes, root, tower.own, look.form(), SETBACK);
    }
    tower.look = Some(look);
    tower.gun = (look.kind == StrongholdKind::Artillery).then(|| artillery::gun(look.form()));
    tower.turret = aim::turret(look);
    if let Some(t) = tower.turret.filter(|_| tower.target.is_none()) {
        tower.pitch = t.rest_pitch;
    }
}
