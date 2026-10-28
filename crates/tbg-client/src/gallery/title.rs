//! Gallery heading: which unit, form, and tool is on screen.

use super::settings::Gallery;
use bevy::prelude::*;
use tbg_core::UnitKind;

pub fn spawn(commands: &mut Commands, options: &Gallery, unit: UnitKind) {
    let title = if options.lineup {
        format!("{unit:?}{}", form_label(options))
    } else if options.frames {
        format!(
            "{unit:?}{} | CONTACT SHEET | 0 / 25 / 50 / 75 %\n{}",
            form_label(options),
            options.clip.map_or_else(
                || "Idle / Walk | Melee / Ranged | Cast / Hit | Death / Spawn | Evolve / Victory"
                    .into(),
                |c| format!("{c:?}")
            )
        )
    } else {
        format!("P15 GALLERY | {} | {}x\nBase / A1 / A2 / B1 / B2 | Idle, Walk, attacks, cast, hit, death, spawn, evolve, victory", options.mode, options.slow)
    };
    commands.spawn((
        Text::new(title),
        TextFont {
            font_size: FontSize::Px(16.0),
            ..default()
        },
        TextColor(crate::art::palette::INK),
        Node {
            position_type: PositionType::Absolute,
            top: px(12),
            left: px(12),
            max_width: percent(94),
            ..default()
        },
    ));
}

/// ` v2 Longbow` and the like when the `?archer=v2` candidate is on review.
fn form_label(options: &Gallery) -> String {
    let form = ["", " Longbow", " Marksman"][usize::from(options.tier.min(2))];
    let v2 = if crate::art::archer_v2::enabled() {
        " v2"
    } else {
        ""
    };
    if options.lineup {
        format!("{v2} | Base / Longbow / Marksman | idle + anchor")
    } else {
        format!("{v2}{form}")
    }
}
