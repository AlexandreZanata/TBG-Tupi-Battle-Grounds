//! Unit details for long-press on a shop card, and the three first-match
//! hints (shown once per browser).

use bevy::prelude::*;
use tbg_core::content::{def, ENERGY_SCALE};
use tbg_core::UnitKind;

use super::Toasts;
use crate::{config, GameState};

/// Who each unit is good and bad against (design intent, GAME-DESIGN §5).
fn matchup(kind: UnitKind) -> (&'static str, &'static str) {
    match kind {
        UnitKind::Shieldbearer => ("holds melee rushes", "slow damage, splash"),
        UnitKind::Archer => ("fragile melee behind a tank", "Sawbot dives, splash"),
        UnitKind::BoneBrute => ("long fights, splash", "Hexer slows, focused DPS"),
        UnitKind::Hexer => ("fast melee (Sawbot)", "ranged and splash"),
        UnitKind::Sawbot => ("Archers, Supports", "tanks, roots, slows"),
        UnitKind::SiegeWalker => ("groups, Strongholds", "fast melee up close"),
        UnitKind::Bloomkeeper => ("drawn-out fights", "burst damage"),
        UnitKind::Rootcaller => ("fast melee, chargers", "ranged poke"),
        UnitKind::Nester => (
            "long fights, walls of Beetlings",
            "splash, burst on the Nester",
        ),
        UnitKind::Rampart => (
            "shielding fragile ranged units",
            "slows, ranged focus, splash",
        ),
        UnitKind::Beetling => ("swarming", "splash"),
        // P44 wave A (staged; §11 identity lines, not in the Shop yet).
        UnitKind::Longaxe => ("second-rank heavies", "swarms, burst"),
        UnitKind::Farwatch => ("safe chip from home", "Rampart guard, dives"),
        UnitKind::Bombard => ("clumped armies", "spread out, dives"),
        // P45 wave B (staged; §11 identity lines, not in the Shop yet).
        UnitKind::Rotcap => ("tanky fronts", "burst, cleanse"),
        UnitKind::OldFinn => ("melee approaches", "ranged armies, cleanse"),
        UnitKind::AntidoteBrewer => ("control and poison", "pure damage"),
        // P46 wave C (staged; §11 identity lines, not in the Shop yet).
        UnitKind::Shatterback => ("long fights", "burst before plates break"),
        UnitKind::BloodFurnace => ("clumped melee", "focus it early, control"),
        UnitKind::Bloodweaver => ("armies that trade HP", "burst, heal cut"),
        // P47 wave D (staged; §11 identity lines, not in the Shop yet).
        UnitKind::Oilpress => ("protect the line", "go around, burst it down"),
        UnitKind::Drumguard => ("one protected carry", "kill the drummer"),
        // P48-A wave E1 (staged; §11 identity lines, not in the Shop yet).
        UnitKind::Leechroot => ("left alone too long", "burst it, splash it"),
        UnitKind::CargoCrab => ("one protected carry", "splash the crab"),
        // P48-B wave E2 (staged; §11 identity lines, not in the Shop yet).
        UnitKind::Puppetmaster => ("your best hitter", "spread out, burst it"),
        UnitKind::Smuggler => ("ammo-hungry ranged", "kill the crates, dive it"),
        UnitKind::Crate => ("free ammo", "deny it"),
        // P43: staged kinds render as a blockout until their wave lands.
        #[allow(unreachable_patterns)]
        _ => ("a new unit", "scout it first"),
    }
}

pub fn unit_details(kind: UnitKind) -> String {
    let d = def(kind);
    let attack = if d.heal > 0 {
        format!("heals {}", d.heal)
    } else {
        format!("{} dmg", d.damage)
    };
    let (good, bad) = matchup(kind);
    format!(
        "{} ({:?}, {:?})\nHP {} | {} every {:.2}s | range {:.1} | cost {}\nGood vs {good}\nWeak vs {bad}",
        d.name,
        d.faction,
        d.archetype,
        d.hp,
        attack,
        f32::from(d.interval) / 20.0,
        d.range as f32 / 1000.0,
        d.cost / ENERGY_SCALE,
    )
}

const HINTS: [(f64, &str); 3] = [
    (2.0, "Tap a card to deploy that unit"),
    (14.0, "Tap one of your units to evolve it"),
    (28.0, "2+ units of one faction trigger a Synergy bonus"),
];

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
) {
    let Some((at, text)) = HINTS.get(hints.next) else {
        config::save_flag("tbg_hints_seen", true);
        commands.remove_resource::<Hints>();
        return;
    };
    if time.elapsed_secs_f64() - hints.start >= *at {
        toasts.push_for(*text, 3.5);
        hints.next += 1;
    }
}
