//! Unit details for long-press on a shop card, and the three first-match
//! hints (shown once per browser).

use bevy::prelude::*;
use tbg_core::content::{def, ENERGY_SCALE};
use tbg_core::UnitKind;

use super::Toasts;
use crate::i18n::{content, Lang, Msg, TIPS_HINT_TEXT};
use crate::{config, GameState};

/// Shop card details: stats from the content table, matchup lines from the
/// catalog (design intent, GAME-DESIGN §5), composed in `lang`.
pub fn unit_details(kind: UnitKind, lang: Lang) -> String {
    let d = def(kind);
    let attack = if d.heal > 0 {
        lang.tf(Msg::TipsHeals, &[("value", &d.heal)])
    } else {
        lang.tf(Msg::TipsDmg, &[("value", &d.damage)])
    };
    let (good, bad) = content::matchup(lang, kind);
    [
        lang.tf(
            Msg::TipsHead,
            &[
                ("name", &content::unit_name(lang, kind)),
                ("faction", &content::faction(lang, d.faction)),
                ("role", &content::archetype(lang, d.archetype)),
            ],
        ),
        lang.tf(
            Msg::TipsStats,
            &[
                ("hp", &d.hp),
                ("attack", &attack),
                ("every", &lang.decimal(f64::from(d.interval) / 20.0, 2)),
                ("range", &lang.decimal(f64::from(d.range) / 1000.0, 1)),
                ("cost", &(d.cost / ENERGY_SCALE)),
            ],
        ),
        lang.tf(Msg::TipsGood, &[("text", &good)]),
        lang.tf(Msg::TipsWeak, &[("text", &bad)]),
    ]
    .join("\n")
}

/// Seconds into the first match for each `tips.hint.{n}.text`.
const HINT_AT: [f64; 3] = [2.0, 14.0, 28.0];

#[derive(Resource)]
struct Hints {
    start: f64,
    next: usize,
}

pub fn plugin(app: &mut App) {
    app.add_systems(OnEnter(GameState::InMatch), start)
        .add_systems(Update, show.run_if(resource_exists::<Hints>));
}

fn start(mut commands: Commands, time: Res<Time>) {
    if !config::load_flag("tbg_hints_seen") {
        commands.insert_resource(Hints {
            start: time.elapsed_secs_f64(),
            next: 0,
        });
    }
}

fn show(
    mut commands: Commands,
    time: Res<Time>,
    mut hints: ResMut<Hints>,
    mut toasts: ResMut<Toasts>,
    lang: Res<Lang>,
) {
    let (Some(at), Some(text)) = (HINT_AT.get(hints.next), TIPS_HINT_TEXT.get(hints.next)) else {
        config::save_flag("tbg_hints_seen", true);
        commands.remove_resource::<Hints>();
        return;
    };
    if time.elapsed_secs_f64() - hints.start >= *at {
        toasts.push_for(lang.t(*text), 3.5);
        hints.next += 1;
    }
}
