//! Per-arena color palettes for the `Mineiro` and `Mercador` 3D precincts (P74).
//! Harmonizes rock outcrops, timber shoring, glowing ore, piers, and ships with all 8 Arenas.

use crate::arena::theme::Theme;
use bevy::prelude::*;

#[derive(Clone, Copy, Debug)]
pub(super) struct AuxPalette {
    pub rock_outer: Color,
    pub rock_inner: Color,
    pub tunnel_void: Color,
    pub timber_post: Color,
    pub timber_beam: Color,
    pub iron_rail: Color,
    pub ore_crystal: Color,
    pub lantern_glow: Color,
    pub pier_wood: Color,
    pub pier_piling: Color,
    pub awning_red: Color,
    pub awning_cream: Color,
    pub hull_wood: Color,
    pub sail_cloth: Color,
    pub trim_gold: Color,
    pub crate_wood: Color,
    pub sack_cloth: Color,
}

const fn rgb(r: u8, g: u8, b: u8) -> Color {
    Color::srgb(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0)
}

impl AuxPalette {
    pub fn for_theme(theme: Theme) -> Self {
        let p = theme.palette();
        let (rock_outer, rock_inner, ore_crystal, pier_wood, pier_piling) = match theme {
            Theme::Forest => (
                rgb(112, 118, 124),
                rgb(82, 88, 94),
                rgb(255, 206, 68),
                rgb(138, 94, 52),
                rgb(92, 62, 34),
            ),
            Theme::Tundra => (
                rgb(126, 144, 162),
                rgb(88, 106, 126),
                rgb(110, 228, 255),
                rgb(122, 96, 72),
                rgb(82, 64, 48),
            ),
            Theme::Volcanic => (
                rgb(68, 58, 62),
                rgb(46, 38, 42),
                rgb(255, 124, 36),
                rgb(94, 62, 46),
                rgb(58, 38, 30),
            ),
            Theme::Ruins => (
                rgb(134, 128, 114),
                rgb(98, 92, 82),
                rgb(124, 242, 176),
                rgb(132, 102, 68),
                rgb(92, 70, 46),
            ),
            Theme::Storm => (
                rgb(86, 96, 114),
                rgb(58, 66, 82),
                rgb(136, 198, 255),
                rgb(104, 82, 62),
                rgb(68, 52, 40),
            ),
            Theme::Celestial => (
                rgb(212, 206, 192),
                rgb(168, 160, 146),
                rgb(255, 224, 112),
                rgb(168, 132, 86),
                rgb(122, 94, 58),
            ),
            Theme::Clockwork => (
                rgb(118, 108, 96),
                rgb(84, 76, 68),
                rgb(255, 196, 64),
                rgb(142, 96, 54),
                rgb(96, 64, 36),
            ),
            Theme::Desert => (
                rgb(194, 152, 104),
                rgb(154, 116, 74),
                rgb(86, 232, 216),
                rgb(156, 112, 66),
                rgb(112, 78, 44),
            ),
        };
        Self {
            rock_outer,
            rock_inner,
            tunnel_void: rgb(18, 16, 22),
            timber_post: pier_piling,
            timber_beam: p.wood,
            iron_rail: rgb(92, 100, 112),
            ore_crystal,
            lantern_glow: rgb(255, 214, 92),
            pier_wood,
            pier_piling,
            awning_red: rgb(198, 48, 42),
            awning_cream: rgb(244, 232, 206),
            hull_wood: rgb(116, 74, 40),
            sail_cloth: rgb(246, 238, 218),
            trim_gold: rgb(236, 188, 66),
            crate_wood: rgb(164, 114, 62),
            sack_cloth: rgb(208, 184, 138),
        }
    }
}
