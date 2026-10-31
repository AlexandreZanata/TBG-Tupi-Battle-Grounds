//! Code-built art (decision D-07): mesh builders, figurines on a shared
//! v2 body plan with a twelve-joint compatibility adapter and shared clips,
//! and per-unit art kits (D-51) that own their figure, motion, aim and
//! attack. Every unit form is built once at startup into `FigureLibrary`.

use std::collections::HashMap;

use bevy::prelude::*;
use tbg_core::{UnitKind, ALL_KINDS};

mod adapter;
pub mod aim;
pub mod anim;
#[cfg(test)]
mod anim_tests;
mod archer_aim;
pub mod archer_v2;
mod archer_v2_clips;
#[cfg(test)]
mod archer_v2_tests;
pub mod arrows;
pub mod bow;
mod clips;
mod curves;
pub mod death;
mod feet;
pub mod figure;
pub mod ik;
pub mod kits;
mod legacy;
pub mod melee;
pub mod mesh;
pub mod palette;
pub mod rig;
pub(crate) mod roster;
#[cfg(test)]
mod roster_tests;
pub(crate) mod springs;

use rig::{Proportions as BodyPlan, NODE_COUNT};

/// The five forms of a unit: base, A1, A2, B1, B2 as (tier, branch).
pub const FORMS: [(u8, u8); 5] = [(0, 0), (1, 0), (2, 0), (1, 1), (2, 1)];

pub struct FigureForm {
    pub plan: BodyPlan,
    pub meshes: [Option<Handle<Mesh>>; NODE_COUNT],
    pub triangles: usize,
    pub native: bool,
    /// Half the bow-string length for native archer forms (`bow.rs`).
    pub bow_half: f32,
    enemy_meshes: Option<[Option<Handle<Mesh>>; NODE_COUNT]>,
    /// Unlit glow layer per joint (`figure::spawn_glow`), shared by both teams.
    pub glow: [Option<Handle<Mesh>>; NODE_COUNT],
}

impl FigureForm {
    pub fn meshes(&self, own: bool) -> &[Option<Handle<Mesh>>; NODE_COUNT] {
        if own {
            &self.meshes
        } else {
            self.enemy_meshes.as_ref().unwrap_or(&self.meshes)
        }
    }
}

#[derive(Resource)]
pub struct FigureLibrary {
    forms: HashMap<(UnitKind, u8, u8), FigureForm>,
    blockouts: HashMap<UnitKind, FigureForm>,
    /// One shared material for every figurine (vertex colors).
    pub material: Handle<StandardMaterial>,
    /// One shared unlit vertex-color material for every glow layer.
    pub glow_material: Handle<StandardMaterial>,
    pub bow: bow::BowAssets,
    pub stage: Handle<Mesh>,
}

impl FigureLibrary {
    pub fn blockout(&self, kind: UnitKind) -> &FigureForm {
        &self.blockouts[&kind]
    }
    pub fn get(&self, kind: UnitKind, tier: u8, branch: u8) -> &FigureForm {
        let key = if tier == 0 {
            (kind, 0, 0)
        } else {
            (kind, tier.min(2), branch.min(1))
        };
        &self.forms[&key]
    }
}

pub fn plugin(app: &mut App) {
    app.add_systems(PreStartup, build_library)
        // Animation reads where every unit ended up this frame, so it runs
        // after all Update movement and before transforms propagate.
        .add_systems(
            PostUpdate,
            (anim::animate, bow::strings)
                .chain()
                .before(bevy::transform::TransformSystems::Propagate),
        )
        .add_systems(Update, arrows::fly)
        .add_systems(
            PostUpdate,
            arrows::release.after(bevy::transform::TransformSystems::Propagate),
        )
        .add_systems(
            PostUpdate,
            springs::update.after(bevy::transform::TransformSystems::Propagate),
        );
}

