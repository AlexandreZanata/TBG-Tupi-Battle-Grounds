//! Unit status on click: any unit, own or enemy, shows a discreet panel with
//! its live HP and effective stats (form, Synergy, status). The numbers come
//! from the latest authoritative view and the shared content tables, so the
//! panel follows damage in real time. Desktop draws it in the HTML battle
//! layer (`battle_hud`); smaller screens get the Bevy text panel below.

use bevy::prelude::*;
use tbg_core::content::{def, form_of, TICK_HZ};
use tbg_core::{ArcanaPreset, Ruleset};
use tbg_core::{MatchView, OnHit, UnitId};

use super::{label, UiAction, PANEL};
use crate::sync::LatestView;
use crate::GameState;

/// The unit whose status panel is open.
#[derive(Resource, Default)]
pub struct Inspected(pub Option<UnitId>);

/// Display-ready status of one unit.
#[derive(Debug, PartialEq)]
pub struct Report {
    pub kind: String,
    pub name: &'static str,
    pub form: String,
    pub faction: String,
    pub role: String,
    pub own: bool,
    pub hp: i32,
    pub max_hp: i32,
    /// "Damage" or "Heal" and its value per hit.
    pub attack: (&'static str, i32),
    pub every: String,
    pub per_second: String,
    pub range: String,
    pub speed: String,
    pub splash: String,
    pub on_hit: String,
    pub status: &'static str,
    pub synergy: String,
}

fn tiles(milli: i32) -> String {
    format!("{:.1}", milli as f32 / 1000.0)
}

/// `None` once the unit has left the view (dead or gone). `arcanas` are
/// both frozen presets from `MatchStart` (D-40), so damage and healing
/// match what the server applies.
pub fn report(v: &MatchView, id: UnitId, arcanas: &[ArcanaPreset; 2]) -> Option<Report> {
    let u = v.units.iter().find(|u| u.id == id)?;
    let d = def(u.kind);
    // Summons never receive Synergy (BR-U08).
    let level = if u.kind.is_summon() {
        0
    } else {
        v.synergy[u.seat.index()][d.faction.index()]
    };
    // Summons never receive Arcanas either (D-40).
    let preset = if u.kind.is_summon() {
        ArcanaPreset::NEUTRAL
    } else {
        arcanas[u.seat.index()]
    };
    let bonus = tbg_core::augment::unit_bonus(
        &v.augments[u.seat.index()],
        v.synergy[u.seat.index()],
        u.kind,
        u.tier,
    );
    let st = Ruleset::canon().stats_with(u.kind, u.tier, u.branch, level, &preset, &bonus);
    let seconds = f32::from(st.interval) / TICK_HZ as f32;
    let attack = if st.heal > 0 && st.damage == 0 {
        ("Heal", st.heal)
    } else {
        ("Damage", st.damage)
    };
    Some(Report {
        kind: format!("{:?}", u.kind),
        name: d.name,
        form: match form_of(u.kind, u.tier, u.branch) {
            Some(f) => format!("{} | tier {}", f.name, u.tier),
            None => "Base form".into(),
        },
        faction: format!("{:?}", d.faction),
        role: match d.archetype {
            tbg_core::Archetype::Dps => "DPS".into(),
            a => format!("{a:?}"),
        },
        own: u.seat == v.you,
        hp: u.hp.max(0),
        max_hp: u.max_hp.max(1),
        attack,
        every: format!("{seconds:.2} s"),
        per_second: format!("{:.0}", attack.1 as f32 / seconds.max(0.05)),
        range: tiles(st.range),
        speed: tiles(st.speed * TICK_HZ as i32),
        splash: if st.splash > 0 {
            tiles(st.splash)
        } else {
            "-".into()
        },
        on_hit: match st.on_hit {
            Some(OnHit::Slow { bp, ticks }) => format!(
                "Slow {}% {:.1} s",
                bp / 100,
                f32::from(ticks) / TICK_HZ as f32
            ),
            Some(OnHit::Root { ticks }) => {
                format!("Root {:.1} s", f32::from(ticks) / TICK_HZ as f32)
            }
            None => "-".into(),
        },
        status: match u.status {
            1 => "Slowed",
            2 => "Rooted",
            _ => "Ready",
        },
        synergy: if level > 0 {
            format!("{:?} {level}/2", d.faction)
        } else {
            "-".into()
        },
    })
}

#[cfg(any(target_arch = "wasm32", test))]
impl Report {
    /// One `|`-separated record for the HTML battle layer (no field holds `|`).
    pub fn record(&self) -> String {
        [
            self.kind.clone(),
            self.name.to_string(),
            self.form.clone(),
            self.faction.clone(),
            self.role.clone(),
            self.own.to_string(),
            self.hp.to_string(),
            self.max_hp.to_string(),
            self.attack.0.to_string(),
            self.attack.1.to_string(),
            self.every.clone(),
            self.per_second.clone(),
            self.range.clone(),
            self.speed.clone(),
            self.splash.clone(),
            self.on_hit.clone(),
            self.status.to_string(),
            self.synergy.clone(),
        ]
        .join("|")
    }
}

#[derive(Component)]
struct InspectPanel;

#[derive(Component)]
struct InspectText;

pub fn plugin(app: &mut App) {
    app.init_resource::<Inspected>()
        .add_systems(OnExit(GameState::InMatch), clear)
        .add_systems(
            Update,
            (drop_gone, fallback_panel)
                .chain()
                .run_if(in_state(GameState::InMatch)),
        );
}

fn clear(mut inspected: ResMut<Inspected>) {
    inspected.0 = None;
}

/// Close the panel when the unit dies or leaves the view.
fn drop_gone(mut inspected: ResMut<Inspected>, view: Res<LatestView>) {
    let (Some(id), Some(v)) = (inspected.0, &view.0) else {
        return;
    };
    if !v.units.iter().any(|u| u.id == id) {
        inspected.0 = None;
    }
}

/// Text panel for screens without the HTML battle layer (phones, native).
fn fallback_panel(
    mut commands: Commands,
    inspected: Res<Inspected>,
    view: Res<LatestView>,
    info: Res<crate::sync::MatchInfo>,
    desktop: Res<super::battle_hud::DesktopBattle>,
    panels: Query<Entity, With<InspectPanel>>,
    mut texts: Query<&mut Text, With<InspectText>>,
) {
    let report = match (inspected.0, &view.0) {
        (Some(id), Some(v)) if !desktop.0 => report(v, id, &info.arcanas),
        _ => None,
    };
    let Some(r) = report else {
        for e in &panels {
            commands.entity(e).despawn();
        }
        return;
    };
    let text = format!(
        "{} ({})\n{} | {} {}\nHP {} / {}\n{} {} every {} | range {}\nspeed {} | splash {} | {}\n{}",
        r.name,
        if r.own { "yours" } else { "enemy" },
        r.form,
        r.faction,
        r.role,
        r.hp,
        r.max_hp,
        r.attack.0,
        r.attack.1,
        r.every,
        r.range,
        r.speed,
        r.splash,
        r.on_hit,
        r.status
    );
    if let Ok(mut t) = texts.single_mut() {
        if t.0 != text {
            t.0 = text;
        }
        return;
    }
    commands.spawn((
        InspectPanel,
        // Tapping the panel closes it (ui/input.rs hit-tests UiAction nodes).
        UiAction::CloseInspect,
        Node {
            position_type: PositionType::Absolute,
            left: px(10.0),
            top: px(96.0),
            padding: UiRect::all(px(8.0)),
            border_radius: BorderRadius::all(px(8.0)),
            ..default()
        },
        BackgroundColor(PANEL),
        children![(label(text, 13.0), InspectText)],
    ));
}

#[cfg(test)]
mod tests {
    use tbg_core::{MatchState, Phase, PlayerIntention, Seat, UnitKind};

