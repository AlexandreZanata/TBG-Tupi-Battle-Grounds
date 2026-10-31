//! Top HUD line: your name, blue HP bar, clock (plus a RANKED badge), red
//! HP bar, and the opponent's name. Names shrink and truncate on phones so
//! the line never wraps or leaves the screen.

use bevy::prelude::*;

use crate::i18n::{Lang, Localized, Msg};
use crate::sync::MatchInfo;

use super::hud::{bar, Hud};
use super::{label, PANEL};

const YOU: Color = Color::srgb_u8(0x9C, 0xD2, 0xFF);
const ENEMY: Color = Color::srgb_u8(0xFF, 0xB4, 0xB4);

pub(super) fn spawn(parent: &mut ChildSpawnerCommands, info: &MatchInfo, lang: Lang, narrow: bool) {
    let bar_w = if narrow { 70.0 } else { 110.0 };
    let font = if narrow { 12.0 } else { 14.0 };
    let name_w = if narrow { 96.0 } else { 190.0 };
    let max_chars = if narrow { 14 } else { 23 };
    let clock = if narrow { 18.0 } else { 22.0 };
    parent
        .spawn((
            Node {
                flex_direction: FlexDirection::Row,
                column_gap: px(if narrow { 4.0 } else { 10.0 }),
                align_items: AlignItems::Center,
                padding: UiRect::axes(px(8.0), px(4.0)),
                border_radius: BorderRadius::all(px(8.0)),
                ..default()
            },
            BackgroundColor(PANEL),
        ))
        .with_children(|row| {
            row.spawn(name(
                &fit(&info.you_name, info.you_rating, max_chars),
                font,
                name_w,
                YOU,
            ));
            row.spawn(bar(
                Hud::Hp(0),
                Color::srgb_u8(0x3F, 0xA9, 0xF5),
                bar_w,
                12.0,
            ));
            row.spawn(Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                ..default()
            })
            .with_children(|mid| {
                mid.spawn((label("3:00", clock), Hud::Clock));
                if info.ranked {
                    mid.spawn((
                        label(lang.t(Msg::HudRankedBadge), 9.0),
                        Localized(Msg::HudRankedBadge),
                    ));
                }
            });
            row.spawn(bar(
                Hud::Hp(1),
                Color::srgb_u8(0xF2, 0x54, 0x5B),
                bar_w,
                12.0,
            ));
            row.spawn(name(
                &fit(&info.opponent, info.opponent_rating, max_chars),
                font,
                name_w,
                ENEMY,
            ));
        });
}

fn name(text: &str, size: f32, max_w: f32, color: Color) -> impl Bundle {
    (
        Node {
            max_width: px(max_w),
            overflow: Overflow::clip(),
            ..default()
        },
        Text::new(text),
        TextFont {
            font_size: FontSize::Px(size),
            ..default()
        },
        TextColor(color),
        TextLayout::no_wrap(),
    )
}

/// `Name (rating)` within `max_chars`, truncating the name with ASCII dots.
fn fit(name: &str, rating: Option<u16>, max_chars: usize) -> String {
    let suffix = rating.map(|r| format!(" ({r})")).unwrap_or_default();
    let room = max_chars.saturating_sub(suffix.chars().count());
    if name.chars().count() <= room {
        return format!("{name}{suffix}");
    }
    let kept: String = name.chars().take(room.saturating_sub(3)).collect();
    format!("{kept}...{suffix}")
}

#[cfg(test)]
mod tests {
    use super::fit;

    #[test]
    fn keeps_short_names_and_fits_long_ones() {
        assert_eq!(fit("Ana", None, 14), "Ana");
        assert_eq!(fit("Ana", Some(1034), 23), "Ana (1034)");
        let long = fit("LongNameSixteen!", Some(1034), 14);
        assert_eq!(long.chars().count(), 14);
        assert!(long.ends_with("... (1034)"));
    }
}
