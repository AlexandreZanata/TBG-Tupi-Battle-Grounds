//! Names the packs selected by existing Arena, seat and Fortification state.

use crate::arena::theme::Theme;
use tbg_core::{auxiliary::AuxiliaryKind, Seat, StrongholdKind};

fn theme_name(theme: Theme) -> &'static str {
    match theme {
        Theme::Forest => "forest",
        Theme::Tundra => "tundra",
        Theme::Volcanic => "volcanic",
        Theme::Ruins => "ruins",
        Theme::Storm => "storm",
        Theme::Celestial => "celestial",
        Theme::Clockwork => "clockwork",
        Theme::Desert => "desert",
    }
}

pub(in crate::arena) fn arena(theme: Theme) -> String {
    format!("arena/blender/{}.tbgd", theme_name(theme))
}

pub(in crate::arena) fn classic(theme: Theme, blue: bool) -> String {
    let seat = if blue { "blue" } else { "red" };
    format!("arena/blender/{}-classic-{seat}.tbgd", theme_name(theme))
}

pub(in crate::arena) fn auxiliary(theme: Theme, kind: AuxiliaryKind, seat: Seat) -> String {
    let kind = match kind {
        AuxiliaryKind::Miner => "miner",
        AuxiliaryKind::Merchant => "merchant",
    };
    let seat = match seat {
        Seat::A => "a",
        Seat::B => "b",
    };
    format!("arena/blender/{}-{kind}-{seat}.tbgd", theme_name(theme))
}

pub(in crate::arena) fn stronghold(
    kind: StrongholdKind,
    form: usize,
    blue: bool,
) -> Option<String> {
    let kind = match kind {
        StrongholdKind::Treasury => "treasury",
        StrongholdKind::Bastion => "bastion",
        StrongholdKind::Artillery => "artillery",
        StrongholdKind::Command => "command",
        StrongholdKind::Classic => return None,
    };
    let form = ["base", "a1", "a2", "b1", "b2"].get(form)?;
    let seat = if blue { "blue" } else { "red" };
    Some(format!("arena/blender/{kind}-{form}-{seat}.tbgd"))
}
