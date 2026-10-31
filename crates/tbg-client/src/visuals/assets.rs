//! Shared meshes and materials around the figurines (rings, rank pips, HP
//! bars, projectiles, tints), created once and reused by every unit.

use bevy::prelude::*;

#[derive(Resource)]
pub struct VisualAssets {
    pub ring: Handle<Mesh>,
    pub tier_cube: Handle<Mesh>,
    pub bar: Handle<Mesh>,
    pub projectile: Handle<Mesh>,
    pub team: [Handle<StandardMaterial>; 2],
    pub slowed: Handle<StandardMaterial>,
    pub rooted: Handle<StandardMaterial>,
    pub gold: Handle<StandardMaterial>,
    pub silver: Handle<StandardMaterial>,
    pub flash: Handle<StandardMaterial>,
    pub heal: Handle<StandardMaterial>,
    pub bar_bg: Handle<StandardMaterial>,
    pub bar_fill: [Handle<StandardMaterial>; 3],
    pub projectile_mat: Handle<StandardMaterial>,
}

/// Faction body colors (ARCHITECTURE §7 primary palette).
#[cfg(not(target_os = "linux"))]
pub const FACTION_COLORS: [[u8; 3]; 4] = [
    [0x8B, 0x73, 0x55],
    [0xD8, 0xD2, 0xC0],
    [0x4A, 0x50, 0x58],
    [0x4E, 0x9F, 0x3D],
];

pub(super) fn init(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut mats: ResMut<Assets<StandardMaterial>>,
) {
    let mut lit = |c: Color| {
        mats.add(StandardMaterial {
            base_color: c,
            perceptual_roughness: 0.85,
            ..default()
        })
    };
    let rgb = |c: [u8; 3]| Color::srgb_u8(c[0], c[1], c[2]);
    let team = [lit(rgb([0x3F, 0xA9, 0xF5])), lit(rgb([0xF2, 0x54, 0x5B]))];
    let slowed = lit(rgb([0x4C, 0x7D, 0xFF]));
    let rooted = lit(rgb([0x7A, 0x4E, 0x22]));
    let gold = lit(rgb([0xD9, 0xB4, 0x4A]));
    let silver = lit(rgb([0xC8, 0xD0, 0xD8]));
    let projectile_mat = lit(rgb([0xFF, 0xF3, 0xB0]));
    let mut unlit = |c: Color| {
        mats.add(StandardMaterial {
            base_color: c,
            unlit: true,
            ..default()
        })
    };
    let flash = unlit(Color::WHITE);
    let heal = unlit(rgb([0x9C, 0xFF, 0x9C]));
    let bar_bg = unlit(rgb([0x1D, 0x2B, 0x36]));
    let bar_fill = [
        rgb([0x5C, 0xD6, 0x5C]),
        rgb([0xF2, 0xC1, 0x4E]),
        rgb([0xF2, 0x54, 0x5B]),
    ]
    .map(&mut unlit);
    commands.insert_resource(VisualAssets {
        ring: meshes.add(Cylinder::new(0.42, 0.03).mesh().resolution(16)),
        tier_cube: meshes.add(Cuboid::new(0.12, 0.12, 0.12)),
        bar: meshes.add(Rectangle::new(0.8, 0.08)),
        projectile: meshes.add(Sphere::new(0.09)),
        team,
        slowed,
        rooted,
        gold,
        silver,
        flash,
        heal,
        bar_bg,
        bar_fill,
        projectile_mat,
    });
}
