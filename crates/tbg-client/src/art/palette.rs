//! Art-direction palette (ARCHITECTURE §7). Every figurine and arena color
//! comes from here.

use bevy::prelude::Color;

const fn hex(rgb: u32) -> Color {
    Color::srgb(
        ((rgb >> 16) & 0xff) as f32 / 255.0,
        ((rgb >> 8) & 0xff) as f32 / 255.0,
        (rgb & 0xff) as f32 / 255.0,
    )
}

// Humans: leather, light trim, gold, steel, skin.
pub const LEATHER: Color = hex(0x8B7355);
pub const TRIM: Color = hex(0xC4A882);
pub const GOLD: Color = hex(0xD9B44A);
pub const STEEL: Color = hex(0x9AA3AD);
pub const SKIN: Color = hex(0xE8B98A);
pub const HOOD_GREEN: Color = hex(0x5F7A3C);
pub const WOOD: Color = hex(0x7A5230);
pub const RED_CLOTH: Color = hex(0xB8433A);

// Undead: bone, dark violet, spectral glow.
pub const BONE: Color = hex(0xD8D2C0);
pub const VIOLET: Color = hex(0x4B3B5C);
pub const GLOW: Color = hex(0x7CF29A);
pub const DARK: Color = hex(0x2A2430);

// Machines: gunmetal, brass, orange.
pub const GUNMETAL: Color = hex(0x4A5058);
pub const BRASS: Color = hex(0xB5893B);
pub const ORANGE: Color = hex(0xFF8A3D);
pub const IRON: Color = hex(0x6E7681);

// Plants: leaf, bark, blossom.
pub const LEAF: Color = hex(0x4E9F3D);
pub const LEAF_DARK: Color = hex(0x2F6B2A);
pub const BARK: Color = hex(0x6B4F2E);
pub const BLOSSOM: Color = hex(0xF28AB2);
pub const MUSHROOM: Color = hex(0xC85C5C);

// Arena.
pub const GRASS: Color = hex(0x7BB661);
pub const LANE: Color = hex(0xC9A46A);
pub const LANE_DARK: Color = hex(0xB8925A);
pub const CLIFF: Color = hex(0x8B6B4A);
pub const WATER: Color = hex(0x3FA7B5);
pub const ROCK: Color = hex(0x8C8A86);
pub const STONE: Color = hex(0xB9B2A3);
pub const PINE: Color = hex(0x3E7A3A);
pub const SKY: Color = hex(0xA9D8F0);
pub const ALLY: Color = hex(0x3FA9F5);
pub const ENEMY: Color = hex(0xF2545B);

// Medieval additions, ART-DIRECTION §§2–3. Legacy aliases above stay for the adapter.
pub const OAK: Color = hex(0x7A5636);
pub const LINEN: Color = hex(0xE6D8B8);
pub const RUST: Color = hex(0x6E4B3A);
pub const DARK_LEATHER: Color = hex(0x5C3B26);
pub const OGRE_SKIN: Color = hex(0x8C9A6B);
pub const SAP: Color = hex(0xF2B544);
pub const TIMBER: Color = hex(0x8A6A44);
pub const SKY_TOP: Color = hex(0x7FB8E6);
pub const HORIZON: Color = hex(0xE8D9B5);
pub const PARCHMENT: Color = hex(0xEFE3C6);
pub const INK: Color = hex(0x2B2118);
pub const UI_GOLD: Color = hex(0xC9A13B);
pub const FRAME: Color = hex(0x3A3F45);
pub const DANGER: Color = hex(0xC0392B);
pub const OK: Color = hex(0x3E8E41);
pub const KEY: Color = hex(0xFFE9C7);
pub const AMBIENT_SKY: Color = hex(0xA9C8E8);
pub const AMBIENT_GROUND: Color = hex(0x6B5A43);

#[derive(Clone, Copy, Debug)]
pub enum PaletteSlot {
    Base,
    Shade,
    Accent,
    Linen,
    Team,
    Skin,
    Metal,
    Glow,
}

pub const FACTIONS: [[Color; 8]; 4] = [
    [STEEL, OAK, GOLD, LINEN, ALLY, SKIN, STEEL, GOLD],
    [BONE, RUST, GLOW, VIOLET, ENEMY, BONE, RUST, GLOW],
    [
        GUNMETAL,
        BRASS,
        ORANGE,
        DARK_LEATHER,
        ALLY,
        OGRE_SKIN,
        GUNMETAL,
        ORANGE,
    ],
    [BARK, LEAF, BLOSSOM, LEAF, ENEMY, BARK, SAP, SAP],
];
pub const SLOTS: [PaletteSlot; 8] = [
    PaletteSlot::Base,
    PaletteSlot::Shade,
    PaletteSlot::Accent,
    PaletteSlot::Linen,
    PaletteSlot::Team,
    PaletteSlot::Skin,
    PaletteSlot::Metal,
    PaletteSlot::Glow,
];
pub const ENVIRONMENT: [Color; 8] = [GRASS, LANE, WATER, STONE, TIMBER, SKY_TOP, HORIZON, SAP];
pub const UI: [Color; 8] = [
    PARCHMENT,
    INK,
    UI_GOLD,
    FRAME,
    DANGER,
    OK,
    AMBIENT_SKY,
    AMBIENT_GROUND,
];