    #[test]
    fn report_follows_live_hp_for_own_and_enemy_units() {
        let mut s = MatchState::new(3);
        while s.phase != Phase::Running {
            s.step();
        }
        for seat in Seat::BOTH {
            s.players[seat.index()].offers[0] = UnitKind::Hexer;
            s.players[seat.index()].energy = 6_000;
            s.apply_intention(seat, 1, PlayerIntention::Purchase { offer: 0 })
                .unwrap();
        }
        let (mine, theirs) = (s.units[0].id, s.units[1].id);
        s.units[1].hp = 123;
        let v = s.view_for(Seat::A, &[]);
        let neutral = [tbg_core::ArcanaPreset::NEUTRAL; 2];
        let own = super::report(&v, mine, &neutral).unwrap();
        let enemy = super::report(&v, theirs, &neutral).unwrap();
        assert!(own.own && !enemy.own);
        assert_eq!((enemy.hp, enemy.max_hp), (123, 300));
        // The canonical value, so a balance pass (D-33) does not break the UI test.
        let damage = tbg_core::content::def(UnitKind::Hexer).damage;
        assert_eq!(own.attack, ("Damage", damage));
        assert_eq!(own.on_hit, "Slow 40% 2.0 s");
        assert_eq!((own.every.as_str(), own.range.as_str()), ("1.40 s", "3.5"));
        assert!(
            super::report(&v, 999, &neutral).is_none(),
            "gone units close the panel"
        );
        let record = enemy.record();
        assert_eq!(record.split('|').count(), 18, "{record}");
        assert!(record.starts_with("Hexer|Hexer|Base form|Undead|Control|false|123|300|"));
    }
}
