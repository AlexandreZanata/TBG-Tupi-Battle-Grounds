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

impl From<tbg_core::Arena> for Theme {
    fn from(arena: tbg_core::Arena) -> Self {
        match arena {
            tbg_core::Arena::Forest => Self::Forest,
            tbg_core::Arena::Tundra => Self::Tundra,
            tbg_core::Arena::Volcanic => Self::Volcanic,
            tbg_core::Arena::Ruins => Self::Ruins,
            tbg_core::Arena::Storm => Self::Storm,
        }
    }
}

impl Theme {
    /// The scene outside a match: the `?theme`/`?arena` inspection override,
    /// else the saved Practice choice. A match always shows its own arena.
    pub fn selected() -> Self {
        let name = config::url_param("theme").or_else(|| config::url_param("arena"));
        name.as_deref()
            .and_then(tbg_core::Arena::from_slug)
            .unwrap_or_else(config::load_arena)
            .into()
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

    pub fn fog(self) -> FogFalloff {
        let (start, end) = match self {
            Self::Storm => (28.0, 52.0),
            Self::Volcanic => (30.0, 64.0),
            Self::Tundra => (34.0, 85.0),
            Self::Ruins => (36.0, 90.0),
            Self::Forest => (38.0, 95.0),
        };
        FogFalloff::Linear { start, end }
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
                ground: [rgb(0xe6f1f7), rgb(0xd7e8f1)],
                lane: [rgb(0x527d9d), rgb(0xb6d4e5), rgb(0xc4dfed), rgb(0xe0f2fa)],
                cliff: [rgb(0xb9d9e8), rgb(0x6db5d3), rgb(0x367eaa)],
                sea: rgb(0x176d9e),
                shore: [rgb(0xe3f5f6), rgb(0x9acbdc), rgb(0x5d98b4)],
                stone: rgb(0xa8c7d9),
                dark: rgb(0x3f596d),
                accent: rgb(0x86d9e8),
                plant: rgb(0x2d625a),
                wood: rgb(0x647984),
                sky: rgb(0xb2d2e0),
                key: rgb(0xe6f4ff),
                ambient: rgb(0xb4d4e9),
                fire: [rgb(0xff6420), rgb(0xffb83e), rgb(0xfff0bf)],
            },
            Self::Volcanic => Palette {
                ground: [rgb(0x403e49), rgb(0x4d4851)],
                lane: [rgb(0x282a36), rgb(0x77717a), rgb(0x89818a), rgb(0xbba0a0)],
                cliff: [rgb(0x4b424e), rgb(0x39333f), rgb(0x262632)],
                sea: rgb(0xe56520),
                shore: [rgb(0xffdc69), rgb(0xffa02e), rgb(0xef6b21)],
                stone: rgb(0x4c4b58),
                dark: rgb(0x2d2c3b),
                accent: rgb(0xffa247),
                plant: rgb(0x74535d),
                wood: rgb(0x45333e),
                sky: rgb(0x6f505d),
                key: rgb(0xffe1c4),
                ambient: rgb(0xc5c1cc),
                fire: [rgb(0xee4b25), rgb(0xffad3f), rgb(0xffe6a0)],
            },
            Self::Ruins => Palette {
                ground: [rgb(0x9cb865), rgb(0xb0c676)],
                lane: [rgb(0x9e9a77), rgb(0xd5c69f), rgb(0xcbbc98), rgb(0xefe1bc)],
                cliff: [rgb(0xc3bca5), rgb(0xaaa58d), rgb(0x7c8977)],
                sea: rgb(0x138ca8),
                shore: [rgb(0xd1eee1), rgb(0x4db8bf), rgb(0x2697ad)],
                stone: rgb(0xdfd4b5),
                dark: rgb(0x999b7f),
                accent: rgb(0x68d2c5),
                plant: rgb(0x668b45),
                wood: rgb(0x8a7a58),
                sky: rgb(0xd4d1b0),
                key: rgb(0xffefdc),
                ambient: rgb(0xc3d5d6),
                fire: [rgb(0x28bdd0), rgb(0x8ae5ed), rgb(0xe0fbff)],
            },
            Self::Storm => Palette {
                ground: [rgb(0x879aa9), rgb(0x95a8b6)],
                lane: [rgb(0x3c526b), rgb(0x98a7b6), rgb(0x889bae), rgb(0xc5cfd8)],
                cliff: [rgb(0x8d9fb3), rgb(0x607c98), rgb(0x3b526c)],
                sea: rgb(0x163c59),
                shore: [rgb(0xb1cfdf), rgb(0x6e96b2), rgb(0x486a89)],
                stone: rgb(0x718399),
                dark: rgb(0x394d66),
                accent: rgb(0xb0d9ec),
                plant: rgb(0x334f61),
                wood: rgb(0x56657a),
                sky: rgb(0x4c6380),
                key: rgb(0xd3e2ff),
                ambient: rgb(0xafc5e7),
                fire: [rgb(0xff6420), rgb(0xffb83e), rgb(0xfff0bf)],
            },
        }
    }
}
