//! Shared Troops catalog, event-free model viewer and isolated local training lifecycle.

pub(crate) mod catalog;
mod preview;
pub(crate) mod runtime;
mod trial_ui;

use crate::{art::anim::Clip, GameState};
use bevy::prelude::*;
use tbg_core::{
    training::{DummyHp, DummyMode, TrainingSession, TrainingUnitSpec},
    Archetype, Arena, Faction, UnitKind,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tab {
    Catalog,
    Editor,
    Practice,
}

#[derive(Clone, Copy, Debug)]
pub enum Action {
    Open(Tab),
    Select(UnitKind),
    Form(u8, u8),
    Clip(Clip),
    Tab(Tab),
    Search,
    Seed,
    Faction,
    Role,
    Rotate(i8),
    Zoom(i8),
    Motion,
    Add(bool),
    Remove(bool, usize),
    Move(bool, usize, i8),
    Opponent,
    DummyCount(i8),
    DummyHp,
    DummyMode,
    Arena,
    Start,
    Control(tbg_core::training::SessionId, u8),
    Back,
    Target(u32),
    Page(i8),
    Practice,
    Difficulty(u8),
}

#[derive(Resource)]
pub struct Catalog {
    pub seed: u64,
    pub selected: UnitKind,
    pub form: (u8, u8),
    pub clip: Clip,
    pub revision: u32,
    pub tab: Tab,
    pub search: String,
    pub faction: Option<Faction>,
    pub role: Option<Archetype>,
    pub yaw: f32,
    pub zoom: f32,
    pub reduced: bool,
    pub allies: Vec<TrainingUnitSpec>,
    pub enemies: Vec<TrainingUnitSpec>,
    pub dummies: bool,
    pub dummy_count: usize,
    pub dummy_hp: DummyHp,
    pub dummy_mode: DummyMode,
    pub arena: Arena,
    pub difficulty: u8,
    pub notice: String,
}
impl Default for Catalog {
    fn default() -> Self {
        Self {
            seed: 62,
            selected: UnitKind::BoneBrute,
            form: (0, 0),
            clip: Clip::Idle,
            revision: 0,
            tab: Tab::Catalog,
            search: String::new(),
            faction: None,
            role: None,
            yaw: 0.,
            zoom: 1.5,
            reduced: crate::config::reduced_motion(),
            allies: [UnitKind::BoneBrute, UnitKind::Hexer, UnitKind::Archer]
                .map(TrainingUnitSpec::base)
                .to_vec(),
            enemies: [UnitKind::Shieldbearer, UnitKind::Archer]
                .map(TrainingUnitSpec::base)
                .to_vec(),
            dummies: true,
            dummy_count: 1,
            dummy_hp: DummyHp::Light,
            dummy_mode: DummyMode::Finite,
            arena: crate::config::load_arena(),
            difficulty: crate::platform::storage::get(
                crate::platform::storage::Scope::Local,
                "practice.difficulty",
            )
            .and_then(|s| s.parse::<u8>().ok())
            .unwrap_or(1)
            .min(3),
            notice: String::new(),
        }
    }
}
impl Catalog {
    pub fn spec(&self) -> TrainingUnitSpec {
        TrainingUnitSpec {
            kind: self.selected,
            tier: self.form.0,
            branch: self.form.1,
            x: None,
        }
    }
    pub fn visible(&self) -> Vec<UnitKind> {
        let query = self.search.trim().to_lowercase();
        tbg_core::ALL_KINDS
            .into_iter()
            .filter(|kind| *kind != UnitKind::Crate)
            .filter(|kind| {
                let d = tbg_core::content::def(*kind);
                self.faction.is_none_or(|f| f == d.faction)
                    && self.role.is_none_or(|r| r == d.archetype)
                    && (query.is_empty() || d.name.to_lowercase().contains(&query))
            })
            .collect()
    }
}

#[derive(Resource, Default)]
pub struct Requests(pub Vec<Action>);

#[derive(Resource, Default)]
pub struct Trial {
    pub session: Option<TrainingSession>,
    pub dirty: bool,
    pub reset: bool,
    pub target: Option<u32>,
    pub page: usize,
    pub(crate) stats: crate::sync::MatchStats,
}

pub fn plugin(app: &mut App) {
    app.init_resource::<Catalog>()
        .init_resource::<Requests>()
        .init_resource::<Trial>();
    runtime::plugin(app);
    preview::plugin(app);
    trial_ui::plugin(app);
    app.add_systems(Update, invalidate.run_if(in_state(GameState::Menu)));
}
fn invalidate(catalog: Res<Catalog>, mut shell: ResMut<crate::ui::shell::Shell>) {
    if catalog.is_changed() && shell.panel == crate::ui::shell::Panel::Troops {
        shell.dirty = true;
    }
}