fn build_library(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut mats: ResMut<Assets<StandardMaterial>>,
) {
    let material = mats.add(StandardMaterial {
        base_color: Color::WHITE,
        perceptual_roughness: 0.9,
        ..default()
    });
    let glow_material = mats.add(StandardMaterial {
        base_color: Color::WHITE,
        unlit: true,
        ..default()
    });
    let bow = bow::BowAssets::new(&mut meshes);
    let mut forms = HashMap::new();
    for kind in ALL_KINDS {
        let mut max_tris = 0;
        for (tier, branch) in FORMS {
            if let Some(kit) = kits::kit(kind) {
                let form = native_form((kit.figure)(tier, branch), &mut meshes, true);
                info!(
                    "figure {kind:?} {tier}/{branch} (kit): {} triangles, {} glow joints",
                    form.triangles,
                    form.glow.iter().flatten().count()
                );
                max_tris = max_tris.max(form.triangles);
                forms.insert((kind, tier, branch), form);
                continue;
            }
            if kind == UnitKind::Archer {
                let mut form = native_form(
                    roster::humans::archer_v2::spec_full(tier, branch),
                    &mut meshes,
                    true,
                );
                form.bow_half = archer_v2::bow_half(tier);
                info!(
                    "figure Archer {tier}/{branch} (v2): {} triangles + {} bow/arrow triangles, {} mesh joints",
                    form.triangles,
                    bow.triangles,
                    form.meshes.iter().flatten().count()
                );
                max_tris = max_tris.max(form.triangles);
                forms.insert((kind, tier, branch), form);
                continue;
            }
            if kind == UnitKind::Shieldbearer {
                let form = native_form(
                    roster::humans::shieldbearer_v2::spec(tier, branch),
                    &mut meshes,
                    true,
                );
                info!(
                    "figure Shieldbearer {tier}/{branch} (v2): {} triangles, {} mesh joints",
                    form.triangles,
                    form.meshes.iter().flatten().count()
                );
                max_tris = max_tris.max(form.triangles);
                forms.insert((kind, tier, branch), form);
                continue;
            }
            if kind == UnitKind::BoneBrute {
                let form = native_form(
                    roster::undead::bone_brute_v2::spec(tier, branch),
                    &mut meshes,
                    true,
                );
                info!(
                    "figure BoneBrute {tier}/{branch} (v2): {} triangles, {} mesh joints",
                    form.triangles,
                    form.meshes.iter().flatten().count()
                );
                max_tris = max_tris.max(form.triangles);
                forms.insert((kind, tier, branch), form);
                continue;
            }
            if kind == UnitKind::Hexer {
                let form = native_form(
                    roster::undead::hexer_v2::spec(tier, branch),
                    &mut meshes,
                    true,
                );
                info!(
                    "figure Hexer {tier}/{branch} (v2): {} triangles, {} mesh joints",
                    form.triangles,
                    form.meshes.iter().flatten().count()
                );
                max_tris = max_tris.max(form.triangles);
                forms.insert((kind, tier, branch), form);
                continue;
            }
            if kind == UnitKind::Sawbot {
                let form = native_form(
                    roster::machines::sawbot_v2::spec(tier, branch),
                    &mut meshes,
                    true,
                );
                info!(
                    "figure Sawbot {tier}/{branch} (v2): {} triangles, {} mesh joints",
                    form.triangles,
                    form.meshes.iter().flatten().count()
                );
                max_tris = max_tris.max(form.triangles);
                forms.insert((kind, tier, branch), form);
                continue;
            }
            if kind == UnitKind::SiegeWalker {
                let form = native_form(
                    roster::machines::siege_walker_v2::spec(tier, branch),
                    &mut meshes,
                    true,
                );
                info!(
                    "figure SiegeWalker {tier}/{branch} (v2): {} triangles, {} mesh joints",
                    form.triangles,
                    form.meshes.iter().flatten().count()
                );
                max_tris = max_tris.max(form.triangles);
                forms.insert((kind, tier, branch), form);
                continue;
            }
            if kind == UnitKind::Rampart {
                let form = native_form(
                    roster::machines::rampart_v2::spec(tier, branch),
                    &mut meshes,
                    true,
                );
                info!(
                    "figure Rampart {tier}/{branch} (v2): {} triangles, {} mesh joints",
                    form.triangles,
                    form.meshes.iter().flatten().count()
                );
                max_tris = max_tris.max(form.triangles);
                forms.insert((kind, tier, branch), form);
                continue;
            }
            if kind == UnitKind::Bloomkeeper {
                let form = native_form(
                    roster::plants::bloomkeeper_v2::spec(tier, branch),
                    &mut meshes,
                    true,
                );
                info!(
                    "figure Bloomkeeper {tier}/{branch} (v2): {} triangles, {} mesh joints",
                    form.triangles,
                    form.meshes.iter().flatten().count()
                );
                max_tris = max_tris.max(form.triangles);
                forms.insert((kind, tier, branch), form);
                continue;
            }
            if kind == UnitKind::Rootcaller {
                let form = native_form(
                    roster::plants::rootcaller_v2::spec(tier, branch),
                    &mut meshes,
                    true,
                );
                info!(
                    "figure Rootcaller {tier}/{branch} (v2): {} triangles, {} mesh joints",
                    form.triangles,
                    form.meshes.iter().flatten().count()
                );
                max_tris = max_tris.max(form.triangles);
                forms.insert((kind, tier, branch), form);
                continue;
            }
            if kind == UnitKind::Nester {
                let form = native_form(
                    roster::plants::nester_v2::spec(tier, branch),
                    &mut meshes,
                    true,
                );
                info!(
                    "figure Nester {tier}/{branch} (v2): {} triangles, {} mesh joints",
                    form.triangles,
                    form.meshes.iter().flatten().count()
                );
                max_tris = max_tris.max(form.triangles);
                forms.insert((kind, tier, branch), form);
                continue;
            }
            if kind == UnitKind::Beetling {
                let form = native_form(
                    roster::plants::beetling_v2::spec(tier, branch),
                    &mut meshes,
                    true,
                );
                info!(
                    "figure Beetling {tier}/{branch} (v2): {} triangles, {} mesh joints",
                    form.triangles,
                    form.meshes.iter().flatten().count()
                );
                max_tris = max_tris.max(form.triangles);
                forms.insert((kind, tier, branch), form);
                continue;
            }
            let spec = roster::figure_spec(kind, tier, branch);
            let built = adapter::build(&spec);
            let joint_count = built.iter().flatten().count();
            let triangles = built.iter().flatten().map(mesh::triangles).sum();
            info!("figure {kind:?} {tier}/{branch}: 17 body + 3 springs + 3 sockets, {joint_count} mesh joints, {triangles} triangles");
            max_tris = max_tris.max(triangles);
            let handles = built.map(|m| m.map(|m| meshes.add(m)));
            forms.insert(
                (kind, tier, branch),
                FigureForm {
                    plan: spec.plan,
                    meshes: handles,
                    triangles,
                    native: false,
                    bow_half: 0.64,
                    enemy_meshes: None,
                    glow: Default::default(),
                },
            );
        }
        info!("figure {kind:?}: max {max_tris} triangles over 5 forms");
    }
    let blockouts = ALL_KINDS
        .into_iter()
        .map(|kind| {
            let form = native_form(roster::blockout::spec(kind), &mut meshes, false);
            info!("blockout {kind:?}: {} triangles", form.triangles);
            (kind, form)
        })
        .collect();
    let stage = meshes.add(mesh::paint(
        mesh::build(mesh::Shape::Box(1.3, 0.05, 1.3)),
        palette::PARCHMENT,
    ));
    commands.insert_resource(FigureLibrary {
        forms,
        blockouts,
        material,
        glow_material,
        bow,
        stage,
    });
}

fn native_form(mut spec: figure::FigureSpec, meshes: &mut Assets<Mesh>, team: bool) -> FigureForm {
    let built = figure::build_joint_meshes(&spec);
    let glow = figure::build_glow_meshes(&spec);
    let triangles = built
        .iter()
        .chain(&glow)
        .flatten()
        .map(mesh::triangles)
        .sum();
    let handles = built.map(|m| m.map(|m| meshes.add(m)));
    let glow = glow.map(|m| m.map(|m| meshes.add(m)));
    let enemy_meshes = team.then(|| {
        for p in &mut spec.parts {
            if p.color == palette::ALLY {
                p.color = palette::ENEMY;
            }
        }
        figure::build_joint_meshes(&spec).map(|m| m.map(|m| meshes.add(m)))
    });
    FigureForm {
        plan: spec.plan,
        meshes: handles,
        triangles,
        native: true,
        bow_half: 0.64,
        enemy_meshes,
        glow,
    }
}
