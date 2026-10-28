//! Code-built art (decision D-07): mesh builders, figurines on a shared
//! 12-joint body plan, a shared animation library, the faction roster.
//! Every unit form is built once at startup into `FigureLibrary`.

use std::collections::HashMap;

use bevy::prelude::*;
use tbg_core::{UnitKind, UNIT_KINDS};

pub mod anim;
mod clips;
pub mod figure;
pub mod mesh;
pub mod palette;
mod roster;

use figure::BodyPlan;

/// The five forms of a unit: base, A1, A2, B1, B2 as (tier, branch).
pub const FORMS: [(u8, u8); 5] = [(0, 0), (1, 0), (2, 0), (1, 1), (2, 1)];

pub struct FigureForm {
    pub plan: BodyPlan,
    pub meshes: [Option<Handle<Mesh>>; 12],
    pub triangles: usize,
}

#[derive(Resource)]
pub struct FigureLibrary {
    forms: HashMap<(UnitKind, u8, u8), FigureForm>,
    /// One shared material for every figurine (vertex colors).
    pub material: Handle<StandardMaterial>,
}

impl FigureLibrary {
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
        .add_systems(Update, anim::animate);
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
    let mut forms = HashMap::new();
    for kind in UNIT_KINDS {
        let mut max_tris = 0;
        for (tier, branch) in FORMS {
            let spec = roster::figure_spec(kind, tier, branch);
            let built = figure::build_joint_meshes(&spec);
            let triangles = built.iter().flatten().map(mesh::triangles).sum();
            max_tris = max_tris.max(triangles);
            let handles = built.map(|m| m.map(|m| meshes.add(m)));
            forms.insert(
                (kind, tier, branch),
                FigureForm {
                    plan: spec.plan,
                    meshes: handles,
                    triangles,
                },
            );
        }
        info!("figure {kind:?}: max {max_tris} triangles over 5 forms");
    }
    commands.insert_resource(FigureLibrary { forms, material });
}
