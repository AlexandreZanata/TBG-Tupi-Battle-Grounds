//! Fits Blender Stronghold pieces and sockets onto the existing tower motion and wear systems.

use super::{
    aim::{self, Turret},
    damage::Wear,
    motion::Moving,
    parts::{Form, Motion, StrongholdArt, NEVER},
    Look, Tower,
};
use crate::{
    arena::{
        blender::{
            self,
            packs::{Packs, Status},
            Model,
        },
        theme::Theme,
    },
    vfx::{impacts::Muzzle, shots::ShotKind},
};
use bevy::prelude::*;
use tbg_core::StrongholdKind;

#[derive(Clone)]
pub(super) struct Part {
    mesh: Handle<Mesh>,
    rest: Transform,
    motion: Option<Motion>,
    hide: u8,
}

#[derive(Clone)]
pub(super) struct Art {
    parts: Vec<Part>,
    turret: Option<Turret>,
}

pub(super) fn form_index(form: Form) -> usize {
    match form {
        Form::Base => 0,
        Form::A1 => 1,
        Form::A2 => 2,
        Form::B1 => 3,
        Form::B2 => 4,
    }
}

pub(super) fn pack(packs: &mut Packs, look: Look, blue: bool, theme: Theme) -> Status<'_> {
    if blender::reference() {
        return Status::Missing;
    }
    if look.kind == StrongholdKind::Classic {
        return packs.get(blender::classic(theme, blue));
    }
    match blender::stronghold(look.kind, form_index(look.form()), blue) {
        Some(path) => packs.get(path),
        None => Status::Missing,
    }
}

fn load(look: Look, blue: bool, meshes: &mut Assets<Mesh>, bytes: &[u8]) -> Option<Art> {
    let model = Model::read(bytes)?;
    if look.kind == StrongholdKind::Classic {
        let parts = model
            .batches
            .into_iter()
            .map(|batch| Part {
                mesh: meshes.add(batch.mesh),
                rest: batch.track[0].1,
                motion: None,
                hide: wear(&batch.name).unwrap_or(NEVER),
            })
            .collect();
        return Some(Art {
            parts,
            turret: None,
        });
    }
    let primary = if look.kind == StrongholdKind::Bastion && look.form() == Form::B2 {
        "gun2"
    } else {
        "gun"
    };
    let barrel = model.batches.iter().find(|part| part.name == primary)?;
    let rest = barrel.track[0].1;
    let (shot, flash, arc) = match look.kind {
        StrongholdKind::Artillery | StrongholdKind::Treasury => {
            let authored = aim::turret(Look { damage: 0, ..look })?;
            (authored.shot, authored.flash, authored.arc)
        }
        StrongholdKind::Bastion if primary == "gun2" => (ShotKind::Ballista, Muzzle::Bolt, 0.42),
        StrongholdKind::Bastion => (ShotKind::RapidCannon, Muzzle::Bombard, 0.9),
        _ => (ShotKind::Crossbow, Muzzle::Bolt, std::f32::consts::PI),
    };
    let turret = Turret {
        pivot: rest.translation,
        rest_pitch: (rest.rotation * Vec3::X).y.asin(),
        muzzle: barrel.socket,
        shot,
        flash,
        orbit: look.kind == StrongholdKind::Command,
        arc,
    };
    let parts = model
        .batches
        .into_iter()
        .map(|batch| {
            let name = batch.name;
            let mut rest = batch.track[0].1;
            let motion = mechanism(name, primary, look.kind);
            if !blue {
                rest.translation.x = -rest.translation.x;
                if name == primary || name.ends_with("bolt") || name.starts_with("orbit") {
                    rest.rotation = Quat::from_rotation_y(std::f32::consts::PI) * rest.rotation;
                } else {
                    rest.rotation = Quat::from_xyzw(
                        rest.rotation.x,
                        -rest.rotation.y,
                        -rest.rotation.z,
                        rest.rotation.w,
                    );
                    rest.scale.x = -rest.scale.x;
                }
            }
            let hide = wear(name).unwrap_or(if motion.is_some() { 20 } else { NEVER });
            Part {
                mesh: meshes.add(batch.mesh),
                rest,
                motion,
                hide,
            }
        })
        .collect();
    Some(Art {
        parts,
        turret: Some(turret),
    })
}

fn wear(name: &str) -> Option<u8> {
    name.strip_prefix("wear")?.parse().ok()
}

fn mechanism(name: &str, primary: &str, kind: StrongholdKind) -> Option<Motion> {
    if name == primary {
        return Some(match kind {
            StrongholdKind::Command => Motion::Mounted(0.025),
            StrongholdKind::Artillery => Motion::Kick(0.18),
            StrongholdKind::Bastion if primary == "gun" => Motion::Kick(-0.10),
            _ => Motion::Kick(0.06),
        });
    }
    let role = name.trim_end_matches(|c: char| c.is_ascii_digit());
    match role {
        "gunbolt" | "gun2bolt" => Some(Motion::Nocked),
        "swivel" | "orbit" => Some(Motion::Swivel),
        "hatch" => Some(Motion::Hatch(0.84)),
        "notch" => Some(Motion::Notch(0.26)),
        "turn" => Some(Motion::Turn(1.1)),
        _ => None,
    }
}

pub(super) fn dress(
    commands: &mut Commands,
    art: &mut StrongholdArt,
    meshes: &mut Assets<Mesh>,
    root: Entity,
    tower: &Tower,
    look: Look,
    bytes: Option<&[u8]>,
) -> Option<Option<Turret>> {
    if blender::reference() {
        return None;
    }
    let key = (
        look.kind as u8,
        form_index(look.form()),
        tower.blue,
        tower.skin.map(|theme| theme as u8),
    );
    if let std::collections::hash_map::Entry::Vacant(entry) = art.blender.entry(key) {
        entry.insert(load(look, tower.blue, meshes, bytes?)?);
    }
    let model = art.blender.get(&key)?;
    for part in &model.parts {
        let mut entity = commands.spawn((
            Mesh3d(part.mesh.clone()),
            MeshMaterial3d(art.blender_material.clone()),
            part.rest,
            ChildOf(root),
            Wear::new(0, part.hide),
        ));
        if let Some(motion) = part.motion {
            entity.insert(Moving {
                motion,
                rest: part.rest,
                axle: Vec3::NEG_Z,
            });
        }
    }
    Some(model.turret)
}
