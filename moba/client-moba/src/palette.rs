//! Shared colours of the MOBA presentation (world and HUD).

use bevy::prelude::*;
use tbg_moba_core::Team;

pub const BLUE: Color = Color::srgb(0.20, 0.52, 0.95);
pub const BLUE_DARK: Color = Color::srgb(0.10, 0.22, 0.48);
pub const BLUE_GLOW: Color = Color::srgb(0.35, 0.75, 1.0);
pub const RED: Color = Color::srgb(0.88, 0.20, 0.18);
pub const RED_DARK: Color = Color::srgb(0.45, 0.09, 0.08);
pub const RED_GLOW: Color = Color::srgb(1.0, 0.35, 0.25);
pub const NEUTRAL: Color = Color::srgb(0.85, 0.55, 0.20);
pub const GOLD: Color = Color::srgb(0.96, 0.76, 0.26);
pub const GOLD_DARK: Color = Color::srgb(0.55, 0.40, 0.12);
pub const TEXT: Color = Color::srgb(0.95, 0.93, 0.88);
pub const MUTED: Color = Color::srgb(0.66, 0.68, 0.72);
pub const PANEL: Color = Color::srgba(0.06, 0.08, 0.12, 0.88);
pub const PANEL_LIGHT: Color = Color::srgba(0.12, 0.15, 0.21, 0.94);
pub const HP_ALLY: Color = Color::srgb(0.30, 0.85, 0.35);
pub const HP_ENEMY: Color = Color::srgb(0.92, 0.25, 0.22);
pub const SKY: Color = Color::srgb(0.07, 0.10, 0.13);

pub fn team(team: Team) -> Color {
    match team {
        Team::Blue => BLUE,
        Team::Red => RED,
    }
}

pub fn team_dark(team: Team) -> Color {
    match team {
        Team::Blue => BLUE_DARK,
        Team::Red => RED_DARK,
    }
}

pub fn glow(team: Team) -> Color {
    match team {
        Team::Blue => BLUE_GLOW,
        Team::Red => RED_GLOW,
    }
}
