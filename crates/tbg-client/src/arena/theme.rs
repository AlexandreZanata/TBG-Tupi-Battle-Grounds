//! Presentation-only arena identities and vertex-color palettes; no simulation data.

use crate::{art::palette::*, config};
use bevy::prelude::*;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum Theme {
    #[default]
    Forest,
    Tundra,
    Volcanic,
    Ruins,
    Storm,
}

pub(crate) struct Palette {
    pub ground: [Color; 2],
    pub lane: [Color; 4],
    pub cliff: [Color; 3],
    pub sea: Color,
    pub shore: [Color; 3],
    pub stone: Color,
    pub dark: Color,
    pub accent: Color,
    pub plant: Color,
    pub wood: Color,
    pub sky: Color,
    pub key: Color,
    pub ambient: Color,
    pub fire: [Color; 3],
}

pub(super) const fn rgb(hex: u32) -> Color {
    Color::srgb(
        ((hex >> 16) & 255) as f32 / 255.0,
        ((hex >> 8) & 255) as f32 / 255.0,
        (hex & 255) as f32 / 255.0,
    )
}

impl Theme {
    pub fn selected() -> Self {
        let name = config::url_param("theme").or_else(|| config::url_param("arena"));
        match name.as_deref() {
            Some("tundra") => Self::Tundra,
            Some("volcanic") => Self::Volcanic,
            Some("ruins") => Self::Ruins,
            Some("storm") => Self::Storm,
            _ => Self::Forest,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Forest => "Verdant Forest",
            Self::Tundra => "Frozen Tundra",
            Self::Volcanic => "Volcanic Forge",
            Self::Ruins => "Ancient Ruins",
            Self::Storm => "Storm Valley",
        }
    }

    pub fn palette(self) -> Palette {
        match self {
            Self::Forest => Palette {
                ground: [Color::srgb(0.47, 0.65, 0.20), Color::srgb(0.53, 0.70, 0.24)],
                lane: [
                    LANE_DARK,
                    LANE,
                    Color::srgb(0.82, 0.64, 0.37),
                    Color::srgb(0.89, 0.73, 0.46),
                ],
                cliff: [ROCK; 3],
                sea: WATER,
                shore: [
                    Color::srgb(0.72, 0.91, 0.85),
                    Color::srgb(0.35, 0.75, 0.74),
                    Color::srgb(0.26, 0.69, 0.72),
                ],
                stone: STONE,
                dark: ROCK,
                accent: GOLD,
                plant: PINE,
                wood: WOOD,
                sky: SKY,
                key: KEY,
                ambient: AMBIENT_SKY,
                fire: [
                    Color::srgb(1.0, 0.32, 0.025),
                    Color::srgb(1.0, 0.81, 0.14),
                    LINEN,
                ],
            },
            Self::Tundra => Palette {
                ground: [rgb(0xd9e9ee), rgb(0xc3dbe5)],
                lane: [rgb(0x527688), rgb(0xa6c1d0), rgb(0xb8d1dc), rgb(0xd9edf1)],
                cliff: [rgb(0x93b3c6), rgb(0x6a8a9f), rgb(0x3d596e)],
                sea: rgb(0x345e7b),
                shore: [rgb(0xe3f5f6), rgb(0x9acbdc), rgb(0x5d98b4)],
                stone: rgb(0x95b6c9),
                dark: rgb(0x3f596d),
                accent: rgb(0x86d9e8),
                plant: rgb(0x416f78),
                wood: rgb(0x647984),
                sky: rgb(0xb2d2e0),
                key: rgb(0xe6f4ff),
                ambient: rgb(0xb4d4e9),
                fire: [rgb(0x4caecb), rgb(0xb2ebf3), rgb(0xf0ffff)],
            },
            Self::Volcanic => Palette {
                ground: [rgb(0x51494e), rgb(0x605157)],
                lane: [rgb(0x282a36), rgb(0x77717a), rgb(0x89818a), rgb(0xbba0a0)],
                cliff: [rgb(0x594652), rgb(0x3e3544), rgb(0x282735)],
                sea: rgb(0xab3927),
                shore: [rgb(0xffc367), rgb(0xf77b33), rgb(0xdb4e28)],
                stone: rgb(0x514956),
                dark: rgb(0x2d2c3b),
                accent: rgb(0xffa247),
                plant: rgb(0x74535d),
                wood: rgb(0x45333e),
                sky: rgb(0x6f505d),
                key: rgb(0xffd6b0),
                ambient: rgb(0xc0adb9),
                fire: [rgb(0xee4b25), rgb(0xffad3f), rgb(0xffe6a0)],
            },
            Self::Ruins => Palette {
                ground: [rgb(0x9ea078), rgb(0xadaa83)],
                lane: [rgb(0x737963), rgb(0xc3b899), rgb(0xb7ac8d), rgb(0xe0d3b2)],
                cliff: [rgb(0xa89d80), rgb(0x89876e), rgb(0x5b685a)],
                sea: rgb(0x456d66),
                shore: [rgb(0xafc7a3), rgb(0x7caa94), rgb(0x598b7e)],
                stone: rgb(0xc4b996),
                dark: rgb(0x777960),
                accent: rgb(0x65b6a7),
                plant: rgb(0x657c4b),
                wood: rgb(0x8a7a58),
                sky: rgb(0xd4d1b0),
                key: rgb(0xffedce),
                ambient: rgb(0xd0d8bd),
                fire: [rgb(0x438f83), rgb(0x9ce0b8), rgb(0xe0f6d3)],
            },
            Self::Storm => Palette {
                ground: [rgb(0x63788a), rgb(0x718596)],
                lane: [rgb(0x3c526b), rgb(0x98a7b6), rgb(0x889bae), rgb(0xc5cfd8)],
                cliff: [rgb(0x73859b), rgb(0x52667e), rgb(0x36465e)],
                sea: rgb(0x2d455f),
                shore: [rgb(0xb1cfdf), rgb(0x6e96b2), rgb(0x486a89)],
                stone: rgb(0x718399),
                dark: rgb(0x394d66),
                accent: rgb(0xb0d9ec),
                plant: rgb(0x4b6275),
                wood: rgb(0x56657a),
                sky: rgb(0x7a8caa),
                key: rgb(0xd3e2ff),
                ambient: rgb(0xafc5e7),
                fire: [rgb(0x648dd6), rgb(0xb8dafa), rgb(0xe9f5ff)],
            },
        }
    }
}
