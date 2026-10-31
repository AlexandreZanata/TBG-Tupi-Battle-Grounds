//! P30-A network/view audit harness (standard library + `tbg-core` only).
//!
//! Measures the current full snapshot, the field composition, frequency,
//! and four prototypes (A derive-don't-send, B delta, C static/dynamic,
//! D events-only negative control), rebuilding the canonical view from
//! every prototype on 100 seeded bot matches and 10/20/40/60/100-unit
//! synthetic armies. Prints Markdown and JSON to stdout.
//!
//! `cargo run --release -p tbg-core --example view_audit`

use std::alloc::{GlobalAlloc, Layout, System};
use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::sync::OnceLock;
use std::time::Instant;

use serde::Serialize;

use tbg_core::arcana::ArcanaPreset;
use tbg_core::augment::{self, AugmentCounts};
use tbg_core::bot::Bot;
use tbg_core::content::{self, OFFERS};
use tbg_core::protocol::{encode, ServerMsg};
use tbg_core::ruleset::Ruleset;
use tbg_core::view::{RoundView, FX_SUMMON};
use tbg_core::{
    Event, MatchState, MatchView, Phase, PlayerIntention, Seat, Status, StrongholdView, Tick,
    UnitId, UnitKind, UnitView,
};

const VIEW_EVERY: Tick = 2;
/// The pre-D-41 match end (countdown, regulation and overtime): matches are
/// open-ended now, so natural runs sample the same window P30-A measured.
const SAMPLE_END: Tick = 4_860;
const SYNTHETIC_TICKS: u32 = 1_200;
const NATURAL_SEEDS: u64 = 100;
/// A delta keyframe every 10 s of views.
const KEYFRAME_VIEWS: u32 = 100;
/// Estimated per-message wire overhead: 2–4 B WebSocket, ~22 B TLS record,
/// 40–52 B TCP/IP. The tables use the midpoint; the JSON has the range.
const WIRE_OVERHEAD_MIN: u64 = 64;
const WIRE_OVERHEAD_MAX: u64 = 78;
const WIRE_OVERHEAD_MID: u64 = 71;
/// `TBG_AUDIT_VIEW_DIR`: write raw encoded views for the compression check.
static VIEW_DUMP: OnceLock<Option<PathBuf>> = OnceLock::new();

static COUNT_ALLOC: AtomicBool = AtomicBool::new(false);
static ALLOC_CALLS: AtomicU64 = AtomicU64::new(0);
static ALLOC_BYTES: AtomicU64 = AtomicU64::new(0);

struct CountingAllocator;

unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if COUNT_ALLOC.load(Ordering::Relaxed) {
            ALLOC_CALLS.fetch_add(1, Ordering::Relaxed);
            ALLOC_BYTES.fetch_add(layout.size() as u64, Ordering::Relaxed);
        }
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) }
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        if COUNT_ALLOC.load(Ordering::Relaxed) {
            ALLOC_CALLS.fetch_add(1, Ordering::Relaxed);
            ALLOC_BYTES.fetch_add(new_size as u64, Ordering::Relaxed);
        }
        unsafe { System.realloc(ptr, layout, new_size) }
    }
}

#[global_allocator]
static GLOBAL: CountingAllocator = CountingAllocator;

fn alloc_scope<T>(f: impl FnOnce() -> T) -> (T, u64, u64) {
    ALLOC_CALLS.store(0, Ordering::Relaxed);
    ALLOC_BYTES.store(0, Ordering::Relaxed);
    COUNT_ALLOC.store(true, Ordering::Relaxed);
    let value = f();
    COUNT_ALLOC.store(false, Ordering::Relaxed);
    (
        value,
        ALLOC_CALLS.load(Ordering::Relaxed),
        ALLOC_BYTES.load(Ordering::Relaxed),
    )
}

fn field_len<T: Serialize>(value: &T) -> usize {
    postcard::to_allocvec(value)
        .map(|bytes| bytes.len())
        .unwrap_or(0)
}

/// Encoded payload plus the one-byte `ServerMsg` variant tag.
fn message_len<T: Serialize>(value: &T) -> usize {
    field_len(value) + 1
}

fn percentile(sorted: &[u64], q: f64) -> u64 {
    if sorted.is_empty() {
        return 0;
    }
    let i = ((sorted.len() - 1) as f64 * q).round() as usize;
    sorted[i.min(sorted.len() - 1)]
}

/// Uniformly sampled timings: one in four, capped only by memory.
#[derive(Default)]
struct Samples {
    values: Vec<u64>,
    seen: u64,
}

impl Samples {
    fn push(&mut self, value: u64) {
        self.seen += 1;
        if self.seen % 4 == 1 {
            self.values.push(value);
        }
    }

    fn summary(&self) -> TimingSummary {
        let mut values = self.values.clone();
        values.sort_unstable();
        TimingSummary {
            samples: self.seen,
            median_us: percentile(&values, 0.5) as f64 / 1_000.0,
            p95_us: percentile(&values, 0.95) as f64 / 1_000.0,
        }
    }
}

struct TimingSummary {
    samples: u64,
    median_us: f64,
    p95_us: f64,
}

#[derive(Default, Clone)]
struct SizeStats {
    sizes: Vec<usize>,
}

impl SizeStats {
    fn push(&mut self, bytes: usize) {
        self.sizes.push(bytes);
    }

    fn summary(&self, views_per_second: f64) -> Summary {
        let mut sizes = self.sizes.clone();
        sizes.sort_unstable();
        let avg = if sizes.is_empty() {
            0.0
        } else {
            sizes.iter().sum::<usize>() as f64 / sizes.len() as f64
        };
        let p95 = if sizes.is_empty() {
            0
        } else {
            let i = ((sizes.len() - 1) as f64 * 0.95).round() as usize;
            sizes[i.min(sizes.len() - 1)]
        };
        Summary {
            views: sizes.len() as u64,
            avg_b: avg,
            p95_b: p95,
            max_b: sizes.last().copied().unwrap_or(0),
            bytes_s: avg * views_per_second,
        }
    }
}

struct Summary {
    views: u64,
    avg_b: f64,
    p95_b: usize,
    max_b: usize,
    bytes_s: f64,
}

#[derive(Default, Clone, Copy)]
struct Sections {
    total: u64,
    header: u64,
    units: u64,
    statics: u64,
    removed: u64,
    events: u64,
}

impl Sections {
    fn add(&mut self, other: Sections) {
        self.total += other.total;
        self.header += other.header;
        self.units += other.units;
        self.statics += other.statics;
        self.removed += other.removed;
        self.events += other.events;
    }
}

#[derive(Default)]
struct ShareAgg {
    views: u64,
    total: u64,
    header: u64,
    units: u64,
    unit_static: u64,
    unit_max_hp: u64,
    unit_dynamic: u64,
    /// Same dynamic fields with `x` and `hp` as `u16` varints (P30-A step 3).
    unit_dynamic_tight: u64,
    events: u64,
    units_seen: u64,
    events_seen: u64,
    header_fields: BTreeMap<&'static str, u64>,
    event_counts: BTreeMap<&'static str, u64>,
}

fn event_name(event: &Event) -> &'static str {
    match event {
        Event::Spawn { .. } => "Spawn",
        Event::AttackStart { .. } => "AttackStart",
        Event::Hit { .. } => "Hit",
        Event::Heal { .. } => "Heal",
        Event::StatusApplied { .. } => "StatusApplied",
        Event::Death { .. } => "Death",
        Event::Evolved { .. } => "Evolved",
        Event::SynergyChanged { .. } => "SynergyChanged",
        Event::PhaseChanged { .. } => "PhaseChanged",
        Event::Fortified { .. } => "Fortified",
        Event::RoundOpened { .. } => "RoundOpened",
        Event::AugmentChosen { .. } => "AugmentChosen",
        Event::Forgone { .. } => "Forgone",
        Event::ShieldRaised { .. } => "ShieldRaised",
        Event::SecondWind { .. } => "SecondWind",
        Event::PoisonTick { .. } => "PoisonTick",
        Event::Betrayal { .. } => "Betrayal",
        Event::Cleansed { .. } => "Cleansed",
        Event::PlateBroken { .. } => "PlateBroken",
        Event::CrateOpened { .. } => "CrateOpened",
        Event::Transfer { .. } => "Transfer",
        Event::Boarded { .. } => "Boarded",
        Event::Dismounted { .. } => "Dismounted",
        Event::CorrosiveSplash { .. } => "CorrosiveSplash",
        Event::AbilityStarted { .. } => "AbilityStarted",
        Event::AbilityResolved { .. } => "AbilityResolved",
        Event::Crossfire { .. } => "Crossfire",
    }
}

/// Exact field-by-field decomposition of one canonical view.
fn collect_shares(view: &MatchView, agg: &mut ShareAgg) {
    let total = field_len(view);
    let units_bytes = field_len(&view.units);
    let events_bytes = field_len(&view.events);
    let header = total - units_bytes - events_bytes;
    let header_fields = [
        ("tick", field_len(&view.tick)),
        ("phase", field_len(&view.phase)),
        ("phase_ticks_left", field_len(&view.phase_ticks_left)),
        ("you", field_len(&view.you)),
        ("energy", field_len(&view.energy)),
        ("offers", field_len(&view.offers)),
        ("clock", field_len(&view.clock)),
        ("elapsed", field_len(&view.elapsed)),
        ("regen_bp", field_len(&view.regen_bp)),
        ("evolve_costs", field_len(&view.evolve_costs)),
        ("stronghold_hp", field_len(&view.stronghold_hp)),
        ("strongholds", field_len(&view.strongholds)),
        ("shields", field_len(&view.shields)),
        ("synergy", field_len(&view.synergy)),
        ("augments", field_len(&view.augments)),
        ("augment_round", field_len(&view.augment_round)),
        ("last_seq", field_len(&view.last_seq)),
        ("outcome", field_len(&view.outcome)),
    ];
    let header_sum: usize = header_fields.iter().map(|(_, n)| n).sum();
    assert_eq!(
        header_sum, header,
        "header field decomposition must sum to the header residual"
    );
    let mut unit_static = 0;
    let mut unit_max_hp = 0;
    let mut unit_dynamic = 0;
    let mut unit_dynamic_tight = 0;
    for unit in &view.units {
        unit_static += field_len(&unit.seat)
            + field_len(&unit.kind)
            + field_len(&unit.tier)
            + field_len(&unit.branch);
        unit_max_hp += field_len(&unit.max_hp);
        unit_dynamic += field_len(&unit.id)
            + field_len(&unit.x)
            + field_len(&unit.hp)
            + field_len(&unit.status)
            + field_len(&unit.fx);
        let x = u16::try_from(unit.x).expect("x must fit u16 for the tight-integer estimate");
        let hp = u16::try_from(unit.hp).expect("live hp must fit u16");
        unit_dynamic_tight += field_len(&unit.id)
            + field_len(&x)
            + field_len(&hp)
            + field_len(&unit.status)
            + field_len(&unit.fx);
    }
    let unit_fields = unit_static + unit_max_hp + unit_dynamic;
    let unit_prefix = units_bytes - unit_fields;
    assert!(
        unit_prefix <= 3,
        "unit vector prefix should be a small varint, got {unit_prefix}"
    );
    let mut event_sum = 0;
    for event in &view.events {
        event_sum += field_len(event);
        *agg.event_counts.entry(event_name(event)).or_default() += 1;
    }
    let event_prefix = events_bytes - event_sum;
    assert!(
        event_prefix <= 3,
        "event vector prefix should be a small varint, got {event_prefix}"
    );
    agg.views += 1;
    agg.total += total as u64;
    agg.header += header as u64;
    agg.units += units_bytes as u64;
    agg.unit_static += unit_static as u64;
    agg.unit_max_hp += unit_max_hp as u64;
    agg.unit_dynamic += unit_dynamic as u64;
    agg.unit_dynamic_tight += unit_dynamic_tight as u64;
    agg.events += events_bytes as u64;
    agg.units_seen += view.units.len() as u64;
    agg.events_seen += view.events.len() as u64;
    for (name, n) in header_fields {
        *agg.header_fields.entry(name).or_default() += n as u64;
    }
}

// ---------- Prototype payloads ----------

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
struct UnitA {
    id: UnitId,
    seat: Seat,
    kind: UnitKind,
    tier: u8,
    branch: u8,
    x: i32,
    hp: i32,
    status: u8,
    fx: u16,
}

#[derive(Serialize)]
struct ViewA {
    tick: Tick,
    extra: Extra,
    phase: Phase,
    energy: u32,
    offers: [UnitKind; OFFERS],
    stronghold_hp: [i32; 2],
    units: Vec<UnitA>,
    synergy: [[u8; tbg_core::FACTION_COUNT]; 2],
    last_seq: u32,
    events: Vec<Event>,
    outcome: Option<tbg_core::Outcome>,
}

/// Header fields the canonical view gained after P30-A (Augment pause,
/// open-ended clock, Stronghold kinds, shields): every prototype carries
/// them unchanged, so the comparison stays about the unit list.
#[derive(Clone, Debug, Serialize)]
struct Extra {
    clock: Tick,
    elapsed: u32,
    regen_bp: u32,
    evolve_costs: [u32; 2],
    strongholds: [StrongholdView; 2],
    shields: [i32; 2],
    augments: [AugmentCounts; 2],
    augment_round: Option<RoundView>,
}

fn extra_of(view: &MatchView) -> Extra {
    Extra {
        clock: view.clock,
        elapsed: view.elapsed,
        regen_bp: view.regen_bp,
        evolve_costs: view.evolve_costs,
        strongholds: view.strongholds,
        shields: view.shields,
        augments: view.augments,
        augment_round: view.augment_round.clone(),
    }
}

/// The same header straight from the state, as the server would build it;
/// the round (only during a pause) comes from the canonical view.
fn extra_state(state: &MatchState, seat: Seat) -> Extra {
    let augment_round = if state.is_live() && state.augments.selecting() {
        state.view_for(seat, &[]).augment_round
    } else {
        None
    };
    Extra {
        clock: state.clock,
        elapsed: state.tick.saturating_sub(content::RUNNING_START),
        regen_bp: state.rules().interest.regen_bp(state.player(seat).energy),
        evolve_costs: [0, 1].map(|tier| state.evolve_price(seat, tier)),
        strongholds: Seat::BOTH.map(|s| StrongholdView {
            kind: state.player(s).stronghold,
            tier: state.player(s).fort_tier,
            branch: state.player(s).fort_branch,
            max_hp: state.stronghold(s).max_hp,
        }),
        shields: [0, 1].map(|i| state.players[i].shield),
        augments: [state.players[0].augments, state.players[1].augments],
        augment_round,
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
struct UnitStatic {
    id: UnitId,
    seat: Seat,
    kind: UnitKind,
    tier: u8,
    branch: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
struct UnitDynamic {
    id: UnitId,
    x: i32,
    hp: i32,
    status: u8,
    fx: u16,
}

#[derive(Serialize)]
struct ViewC {
    tick: Tick,
    extra: Extra,
    phase: Phase,
    energy: u32,
    offers: [UnitKind; OFFERS],
    stronghold_hp: [i32; 2],
    full: bool,
    static_units: Vec<UnitStatic>,
    units: Vec<UnitDynamic>,
    synergy: Option<[[u8; tbg_core::FACTION_COUNT]; 2]>,
    last_seq: u32,
    events: Vec<Event>,
    outcome: Option<tbg_core::Outcome>,
}

#[derive(Serialize)]
struct ViewB {
    tick: Tick,
    extra: Extra,
    phase: Phase,
    energy: u32,
    offers: [UnitKind; OFFERS],
    stronghold_hp: [i32; 2],
    full: bool,
    static_units: Vec<UnitStatic>,
    units: Vec<UnitDynamic>,
    removed: Vec<UnitId>,
    synergy: Option<[[u8; tbg_core::FACTION_COUNT]; 2]>,
    last_seq: u32,
    events: Vec<Event>,
    outcome: Option<tbg_core::Outcome>,
}

#[derive(Serialize)]
struct ViewD {
    tick: Tick,
    events: Vec<Event>,
}

fn unit_static(unit: &UnitView) -> UnitStatic {
    UnitStatic {
        id: unit.id,
        seat: unit.seat,
        kind: unit.kind,
        tier: unit.tier,
        branch: unit.branch,
    }
}

fn unit_dynamic(unit: &UnitView) -> UnitDynamic {
    UnitDynamic {
        id: unit.id,
        x: unit.x,
        hp: unit.hp,
        status: unit.status,
        fx: unit.fx,
    }
}

/// `UnitView.fx` as the server packs it: the summon bit, recognized by
/// unit (BR-U08); the other ability bits are not packed yet.
fn fx_of(unit: &tbg_core::Unit) -> u16 {
    if unit.summoner.is_some() || unit.kind.is_summon() {
        FX_SUMMON
    } else {
        0
    }
}

fn derived_max_hp(
    unit: &UnitStatic,
    fx: u16,
    synergy: [[u8; tbg_core::FACTION_COUNT]; 2],
    arcanas: &[ArcanaPreset; 2],
    augments: &[AugmentCounts; 2],
    rules: &Ruleset,
) -> i32 {
    let seat = unit.seat.index();
    let level = synergy[seat][content::def(unit.kind).faction.index()];
    let bonus = augment::unit_bonus(&augments[seat], synergy[seat], unit.kind, unit.tier);
    rules.unit_max_hp(
        (unit.kind, unit.tier, unit.branch),
        unit.kind.is_summon() || fx & FX_SUMMON != 0,
        level,
        &arcanas[seat],
        &bonus,
    )
}

/// Pure reconstruction of the removed header fields (P30-A Prototype A).
fn phase_ticks_left_at(tick: Tick, phase: Phase) -> u32 {
    match phase {
        Phase::Countdown => content::RUNNING_START.saturating_sub(tick),
        // Open-ended matches: no gameplay clock once running (D-41).
        Phase::Running | Phase::Ended => 0,
    }
}

// ---------- Prototype builds ----------

fn status_code(status: Status) -> u8 {
    match status {
        Status::None => 0,
        Status::Slowed { .. } => 1,
        Status::Rooted { .. } => 2,
    }
}

fn live_units(state: &MatchState) -> impl Iterator<Item = &tbg_core::Unit> {
    state.units.iter().filter(|unit| unit.hp > 0)
}

fn state_statics(state: &MatchState) -> Vec<UnitStatic> {
    live_units(state)
        .map(|unit| UnitStatic {
            id: unit.id,
            seat: unit.seat,
            kind: unit.kind,
            tier: unit.tier,
            branch: unit.branch,
        })
        .collect()
}

fn state_dynamics(state: &MatchState) -> Vec<UnitDynamic> {
    live_units(state)
        .map(|unit| UnitDynamic {
            id: unit.id,
            x: unit.x,
            hp: unit.hp,
            status: status_code(unit.status),
            fx: fx_of(unit),
        })
        .collect()
}

/// Build Prototype A directly from the state, as the server would.
fn build_a_state(state: &MatchState, seat: Seat, events: &[Event]) -> ViewA {
    let me = state.player(seat);
    ViewA {
        tick: state.tick,
        extra: extra_state(state, seat),
        phase: state.phase,
        energy: me.energy,
        offers: me.offers,
        stronghold_hp: [0, 1].map(|i| state.players[i].stronghold_hp),
        units: live_units(state)
            .map(|unit| UnitA {
                id: unit.id,
                seat: unit.seat,
                kind: unit.kind,
                tier: unit.tier,
                branch: unit.branch,
                x: unit.x,
                hp: unit.hp,
                status: status_code(unit.status),
                fx: fx_of(unit),
            })
            .collect(),
        synergy: [0, 1].map(|i| state.players[i].synergy),
        last_seq: me.last_seq,
        events: events.to_vec(),
        outcome: state.outcome,
    }
}

#[derive(Default, Clone)]
struct StaticTracker {
    units: Vec<UnitStatic>,
    synergy: [[u8; tbg_core::FACTION_COUNT]; 2],
}

#[derive(Default, Clone)]
struct StaticCache {
    units: BTreeMap<UnitId, UnitStatic>,
    synergy: [[u8; tbg_core::FACTION_COUNT]; 2],
}

/// Sorted merge: both lists are sorted by `UnitId` (ids only grow).
fn update_statics(
    current: &[UnitStatic],
    tracker: &mut StaticTracker,
    synergy: [[u8; tbg_core::FACTION_COUNT]; 2],
    full: bool,
) -> (Vec<UnitStatic>, Option<[[u8; tbg_core::FACTION_COUNT]; 2]>) {
    let mut statics = Vec::new();
    if full {
        statics.extend_from_slice(current);
    } else {
        let mut old = 0;
        for unit in current {
            while tracker
                .units
                .get(old)
                .is_some_and(|previous| previous.id < unit.id)
            {
                old += 1;
            }
            let changed = match tracker.units.get(old) {
                Some(previous) if previous.id == unit.id => {
                    old += 1;
                    previous != unit
                }
                _ => true,
            };
            if changed {
                statics.push(unit.clone());
            }
        }
    }
    let synergy_update = (full || tracker.synergy != synergy).then_some(synergy);
    tracker.units.clear();
    tracker.units.extend_from_slice(current);
    if let Some(update) = synergy_update {
        tracker.synergy = update;
    }
    (statics, synergy_update)
}

fn build_c(view: &MatchView, tracker: &mut StaticTracker, full: bool) -> ViewC {
    let current: Vec<UnitStatic> = view.units.iter().map(unit_static).collect();
    let (static_units, synergy) = update_statics(&current, tracker, view.synergy, full);
    ViewC {
        tick: view.tick,
        extra: extra_of(view),
        phase: view.phase,
        energy: view.energy,
        offers: view.offers,
        stronghold_hp: view.stronghold_hp,
        full,
        static_units,
        units: view.units.iter().map(unit_dynamic).collect(),
        synergy,
        last_seq: view.last_seq,
        events: view.events.clone(),
        outcome: view.outcome,
    }
}

/// Build Prototype C directly from the state, as the server would.
fn build_c_state(
    state: &MatchState,
    seat: Seat,
    events: &[Event],
    tracker: &mut StaticTracker,
    full: bool,
) -> ViewC {
    let me = state.player(seat);
    let synergy = [0, 1].map(|i| state.players[i].synergy);
    let (static_units, synergy_update) =
        update_statics(&state_statics(state), tracker, synergy, full);
    ViewC {
        tick: state.tick,
        extra: extra_state(state, seat),
        phase: state.phase,
        energy: me.energy,
        offers: me.offers,
        stronghold_hp: [0, 1].map(|i| state.players[i].stronghold_hp),
        full,
        static_units,
        units: state_dynamics(state),
        synergy: synergy_update,
        last_seq: me.last_seq,
        events: events.to_vec(),
        outcome: state.outcome,
    }
}

fn rebuild_c(
    view: &ViewC,
    cache: &mut StaticCache,
    you: Seat,
    arcanas: &[ArcanaPreset; 2],
    rules: &Ruleset,
) -> Result<MatchView, &'static str> {
    if view.full {
        cache.units.clear();
    }
    for unit in &view.static_units {
        cache.units.insert(unit.id, unit.clone());
    }
    if let Some(synergy) = view.synergy {
        cache.synergy = synergy;
    }
    let units = rebuild_units(
        &view.units,
        &cache.units,
        cache.synergy,
        arcanas,
        &view.extra.augments,
        rules,
    )?;
    let live: BTreeSet<UnitId> = view.units.iter().map(|u| u.id).collect();
    cache.units.retain(|id, _| live.contains(id));
    Ok(assemble_c(view, units, cache.synergy, you))
}

fn rebuild_units(
    dynamics: &[UnitDynamic],
    statics: &BTreeMap<UnitId, UnitStatic>,
    synergy: [[u8; tbg_core::FACTION_COUNT]; 2],
    arcanas: &[ArcanaPreset; 2],
    augments: &[AugmentCounts; 2],
    rules: &Ruleset,
) -> Result<Vec<UnitView>, &'static str> {
    let mut units = Vec::with_capacity(dynamics.len());
    for dynamic in dynamics {
        let stat = statics
            .get(&dynamic.id)
            .ok_or("dynamic unit without a static entry")?;
        units.push(UnitView {
            id: stat.id,
            seat: stat.seat,
            kind: stat.kind,
            tier: stat.tier,
            branch: stat.branch,
            x: dynamic.x,
            hp: dynamic.hp,
            max_hp: derived_max_hp(stat, dynamic.fx, synergy, arcanas, augments, rules),
            status: dynamic.status,
            fx: dynamic.fx,
            carrier: None,
            shield: 0,
            reduction: 0,
            special: None,
            effects: None,
        });
    }
    Ok(units)
}

fn assemble_c(
    view: &ViewC,
    units: Vec<UnitView>,
    synergy: [[u8; tbg_core::FACTION_COUNT]; 2],
    you: Seat,
) -> MatchView {
    MatchView {
        tick: view.tick,
        phase: view.phase,
        phase_ticks_left: phase_ticks_left_at(view.tick, view.phase),
        you,
        energy: view.energy,
        offers: view.offers,
        clock: view.extra.clock,
        elapsed: view.extra.elapsed,
        regen_bp: view.extra.regen_bp,
        evolve_costs: view.extra.evolve_costs,
        strongholds: view.extra.strongholds,
        shields: view.extra.shields,
        augments: view.extra.augments,
        augment_round: view.extra.augment_round.clone(),
        stronghold_hp: view.stronghold_hp,
        units,
        synergy,
        last_seq: view.last_seq,
        events: view.events.clone(),
        outcome: view.outcome,
        spectators: 0,
    }
}

fn assemble_b(
    view: &ViewB,
    units: Vec<UnitView>,
    synergy: [[u8; tbg_core::FACTION_COUNT]; 2],
    you: Seat,
) -> MatchView {
    MatchView {
        tick: view.tick,
        phase: view.phase,
        phase_ticks_left: phase_ticks_left_at(view.tick, view.phase),
        you,
        energy: view.energy,
        offers: view.offers,
        clock: view.extra.clock,
        elapsed: view.extra.elapsed,
        regen_bp: view.extra.regen_bp,
        evolve_costs: view.extra.evolve_costs,
        strongholds: view.extra.strongholds,
        shields: view.extra.shields,
        augments: view.extra.augments,
        augment_round: view.extra.augment_round.clone(),
        stronghold_hp: view.stronghold_hp,
        units,
        synergy,
        last_seq: view.last_seq,
        events: view.events.clone(),
        outcome: view.outcome,
        spectators: 0,
    }
}

#[derive(Default, Clone)]
struct DeltaCache {
    statics: StaticCache,
    dynamics: BTreeMap<UnitId, UnitDynamic>,
}

#[derive(Default, Clone)]
struct DeltaTracker {
    statics: StaticTracker,
    dynamics: Vec<UnitDynamic>,
}

fn update_dynamics(
    current: &[UnitDynamic],
    forced: &BTreeSet<UnitId>,
    tracker: &mut Vec<UnitDynamic>,
    full: bool,
) -> (Vec<UnitDynamic>, Vec<UnitId>) {
    let mut units = Vec::new();
    let mut removed = Vec::new();
    let mut old = 0;
    for unit in current {
        while tracker
            .get(old)
            .is_some_and(|previous| previous.id < unit.id)
        {
            removed.push(tracker[old].id);
            old += 1;
        }
        let changed = match tracker.get(old) {
            Some(previous) if previous.id == unit.id => {
                old += 1;
                full || forced.contains(&unit.id) || previous != unit
            }
            _ => true,
        };
        if changed {
            units.push(*unit);
        }
    }
    while let Some(previous) = tracker.get(old) {
        removed.push(previous.id);
        old += 1;
    }
    if full {
        removed.clear();
    }
    tracker.clear();
    tracker.extend_from_slice(current);
    (units, removed)
}

fn build_b(view: &MatchView, tracker: &mut DeltaTracker, full: bool) -> ViewB {
    let current_statics: Vec<UnitStatic> = view.units.iter().map(unit_static).collect();
    let (static_units, synergy) =
        update_statics(&current_statics, &mut tracker.statics, view.synergy, full);
    let forced: BTreeSet<UnitId> = static_units.iter().map(|unit| unit.id).collect();
    let current_dynamics: Vec<UnitDynamic> = view.units.iter().map(unit_dynamic).collect();
    let (units, removed) = update_dynamics(&current_dynamics, &forced, &mut tracker.dynamics, full);
    ViewB {
        tick: view.tick,
        extra: extra_of(view),
        phase: view.phase,
        energy: view.energy,
        offers: view.offers,
        stronghold_hp: view.stronghold_hp,
        full,
        static_units,
        units,
        removed,
        synergy,
        last_seq: view.last_seq,
        events: view.events.clone(),
        outcome: view.outcome,
    }
}

/// Build Prototype B directly from the state, as the server would.
fn build_b_state(
    state: &MatchState,
    seat: Seat,
    events: &[Event],
    tracker: &mut DeltaTracker,
    full: bool,
) -> ViewB {
    let me = state.player(seat);
    let synergy = [0, 1].map(|i| state.players[i].synergy);
    let (static_units, synergy_update) =
        update_statics(&state_statics(state), &mut tracker.statics, synergy, full);
    let forced: BTreeSet<UnitId> = static_units.iter().map(|unit| unit.id).collect();
    let (units, removed) =
        update_dynamics(&state_dynamics(state), &forced, &mut tracker.dynamics, full);
    ViewB {
        tick: state.tick,
        extra: extra_state(state, seat),
        phase: state.phase,
        energy: me.energy,
        offers: me.offers,
        stronghold_hp: [0, 1].map(|i| state.players[i].stronghold_hp),
        full,
        static_units,
        units,
        removed,
        synergy: synergy_update,
        last_seq: me.last_seq,
        events: events.to_vec(),
        outcome: state.outcome,
    }
}

fn rebuild_b(
    view: &ViewB,
    cache: &mut DeltaCache,
    you: Seat,
    arcanas: &[ArcanaPreset; 2],
    rules: &Ruleset,
) -> Result<MatchView, &'static str> {
    if view.full {
        cache.statics.units.clear();
        cache.dynamics.clear();
    }
    for stat in &view.static_units {
        cache.statics.units.insert(stat.id, stat.clone());
    }
    for id in &view.removed {
        cache.statics.units.remove(id);
        cache.dynamics.remove(id);
    }
    for dynamic in &view.units {
        cache.dynamics.insert(dynamic.id, *dynamic);
    }
    if let Some(synergy) = view.synergy {
        cache.statics.synergy = synergy;
    }
    let dynamics: Vec<UnitDynamic> = cache.dynamics.values().copied().collect();
    let units = rebuild_units(
        &dynamics,
        &cache.statics.units,
        cache.statics.synergy,
        arcanas,
        &view.extra.augments,
        rules,
    )?;
    let live: BTreeSet<UnitId> = cache.dynamics.keys().copied().collect();
    cache.statics.units.retain(|id, _| live.contains(id));
    Ok(assemble_b(view, units, cache.statics.synergy, you))
}

/// Lets the state-built view reuse the prototype header lines.
struct ViewHeader {
    extra: Extra,
}

fn assert_same(got: &MatchView, want: &MatchView, label: &str, context: &str) {
    assert_eq!(got.tick, want.tick, "{label} tick {context}");
    assert_eq!(got.phase, want.phase, "{label} phase {context}");
    assert_eq!(
        got.phase_ticks_left, want.phase_ticks_left,
        "{label} phase_ticks_left {context}"
    );
    assert_eq!(got.you, want.you, "{label} you {context}");
    assert_eq!(got.energy, want.energy, "{label} energy {context}");
    assert_eq!(got.offers, want.offers, "{label} offers {context}");
    assert_eq!(got.clock, want.clock, "{label} clock {context}");
    assert_eq!(got.elapsed, want.elapsed, "{label} elapsed {context}");
    assert_eq!(got.regen_bp, want.regen_bp, "{label} regen_bp {context}");
    assert_eq!(
        got.evolve_costs, want.evolve_costs,
        "{label} evolve_costs {context}"
    );
    assert_eq!(
        got.strongholds, want.strongholds,
        "{label} strongholds {context}"
    );
    assert_eq!(got.shields, want.shields, "{label} shields {context}");
    assert_eq!(got.augments, want.augments, "{label} augments {context}");
    assert_eq!(
        got.augment_round, want.augment_round,
        "{label} augment_round {context}"
    );
    assert_eq!(
        got.stronghold_hp, want.stronghold_hp,
        "{label} stronghold_hp {context}"
    );
    assert_eq!(
        got.units.len(),
        want.units.len(),
        "{label} unit count {context}"
    );
    for (a, b) in got.units.iter().zip(&want.units) {
        assert_eq!(a, b, "{label} unit {:?} {context}", b.id);
    }
    assert_eq!(got.synergy, want.synergy, "{label} synergy {context}");
    assert_eq!(got.last_seq, want.last_seq, "{label} last_seq {context}");
    assert_eq!(got.events, want.events, "{label} events {context}");
    assert_eq!(got.outcome, want.outcome, "{label} outcome {context}");
}

// ---------- Recorder ----------

#[derive(Default)]
struct SeatHarness {
    c_server: StaticTracker,
    c_client: StaticCache,
    b_server: DeltaTracker,
    b_client: DeltaCache,
    c_views: u32,
    b_views: u32,
}

struct Recorder {
    matches: u64,
    views: u64,
    units_seen: u64,
    max_units: usize,
    empty_views: u64,
    one_unit_views: u64,
    current: SizeStats,
    a: SizeStats,
    b: SizeStats,
    c: SizeStats,
    d: SizeStats,
    build_current: Samples,
    encode_current: Samples,
    build_a: Samples,
    encode_a: Samples,
    build_b: Samples,
    encode_b: Samples,
    build_c: Samples,
    encode_c: Samples,
    rebuild_a: Samples,
    rebuild_b: Samples,
    rebuild_c: Samples,
    shares: ShareAgg,
    sec_a: Sections,
    sec_b: Sections,
    sec_c: Sections,
    sec_d: Sections,
    b_full: u64,
    c_full: u64,
    b_changed_units: u64,
    freq_current: [SizeStats; 2],
    freq_c: [SizeStats; 2],
    freq_caches: [[StaticTracker; 2]; 2],
    freq_views: [[u32; 2]; 2],
    buckets_current: Vec<SizeStats>,
    buckets_c: Vec<SizeStats>,
    h10_two: Samples,
    h10_shared: Samples,
    h10_alloc_two: (u64, u64),
    h10_alloc_shared: (u64, u64),
    h10_alloc_samples: u64,
    c_cache_max: usize,
    b_cache_max: usize,
    sync_prev: [Option<Arc<MatchView>>; 2],
    sync_current: Samples,
    sync_poc: Samples,
    seats: [SeatHarness; 2],
}

impl Default for Recorder {
    fn default() -> Self {
        Self {
            matches: 0,
            views: 0,
            units_seen: 0,
            max_units: 0,
            empty_views: 0,
            one_unit_views: 0,
            current: SizeStats::default(),
            a: SizeStats::default(),
            b: SizeStats::default(),
            c: SizeStats::default(),
            d: SizeStats::default(),
            build_current: Samples::default(),
            encode_current: Samples::default(),
            build_a: Samples::default(),
            encode_a: Samples::default(),
            build_b: Samples::default(),
            encode_b: Samples::default(),
            build_c: Samples::default(),
            encode_c: Samples::default(),
            rebuild_a: Samples::default(),
            rebuild_b: Samples::default(),
            rebuild_c: Samples::default(),
            shares: ShareAgg::default(),
            sec_a: Sections::default(),
            sec_b: Sections::default(),
            sec_c: Sections::default(),
            sec_d: Sections::default(),
            b_full: 0,
            c_full: 0,
            b_changed_units: 0,
            freq_current: std::array::from_fn(|_| SizeStats::default()),
            freq_c: std::array::from_fn(|_| SizeStats::default()),
            freq_caches: std::array::from_fn(|_| std::array::from_fn(|_| StaticTracker::default())),
            freq_views: [[0; 2]; 2],
            buckets_current: (0..6).map(|_| SizeStats::default()).collect(),
            buckets_c: (0..6).map(|_| SizeStats::default()).collect(),
            h10_two: Samples::default(),
            h10_shared: Samples::default(),
            h10_alloc_two: (0, 0),
            h10_alloc_shared: (0, 0),
            h10_alloc_samples: 0,
            c_cache_max: 0,
            b_cache_max: 0,
            sync_prev: std::array::from_fn(|_| None),
            sync_current: Samples::default(),
            sync_poc: Samples::default(),
            seats: std::array::from_fn(|_| SeatHarness::default()),
        }
    }
}

fn bucket_of(units: usize) -> usize {
    match units {
        0..=10 => 0,
        11..=20 => 1,
        21..=40 => 2,
        41..=60 => 3,
        61..=100 => 4,
        _ => 5,
    }
}

impl Recorder {
    #[allow(clippy::too_many_lines)] // one measurement path, kept linear for auditability
    fn record(
        &mut self,
        state: &MatchState,
        seat: Seat,
        view: &MatchView,
        events: &[Event],
        arcanas: &[ArcanaPreset; 2],
        rules: &Ruleset,
    ) {
        self.views += 1;
        self.units_seen += view.units.len() as u64;
        self.max_units = self.max_units.max(view.units.len());
        if view.units.is_empty() {
            self.empty_views += 1;
        } else if view.units.len() == 1 {
            self.one_unit_views += 1;
        }
        self.buckets_current[bucket_of(view.units.len())].push(message_len(view));
        collect_shares(view, &mut self.shares);

        // Current: full snapshot, postcard, one variant tag.
        let start = Instant::now();
        let current_bytes = message_len(view);
        self.encode_current.push(start.elapsed().as_nanos() as u64);
        self.current.push(current_bytes);

        // A: derive, don't send.
        let start = Instant::now();
        let a = build_a_state(state, seat, events);
        self.build_a.push(start.elapsed().as_nanos() as u64);
        let start = Instant::now();
        let a_bytes = message_len(&a);
        self.encode_a.push(start.elapsed().as_nanos() as u64);
        self.a.push(a_bytes);
        let a_field = field_len(&a);
        self.sec_a.add(Sections {
            total: a_field as u64,
            header: (a_field - field_len(&a.units) - field_len(&a.events)) as u64,
            units: field_len(&a.units) as u64,
            events: field_len(&a.events) as u64,
            ..Sections::default()
        });
        let start = Instant::now();
        match rebuild_a(&a, seat, arcanas, rules) {
            Ok(rebuilt) => assert_same(&rebuilt, view, "A", "corpus"),
            Err(reason) => panic!("A reconstruction failed: {reason}"),
        }
        self.rebuild_a.push(start.elapsed().as_nanos() as u64);

        // C: A + static/dynamic split.
        let full = self.seats[seat.index()].c_views == 0;
        self.seats[seat.index()].c_views += 1;
        if full {
            self.c_full += 1;
        }
        let start = Instant::now();
        let c = build_c_state(
            state,
            seat,
            events,
            &mut self.seats[seat.index()].c_server,
            full,
        );
        self.build_c.push(start.elapsed().as_nanos() as u64);
        let start = Instant::now();
        let c_bytes = message_len(&c);
        self.encode_c.push(start.elapsed().as_nanos() as u64);
        self.c.push(c_bytes);
        self.buckets_c[bucket_of(view.units.len())].push(c_bytes);
        let c_field = field_len(&c);
        self.sec_c.add(Sections {
            total: c_field as u64,
            header: (c_field
                - field_len(&c.static_units)
                - field_len(&c.units)
                - field_len(&c.events)) as u64,
            units: field_len(&c.units) as u64,
            statics: field_len(&c.static_units) as u64,
            events: field_len(&c.events) as u64,
            ..Sections::default()
        });
        let start = Instant::now();
        match rebuild_c(
            &c,
            &mut self.seats[seat.index()].c_client,
            seat,
            arcanas,
            rules,
        ) {
            Ok(rebuilt) => assert_same(&rebuilt, view, "C", "corpus"),
            Err(reason) => panic!("C reconstruction failed: {reason}"),
        }
        self.rebuild_c.push(start.elapsed().as_nanos() as u64);
        let cache = &self.seats[seat.index()].c_client;
        let bytes =
            cache.units.len() * (std::mem::size_of::<UnitId>() + std::mem::size_of::<UnitStatic>());
        self.c_cache_max = self.c_cache_max.max(bytes);

        // B: static cache + changed dynamics, keyframe every 10 s of views.
        let full = self.seats[seat.index()]
            .b_views
            .is_multiple_of(KEYFRAME_VIEWS);
        self.seats[seat.index()].b_views += 1;
        if full {
            self.b_full += 1;
        }
        let start = Instant::now();
        let b = build_b_state(
            state,
            seat,
            events,
            &mut self.seats[seat.index()].b_server,
            full,
        );
        self.build_b.push(start.elapsed().as_nanos() as u64);
        self.b_changed_units += b.units.len() as u64;
        let start = Instant::now();
        let b_bytes = message_len(&b);
        self.encode_b.push(start.elapsed().as_nanos() as u64);
        self.b.push(b_bytes);
        let b_field = field_len(&b);
        self.sec_b.add(Sections {
            total: b_field as u64,
            header: (b_field
                - field_len(&b.static_units)
                - field_len(&b.units)
                - field_len(&b.removed)
                - field_len(&b.events)) as u64,
            units: field_len(&b.units) as u64,
            statics: field_len(&b.static_units) as u64,
            removed: field_len(&b.removed) as u64,
            events: field_len(&b.events) as u64,
        });
        let start = Instant::now();
        match rebuild_b(
            &b,
            &mut self.seats[seat.index()].b_client,
            seat,
            arcanas,
            rules,
        ) {
            Ok(rebuilt) => assert_same(&rebuilt, view, "B", "corpus"),
            Err(reason) => panic!("B reconstruction failed: {reason}"),
        }
        self.rebuild_b.push(start.elapsed().as_nanos() as u64);
        let cache = &self.seats[seat.index()].b_client;
        let bytes = (cache.statics.units.len() + cache.dynamics.len())
            * (std::mem::size_of::<UnitId>()
                + std::mem::size_of::<UnitStatic>()
                + std::mem::size_of::<UnitDynamic>());
        self.b_cache_max = self.b_cache_max.max(bytes);

        // D: events only, negative control.
        let d = ViewD {
            tick: view.tick,
            events: view.events.clone(),
        };
        let d_bytes = message_len(&d);
        self.d.push(d_bytes);
        let d_field = field_len(&d);
        self.sec_d.add(Sections {
            total: d_field as u64,
            header: (d_field - field_len(&d.events)) as u64,
            events: field_len(&d.events) as u64,
            ..Sections::default()
        });

        // Client sync hot path proxy (P30-A H7): full clone + linear lookup
        // versus Arc clone + sorted merge + HashSet retain.
        if self.views.is_multiple_of(8) {
            if let Some(previous) = &self.sync_prev[seat.index()] {
                let start = Instant::now();
                let cloned = (**previous).clone();
                let mut found = 0;
                for unit in &cloned.units {
                    if cloned.units.iter().any(|same| unit.id == same.id) {
                        found += 1;
                    }
                }
                for unit in &previous.units {
                    let _ = cloned.units.iter().find(|current| current.id == unit.id);
                }
                let live: Vec<UnitId> = cloned.units.iter().map(|unit| unit.id).collect();
                for id in &live {
                    let _ = cloned.units.iter().any(|unit| unit.id == *id);
                }
                std::hint::black_box(found);
                self.sync_current.push(start.elapsed().as_nanos() as u64);

                let start = Instant::now();
                let last = Arc::clone(previous);
                let mut old = 0;
                for unit in &last.units {
                    while old < last.units.len() && last.units[old].id < unit.id {
                        old += 1;
                    }
                    if let Some(previous) = last.units.get(old) {
                        let _ = previous.x;
                    }
                }
                let live: HashSet<UnitId> = last.units.iter().map(|unit| unit.id).collect();
                for id in &live {
                    let _ = live.contains(id);
                }
                std::hint::black_box((&last, &live));
                self.sync_poc.push(start.elapsed().as_nanos() as u64);
            }
            self.sync_prev[seat.index()] = Some(Arc::new(view.clone()));
        }
    }

    /// Lower-frequency sizes: current full snapshot and C, seat by seat.
    fn record_freq(&mut self, cadence: usize, seat: Seat, view: &MatchView) {
        let index = cadence - 1;
        self.freq_current[index].push(message_len(view));
        let cache = &mut self.freq_caches[index][seat.index()];
        let full = self.freq_views[index][seat.index()] == 0;
        self.freq_views[index][seat.index()] += 1;
        let c = build_c(view, cache, full);
        self.freq_c[index].push(message_len(&c));
    }
}

fn rebuild_a(
    view: &ViewA,
    you: Seat,
    arcanas: &[ArcanaPreset; 2],
    rules: &Ruleset,
) -> Result<MatchView, &'static str> {
    let mut units = Vec::with_capacity(view.units.len());
    for unit in &view.units {
        let stat = UnitStatic {
            id: unit.id,
            seat: unit.seat,
            kind: unit.kind,
            tier: unit.tier,
            branch: unit.branch,
        };
        units.push(UnitView {
            id: unit.id,
            seat: unit.seat,
            kind: unit.kind,
            tier: unit.tier,
            branch: unit.branch,
            x: unit.x,
            hp: unit.hp,
            max_hp: derived_max_hp(
                &stat,
                unit.fx,
                view.synergy,
                arcanas,
                &view.extra.augments,
                rules,
            ),
            status: unit.status,
            fx: unit.fx,
            carrier: None,
            shield: 0,
            reduction: 0,
            special: None,
            effects: None,
        });
    }
    Ok(MatchView {
        tick: view.tick,
        phase: view.phase,
        phase_ticks_left: phase_ticks_left_at(view.tick, view.phase),
        you,
        energy: view.energy,
        offers: view.offers,
        clock: view.extra.clock,
        elapsed: view.extra.elapsed,
        regen_bp: view.extra.regen_bp,
        evolve_costs: view.extra.evolve_costs,
        strongholds: view.extra.strongholds,
        shields: view.extra.shields,
        augments: view.extra.augments,
        augment_round: view.extra.augment_round.clone(),
        stronghold_hp: view.stronghold_hp,
        units,
        synergy: view.synergy,
        last_seq: view.last_seq,
        events: view.events.clone(),
        outcome: view.outcome,
        spectators: 0,
    })
}

// ---------- Scenarios ----------

fn top_up(state: &mut MatchState, seat: Seat, n: usize, next: &mut usize) {
    while state.army_size(seat) < n {
        let kind = tbg_core::UNIT_KINDS[*next % tbg_core::UNIT_KINDS.len()];
        *next += 1;
        state.players[seat.index()].offers[0] = kind;
        state.players[seat.index()].energy = 6_000;
        if state
            .apply_intention(seat, 0, PlayerIntention::Purchase { offer: 0 })
            .is_err()
        {
            return;
        }
    }
}

fn shared_views(state: &MatchState, events: &[Event]) -> (MatchView, MatchView) {
    let units: Vec<UnitView> = state
        .units
        .iter()
        .filter(|unit| unit.hp > 0)
        .map(|unit| UnitView {
            id: unit.id,
            seat: unit.seat,
            kind: unit.kind,
            tier: unit.tier,
            branch: unit.branch,
            x: unit.x,
            hp: unit.hp,
            max_hp: unit.max_hp,
            status: match unit.status {
                Status::None => 0,
                Status::Slowed { .. } => 1,
                Status::Rooted { .. } => 2,
            },
            fx: fx_of(unit),
            carrier: None,
            shield: 0,
            reduction: 0,
            special: None,
            effects: None,
        })
        .collect();
    let make = |you: Seat| {
        let view = ViewHeader {
            extra: extra_state(state, you),
        };
        MatchView {
            tick: state.tick,
            phase: state.phase,
            phase_ticks_left: phase_ticks_left_at(state.tick, state.phase),
            you,
            energy: state.player(you).energy,
            offers: state.player(you).offers,
            clock: view.extra.clock,
            elapsed: view.extra.elapsed,
            regen_bp: view.extra.regen_bp,
            evolve_costs: view.extra.evolve_costs,
            strongholds: view.extra.strongholds,
            shields: view.extra.shields,
            augments: view.extra.augments,
            augment_round: view.extra.augment_round.clone(),
            stronghold_hp: [0, 1].map(|i| state.players[i].stronghold_hp),
            units: units.clone(),
            synergy: [0, 1].map(|i| state.players[i].synergy),
            last_seq: state.player(you).last_seq,
            events: events.to_vec(),
            outcome: state.outcome,
            spectators: 0,
        }
    };
    (make(Seat::A), make(Seat::B))
}

fn run_synthetic(n: usize) -> Recorder {
    let mut rec = Recorder::default();
    let arcanas = [ArcanaPreset::DEFAULT; 2];
    let rules = Ruleset::canon();
    let mut state = MatchState::with_arcanas(n as u64, arcanas);
    while state.phase != Phase::Running {
        state.step();
    }
    let mut events = Vec::new();
    let mut events8 = Vec::new();
    let mut events5 = Vec::new();
    let mut acc8 = 20u32;
    let mut next = [0usize; 2];
    for tick in 0..SYNTHETIC_TICKS {
        for seat in Seat::BOTH {
            top_up(&mut state, seat, n, &mut next[seat.index()]);
            state.players[seat.index()].stronghold_hp = content::STRONGHOLD_HP;
        }
        if tick.is_multiple_of(VIEW_EVERY) {
            for seat in Seat::BOTH {
                let start = Instant::now();
                let view = state.view_for(seat, &events);
                rec.build_current.push(start.elapsed().as_nanos() as u64);
                rec.record(&state, seat, &view, &events, &arcanas, rules);
            }
            if rec.h10_alloc_samples < 4 && (n == 20 || n == 100) {
                let (_, calls, bytes) = alloc_scope(|| {
                    let a = state.view_for(Seat::A, &events);
                    let b = state.view_for(Seat::B, &events);
                    (a, b)
                });
                let (_, scalls, sbytes) = alloc_scope(|| shared_views(&state, &events));
                rec.h10_alloc_two = (rec.h10_alloc_two.0 + calls, rec.h10_alloc_two.1 + bytes);
                rec.h10_alloc_shared = (
                    rec.h10_alloc_shared.0 + scalls,
                    rec.h10_alloc_shared.1 + sbytes,
                );
                rec.h10_alloc_samples += 1;
            }
            if n == 20 || n == 100 {
                let start = Instant::now();
                let a = state.view_for(Seat::A, &events);
                let b = state.view_for(Seat::B, &events);
                drop((a, b));
                rec.h10_two.push(start.elapsed().as_nanos() as u64);
                let start = Instant::now();
                let (a, b) = shared_views(&state, &events);
                drop((a, b));
                rec.h10_shared.push(start.elapsed().as_nanos() as u64);
            }
            events.clear();
        }
        if acc8 >= 20 {
            for seat in Seat::BOTH {
                let view = state.view_for(seat, &events8);
                rec.record_freq(1, seat, &view);
            }
            events8.clear();
            acc8 -= 20;
        }
        acc8 += 8;
        if tick.is_multiple_of(4) {
            for seat in Seat::BOTH {
                let view = state.view_for(seat, &events5);
                rec.record_freq(2, seat, &view);
            }
            events5.clear();
        }
        let step = state.step();
        events.extend(step.iter().copied());
        events8.extend(step.iter().copied());
        events5.extend(step.iter().copied());
    }
    rec
}

fn dump_view(dir: &Path, index: usize, view: &MatchView) {
    let bytes = encode(&ServerMsg::View(Box::new(view.clone())));
    let path = dir.join(format!("{index:04}.bin"));
    if let Err(error) = std::fs::write(&path, bytes) {
        panic!("cannot write {}: {error}", path.display());
    }
}

fn run_natural() -> Recorder {
    let mut rec = Recorder::default();
    let arcanas = [ArcanaPreset::DEFAULT; 2];
    let rules = Ruleset::canon();
    let dump_dir = VIEW_DUMP.get().and_then(|dir| dir.as_deref());
    let mut dumped = 0usize;
    for seed in 1..=NATURAL_SEEDS {
        let mut state = MatchState::with_arcanas(seed, arcanas);
        let mut bots = [Bot::new(seed), Bot::new(seed ^ 0x9e37_79b9)];
        let mut events = Vec::new();
        let mut events8 = Vec::new();
        let mut events5 = Vec::new();
        let mut acc8 = 20u32;
        while state.phase != Phase::Ended && state.tick <= SAMPLE_END + 5 {
            if state.tick.is_multiple_of(VIEW_EVERY) {
                for seat in Seat::BOTH {
                    let start = Instant::now();
                    let view = state.view_for(seat, &events);
                    rec.build_current.push(start.elapsed().as_nanos() as u64);
                    rec.record(&state, seat, &view, &events, &arcanas, rules);
                    if let Some(dir) = dump_dir {
                        if seat == Seat::A && dumped < 200 {
                            dump_view(dir, dumped, &view);
                            dumped += 1;
                        }
                    }
                    if let Some(intent) = bots[seat.index()].decide(&view) {
                        let _ = state.apply_intention(seat, state.tick, intent);
                    }
                }
                if rec.h10_alloc_samples < 40 {
                    let (_, calls, bytes) = alloc_scope(|| {
                        let a = state.view_for(Seat::A, &events);
                        let b = state.view_for(Seat::B, &events);
                        (a, b)
                    });
                    let (_, scalls, sbytes) = alloc_scope(|| shared_views(&state, &events));
                    rec.h10_alloc_two = (rec.h10_alloc_two.0 + calls, rec.h10_alloc_two.1 + bytes);
                    rec.h10_alloc_shared = (
                        rec.h10_alloc_shared.0 + scalls,
                        rec.h10_alloc_shared.1 + sbytes,
                    );
                    rec.h10_alloc_samples += 1;
                }
                if rec.views % 4 == 0 {
                    let start = Instant::now();
                    let a = state.view_for(Seat::A, &events);
                    let b = state.view_for(Seat::B, &events);
                    drop((a, b));
                    rec.h10_two.push(start.elapsed().as_nanos() as u64);
                    let start = Instant::now();
                    let (a, b) = shared_views(&state, &events);
                    drop((a, b));
                    rec.h10_shared.push(start.elapsed().as_nanos() as u64);
                }
                events.clear();
            }
            if acc8 >= 20 {
                for seat in Seat::BOTH {
                    let view = state.view_for(seat, &events8);
                    rec.record_freq(1, seat, &view);
                }
                events8.clear();
                acc8 -= 20;
            }
            acc8 += 8;
            if state.tick.is_multiple_of(4) {
                for seat in Seat::BOTH {
                    let view = state.view_for(seat, &events5);
                    rec.record_freq(2, seat, &view);
                }
                events5.clear();
            }
            let step = state.step();
            events.extend(step.iter().copied());
            events8.extend(step.iter().copied());
            events5.extend(step.iter().copied());
        }
        rec.matches += 1;
    }
    rec
}

// ---------- Integrity edge cases ----------

struct Integrity {
    reconnect_c: bool,
    reconnect_b: bool,
    delta_loss_mismatches: u64,
    delta_loss_recovered: bool,
    missing_static_c: bool,
    missing_static_b: bool,
    hidden_ok: bool,
    edge_no_units: bool,
    edge_one_unit: bool,
    edge_200_units: bool,
    max_view_bytes: usize,
    size_ceiling_ok: bool,
}

fn integrity_tests(max_synthetic_units: usize) -> Integrity {
    let arcanas = [ArcanaPreset::DEFAULT; 2];
    let rules = Ruleset::canon();

    let state = MatchState::with_arcanas(7, arcanas);
    let no_units = state.view_for(Seat::A, &[]);
    let mut tracker = StaticTracker::default();
    let mut cache = StaticCache::default();
    let payload = build_c(&no_units, &mut tracker, true);
    let c_ok = rebuild_c(&payload, &mut cache, Seat::A, &arcanas, rules)
        .is_ok_and(|view| view == no_units);
    let mut delta_tracker = DeltaTracker::default();
    let mut delta = DeltaCache::default();
    let payload = build_b(&no_units, &mut delta_tracker, true);
    let b_ok = rebuild_b(&payload, &mut delta, Seat::A, &arcanas, rules)
        .is_ok_and(|view| view == no_units);

    let mut one = MatchState::with_arcanas(11, arcanas);
    while one.phase != Phase::Running {
        one.step();
    }
    one.players[0].energy = 10_000;
    one.players[0].offers[0] = UnitKind::Shieldbearer;
    let _ = one.apply_intention(Seat::A, 1, PlayerIntention::Purchase { offer: 0 });
    one.step();
    let one_view = one.view_for(Seat::A, &[]);
    let mut tracker = StaticTracker::default();
    let mut cache = StaticCache::default();
    let payload = build_c(&one_view, &mut tracker, true);
    let one_ok = rebuild_c(&payload, &mut cache, Seat::A, &arcanas, rules)
        .is_ok_and(|view| view == one_view);

    // A natural match: reconnect mid-combat and one deliberately lost delta.
    let mut state = MatchState::with_arcanas(7, arcanas);
    let mut bots = [Bot::new(7), Bot::new(7 ^ 0x9e37_79b9)];
    let mut events = Vec::new();
    let mut c_client: [StaticCache; 2] = std::array::from_fn(|_| StaticCache::default());
    let mut b_client: [DeltaCache; 2] = std::array::from_fn(|_| DeltaCache::default());
    let mut c_server: [StaticTracker; 2] = std::array::from_fn(|_| StaticTracker::default());
    let mut b_server: [DeltaTracker; 2] = std::array::from_fn(|_| DeltaTracker::default());
    let mut c_views = [0u32; 2];
    let mut b_views = [0u32; 2];
    let mut reconnect_c = false;
    let mut reconnect_b = false;
    let mut delta_loss_mismatches = 0u64;
    let mut delta_loss_recovered = false;
    let mut skipped_delta = false;
    let mut max_view_bytes = 0;
    while state.phase != Phase::Ended && state.tick <= SAMPLE_END + 5 {
        if state.tick.is_multiple_of(VIEW_EVERY) {
            for seat in Seat::BOTH {
                let index = seat.index();
                let view = state.view_for(seat, &events);
                max_view_bytes = max_view_bytes.max(message_len(&view));

                let full_c = c_views[index] == 0;
                c_views[index] += 1;
                let c = build_c(&view, &mut c_server[index], full_c);
                if c_views[index] == 350 {
                    c_client[index] = StaticCache::default();
                    let full = build_c(&view, &mut c_server[index], true);
                    let rebuilt = rebuild_c(&full, &mut c_client[index], seat, &arcanas, rules)
                        .expect("reconnect C reconstructs");
                    assert_same(&rebuilt, &view, "C", "reconnect");
                    reconnect_c = true;
                } else {
                    let rebuilt = rebuild_c(&c, &mut c_client[index], seat, &arcanas, rules)
                        .expect("C reconstructs");
                    assert_same(&rebuilt, &view, "C", "edge");
                }

                let full_b = b_views[index] % KEYFRAME_VIEWS == 0;
                b_views[index] += 1;
                let b = build_b(&view, &mut b_server[index], full_b);
                if b_views[index] == 350 {
                    b_client[index] = DeltaCache::default();
                    let full = build_b(&view, &mut b_server[index], true);
                    let rebuilt = rebuild_b(&full, &mut b_client[index], seat, &arcanas, rules)
                        .expect("reconnect B reconstructs");
                    assert_same(&rebuilt, &view, "B", "reconnect");
                    reconnect_b = true;
                } else if b_views[index] == 355 {
                    // Simulate one lost WebSocket frame: do not apply this delta.
                    skipped_delta = true;
                } else {
                    let rebuilt = rebuild_b(&b, &mut b_client[index], seat, &arcanas, rules)
                        .expect("B reconstructs after loss");
                    if rebuilt != view {
                        delta_loss_mismatches += 1;
                    } else if skipped_delta && full_b {
                        delta_loss_recovered = true;
                    }
                }
                if let Some(intent) = bots[index].decide(&view) {
                    let _ = state.apply_intention(seat, state.tick, intent);
                }
            }
            events.clear();
        }
        events.extend(state.step());
    }

    // Unknown static id: a dynamic unit without a static entry must ask for
    // a full snapshot (the client's reconnect path), never render partial.
    let view = state.view_for(Seat::A, &[]);
    let orphan = UnitDynamic {
        id: 999_999,
        x: 1_000,
        hp: 1,
        status: 0,
        fx: 0,
    };
    let payload = ViewC {
        tick: view.tick,
        extra: extra_of(&view),
        phase: view.phase,
        energy: view.energy,
        offers: view.offers,
        stronghold_hp: view.stronghold_hp,
        full: false,
        static_units: Vec::new(),
        units: vec![orphan],
        synergy: None,
        last_seq: view.last_seq,
        events: Vec::new(),
        outcome: view.outcome,
    };
    let missing_static_c = rebuild_c(
        &payload,
        &mut StaticCache::default(),
        Seat::A,
        &arcanas,
        rules,
    )
    .is_err();
    let payload = ViewB {
        tick: view.tick,
        extra: extra_of(&view),
        phase: view.phase,
        energy: view.energy,
        offers: view.offers,
        stronghold_hp: view.stronghold_hp,
        full: false,
        static_units: Vec::new(),
        units: vec![orphan],
        removed: Vec::new(),
        synergy: None,
        last_seq: view.last_seq,
        events: Vec::new(),
        outcome: view.outcome,
    };
    let missing_static_b = rebuild_b(
        &payload,
        &mut DeltaCache::default(),
        Seat::A,
        &arcanas,
        rules,
    )
    .is_err();

    // Hidden data: the view carries only the seat's own Energy/offers/last_seq.
    let mut hidden = MatchState::with_arcanas(3, arcanas);
    hidden.players[0].energy = 123;
    hidden.players[1].energy = 456;
    hidden.players[0].last_seq = 7;
    hidden.players[1].last_seq = 8;
    let a = hidden.view_for(Seat::A, &[]);
    let b = hidden.view_for(Seat::B, &[]);
    let hidden_ok = a.energy == 123
        && b.energy == 456
        && a.last_seq == 7
        && b.last_seq == 8
        && a.offers == hidden.players[0].offers
        && b.offers == hidden.players[1].offers;

    Integrity {
        reconnect_c,
        reconnect_b,
        delta_loss_mismatches,
        delta_loss_recovered,
        missing_static_c,
        missing_static_b,
        hidden_ok,
        edge_no_units: c_ok && b_ok,
        edge_one_unit: one_ok,
        edge_200_units: max_synthetic_units >= 200,
        max_view_bytes,
        size_ceiling_ok: max_view_bytes < 64 * 1024,
    }
}

// ---------- Output ----------

fn print_summary_row(label: &str, stats: &SizeStats, hz: f64) {
    let s = stats.summary(hz);
    println!(
        "| {label} | {} | {:.1} | {} | {} | {:.1} |",
        s.views, s.avg_b, s.p95_b, s.max_b, s.bytes_s
    );
}

fn shares_line(agg: &ShareAgg) -> String {
    let pct = |n: u64| {
        if agg.total == 0 {
            0.0
        } else {
            n as f64 * 100.0 / agg.total as f64
        }
    };
    format!(
        "header {:.1}% | units {:.1}% (static {:.1}%, max_hp {:.1}%, dynamic {:.1}%) | events {:.1}% | {:.2} units/view | {:.2} events/view | u16 x/hp would save {:.1}% of all bytes",
        pct(agg.header),
        pct(agg.units),
        pct(agg.unit_static),
        pct(agg.unit_max_hp),
        pct(agg.unit_dynamic),
        pct(agg.events),
        agg.units_seen as f64 / agg.views.max(1) as f64,
        agg.events_seen as f64 / agg.views.max(1) as f64,
        pct(agg.unit_dynamic - agg.unit_dynamic_tight),
    )
}

fn sections_line(sections: Sections) -> String {
    let pct = |n: u64| {
        if sections.total == 0 {
            0.0
        } else {
            n as f64 * 100.0 / sections.total as f64
        }
    };
    format!(
        "header {:.1}% | units {:.1}% | static {:.1}% | removed {:.1}% | events {:.1}%",
        pct(sections.header),
        pct(sections.units),
        pct(sections.statics),
        pct(sections.removed),
        pct(sections.events)
    )
}

fn size_json(label: &str, stats: &SizeStats, hz: f64) -> String {
    let s = stats.summary(hz);
    format!(
        "\"{label}\":{{\"views\":{},\"avg_b\":{:.2},\"p95_b\":{},\"max_b\":{},\"bytes_s\":{:.2}}}",
        s.views, s.avg_b, s.p95_b, s.max_b, s.bytes_s
    )
}

fn sections_json(label: &str, s: Sections) -> String {
    format!(
        "\"{label}\":{{\"total\":{},\"header\":{},\"units\":{},\"statics\":{},\"removed\":{},\"events\":{}}}",
        s.total, s.header, s.units, s.statics, s.removed, s.events
    )
}

fn timing_json(label: &str, samples: &Samples) -> String {
    let t = samples.summary();
    format!(
        "\"{label}\":{{\"samples\":{},\"median_us\":{:.3},\"p95_us\":{:.3}}}",
        t.samples, t.median_us, t.p95_us
    )
}

fn main() {
    let dump_dir = std::env::var_os("TBG_AUDIT_VIEW_DIR").map(PathBuf::from);
    if let Some(dir) = &dump_dir {
        std::fs::create_dir_all(dir).expect("create TBG_AUDIT_VIEW_DIR");
    }
    let _ = VIEW_DUMP.set(dump_dir);
    println!("# P30-A view audit");
    println!("Generated by `cargo run --release -p tbg-core --example view_audit`.");
    println!();

    println!("## Synthetic armies (both seats topped up every tick, 10 Hz views)");
    println!();
    println!("| units/side | current B avg | p95 | max | A B avg | B B avg | C B avg | D B avg | current KB/s | C KB/s |");
    println!("|---|---|---|---|---|---|---|---|---|---|");
    let mut synthetic = Vec::new();
    let mut synthetic_recorders = Vec::new();
    for n in [10, 20, 40, 60, 100] {
        let rec = run_synthetic(n);
        let current = rec.current.summary(10.0);
        let a = rec.a.summary(10.0);
        let b = rec.b.summary(10.0);
        let c = rec.c.summary(10.0);
        let d = rec.d.summary(10.0);
        println!(
            "| {n} | {:.0} | {} | {} | {:.0} | {:.0} | {:.0} | {:.0} | {:.2} | {:.2} |",
            current.avg_b,
            current.p95_b,
            current.max_b,
            a.avg_b,
            b.avg_b,
            c.avg_b,
            d.avg_b,
            current.bytes_s / 1000.0,
            c.bytes_s / 1000.0
        );
        synthetic.push(format!(
            "{{\"units_per_side\":{n},{},{},{},{},{},\"shares\":\"{}\",\"sections\":{{{},{},{},{}}},\"timing_us\":{{{},{},{},{},{},{},{},{},{},{},{},{},{}}},\"frequency\":{{\"current\":{{\"10hz_bytes_s\":{:.2},\"8hz_bytes_s\":{:.2},\"5hz_bytes_s\":{:.2}}},\"c\":{{\"10hz_bytes_s\":{:.2},\"8hz_bytes_s\":{:.2},\"5hz_bytes_s\":{:.2}}}}},\"cache_max_bytes\":{{\"c\":{},\"b\":{}}}}}",
            size_json("current", &rec.current, 10.0),
            size_json("a", &rec.a, 10.0),
            size_json("b", &rec.b, 10.0),
            size_json("c", &rec.c, 10.0),
            size_json("d", &rec.d, 10.0),
            shares_line(&rec.shares),
            sections_json("a", rec.sec_a),
            sections_json("b", rec.sec_b),
            sections_json("c", rec.sec_c),
            sections_json("d", rec.sec_d),
            timing_json("build_current", &rec.build_current),
            timing_json("encode_current", &rec.encode_current),
            timing_json("build_a", &rec.build_a),
            timing_json("encode_a", &rec.encode_a),
            timing_json("build_b", &rec.build_b),
            timing_json("encode_b", &rec.encode_b),
            timing_json("build_c", &rec.build_c),
            timing_json("encode_c", &rec.encode_c),
            timing_json("sync_clone_linear", &rec.sync_current),
            timing_json("sync_arc_sorted", &rec.sync_poc),
            timing_json("rebuild_a", &rec.rebuild_a),
            timing_json("rebuild_b", &rec.rebuild_b),
            timing_json("rebuild_c", &rec.rebuild_c),
            rec.current.summary(10.0).bytes_s,
            rec.freq_current[0].summary(8.0).bytes_s,
            rec.freq_current[1].summary(5.0).bytes_s,
            rec.c.summary(10.0).bytes_s,
            rec.freq_c[0].summary(8.0).bytes_s,
            rec.freq_c[1].summary(5.0).bytes_s,
            rec.c_cache_max,
            rec.b_cache_max,
        ));
        synthetic_recorders.push(rec);
    }
    println!();
    println!("### Field shares (canonical current view)");
    println!();
    for (n, rec) in [10, 20, 40, 60, 100].iter().zip(&synthetic_recorders) {
        println!("- {n} units/side: {}", shares_line(&rec.shares));
    }
    println!();

    println!("## Natural bot-vs-bot corpus ({NATURAL_SEEDS} matches, default Arcanas)");
    println!();
    let natural = run_natural();
    let n_current = natural.current.summary(10.0);
    println!(
        "Views: {} ({} per match) | units/view avg {:.2} | max units {} | empty {} | one-unit {}",
        n_current.views,
        n_current.views / natural.matches.max(1),
        natural.units_seen as f64 / natural.views.max(1) as f64,
        natural.max_units,
        natural.empty_views,
        natural.one_unit_views,
    );
    println!();
    println!("| variant | views | avg B | p95 B | max B | B/s per player @10Hz |");
    println!("|---|---|---|---|---|---|");
    print_summary_row("current", &natural.current, 10.0);
    print_summary_row("A derive", &natural.a, 10.0);
    print_summary_row("B delta", &natural.b, 10.0);
    print_summary_row("C static/dynamic", &natural.c, 10.0);
    print_summary_row("D events only", &natural.d, 10.0);
    println!();
    println!("Field shares: {}", shares_line(&natural.shares));
    println!();
    println!("### Header bytes (share of the whole message)");
    println!();
    println!("| field | bytes | share |");
    println!("|---|---|---|");
    for (name, bytes) in &natural.shares.header_fields {
        println!(
            "| {name} | {} | {:.1}% |",
            bytes / natural.shares.views.max(1),
            *bytes as f64 * 100.0 / natural.shares.total as f64
        );
    }
    println!();
    println!(
        "Unit fields exclude the vector count prefix; the small gap to the unit share is that prefix."
    );
    println!("Event counts: {:?}", natural.shares.event_counts);
    println!(
        "Section shares: A [{}] | B [{}] | C [{}] | D [{}]",
        sections_line(natural.sec_a),
        sections_line(natural.sec_b),
        sections_line(natural.sec_c),
        sections_line(natural.sec_d)
    );
    println!(
        "B full keyframes {} of {} views ({:.1}%), changed units/view {:.2}",
        natural.b_full,
        natural.views,
        100.0 * natural.b_full as f64 / natural.views.max(1) as f64,
        natural.b_changed_units as f64 / natural.views.max(1) as f64
    );
    println!();
    println!("### Natural bytes per player by living-unit bucket (current, then C)");
    println!();
    println!("| bucket | views | current avg B | current B/s @10Hz | C avg B | C B/s @10Hz |");
    println!("|---|---|---|---|---|---|");
    const BUCKETS: [&str; 6] = ["0–10", "11–20", "21–40", "41–60", "61–100", "101+"];
    for (i, label) in BUCKETS.iter().enumerate() {
        let current = natural.buckets_current[i].summary(10.0);
        let c = natural.buckets_c[i].summary(10.0);
        println!(
            "| {label} | {} | {:.0} | {:.2} | {:.0} | {:.2} |",
            current.views, current.avg_b, current.bytes_s, c.avg_b, c.bytes_s
        );
    }
    println!();
    println!("Top events: {:?}", natural.shares.event_counts);
    println!();

    println!("## Frequency (same matches; decisions stay at 10 Hz)");
    println!();
    println!(
        "| variant | 10 Hz B/s | 8 Hz B/s | 5 Hz B/s | 10 Hz msgs/s | 8 Hz msgs/s | 5 Hz msgs/s |"
    );
    println!("|---|---|---|---|---|---|---|");
    let s10 = natural.current.summary(10.0);
    let s8 = natural.freq_current[0].summary(8.0);
    let s5 = natural.freq_current[1].summary(5.0);
    let c10 = natural.c.summary(10.0);
    let c8 = natural.freq_c[0].summary(8.0);
    let c5 = natural.freq_c[1].summary(5.0);
    println!(
        "| current | {:.2} | {:.2} | {:.2} | 10 | 8 | 5 |",
        s10.bytes_s, s8.bytes_s, s5.bytes_s
    );
    println!(
        "| C (A + split) | {:.2} | {:.2} | {:.2} | 10 | 8 | 5 |",
        c10.bytes_s, c8.bytes_s, c5.bytes_s
    );
    println!();
    println!();
    println!("### Synthetic frequency, 20 and 100 units/side");
    println!();
    println!("| units/side | variant | 10 Hz B/s | 8 Hz B/s | 5 Hz B/s |");
    println!("|---|---|---|---|---|");
    for (n, rec) in [
        (20, &synthetic_recorders[1]),
        (100, &synthetic_recorders[4]),
    ] {
        println!(
            "| {n} | current | {:.2} | {:.2} | {:.2} |",
            rec.current.summary(10.0).bytes_s,
            rec.freq_current[0].summary(8.0).bytes_s,
            rec.freq_current[1].summary(5.0).bytes_s
        );
        println!(
            "| {n} | C | {:.2} | {:.2} | {:.2} |",
            rec.c.summary(10.0).bytes_s,
            rec.freq_c[0].summary(8.0).bytes_s,
            rec.freq_c[1].summary(5.0).bytes_s
        );
    }
    println!();
    println!(
        "Estimated wire bytes/s per player at 10 Hz: current {:.2}, A {:.2}, B {:.2}, C {:.2} (payload + {} B/message midpoint; range {}–{} B)",
        (s10.avg_b + WIRE_OVERHEAD_MID as f64) * 10.0,
        (natural.a.summary(10.0).avg_b + WIRE_OVERHEAD_MID as f64) * 10.0,
        (natural.b.summary(10.0).avg_b + WIRE_OVERHEAD_MID as f64) * 10.0,
        (c10.avg_b + WIRE_OVERHEAD_MID as f64) * 10.0,
        WIRE_OVERHEAD_MID,
        WIRE_OVERHEAD_MIN,
        WIRE_OVERHEAD_MAX
    );
    println!();

    println!("## CPU per view (median / p95 microseconds, both seats where noted)");
    println!();
    println!("| stage | samples | median µs | p95 µs |");
    println!("|---|---|---|---|");
    let timing_rows = [
        ("current build (one seat)", &natural.build_current),
        ("current encode (one seat)", &natural.encode_current),
        ("A build", &natural.build_a),
        ("A encode", &natural.encode_a),
        ("A reconstruct", &natural.rebuild_a),
        ("B build", &natural.build_b),
        ("B encode", &natural.encode_b),
        ("B reconstruct", &natural.rebuild_b),
        ("C build", &natural.build_c),
        ("C encode", &natural.encode_c),
        ("C reconstruct", &natural.rebuild_c),
    ];
    for (label, samples) in timing_rows {
        let t = samples.summary();
        println!(
            "| {label} | {} | {:.2} | {:.2} |",
            t.samples, t.median_us, t.p95_us
        );
    }
    println!();
    let med = |samples: &Samples| samples.summary().median_us;
    let cpu20 = &synthetic_recorders[1];
    let cpu100 = &synthetic_recorders[4];
    println!("### Synthetic CPU per view (median µs)");
    println!();
    println!("| stage | 20 units/side | 100 units/side |");
    println!("|---|---|---|");
    for (label, a, b) in [
        (
            "current build",
            med(&cpu20.build_current),
            med(&cpu100.build_current),
        ),
        (
            "current encode",
            med(&cpu20.encode_current),
            med(&cpu100.encode_current),
        ),
        (
            "A build+encode",
            med(&cpu20.build_a) + med(&cpu20.encode_a),
            med(&cpu100.build_a) + med(&cpu100.encode_a),
        ),
        (
            "B build+encode",
            med(&cpu20.build_b) + med(&cpu20.encode_b),
            med(&cpu100.build_b) + med(&cpu100.encode_b),
        ),
        (
            "C build+encode",
            med(&cpu20.build_c) + med(&cpu20.encode_c),
            med(&cpu100.build_c) + med(&cpu100.encode_c),
        ),
        (
            "A reconstruct",
            med(&cpu20.rebuild_a),
            med(&cpu100.rebuild_a),
        ),
        (
            "B reconstruct",
            med(&cpu20.rebuild_b),
            med(&cpu100.rebuild_b),
        ),
        (
            "C reconstruct",
            med(&cpu20.rebuild_c),
            med(&cpu100.rebuild_c),
        ),
        (
            "client sync: clone+linear",
            med(&cpu20.sync_current),
            med(&cpu100.sync_current),
        ),
        (
            "client sync: Arc+sorted",
            med(&cpu20.sync_poc),
            med(&cpu100.sync_poc),
        ),
    ] {
        println!("| {label} | {a:.2} | {b:.2} |");
    }
    println!();
    let sync_current = med(&cpu100.sync_current);
    let sync_poc = med(&cpu100.sync_poc);
    println!(
        "Client sync at 100 units/side: clone+linear {:.2} µs, Arc+sorted {:.2} µs ({:.1}% of the sync cost removed). Browser FPS is blocked on this machine (see report).",
        sync_current,
        sync_poc,
        if sync_current > 0.0 {
            100.0 * (sync_current - sync_poc) / sync_current
        } else {
            0.0
        }
    );
    println!();

    println!("## H10 shared build (two seats, same tick)");
    println!();
    let two = natural.h10_two.summary();
    let shared = natural.h10_shared.summary();
    println!("| two view_for calls | shared unit list + clone | change |",);
    println!("|---|---|---|");
    println!(
        "| {:.2} µs median / {:.2} µs p95 | {:.2} µs median / {:.2} µs p95 | {:.1}% |",
        two.median_us,
        two.p95_us,
        shared.median_us,
        shared.p95_us,
        if two.median_us > 0.0 {
            100.0 * (shared.median_us - two.median_us) / two.median_us
        } else {
            0.0
        }
    );
    println!(
        "Allocations per two-view build ({} samples): two view_for {} calls / {} B; shared {} calls / {} B",
        natural.h10_alloc_samples,
        natural.h10_alloc_two.0,
        natural.h10_alloc_two.1,
        natural.h10_alloc_shared.0,
        natural.h10_alloc_shared.1
    );
    let encode_two = 2.0 * med(&natural.encode_current);
    let current_two = two.median_us + encode_two;
    let shared_two = shared.median_us + encode_two;
    println!(
        "Build+encode for two seats: current {:.2} µs, shared {:.2} µs ({:.1}% change). H10 needs -15%.",
        current_two,
        shared_two,
        100.0 * (shared_two - current_two) / current_two
    );
    println!(
        "Client cache retained per seat (key+value bytes, excluding BTreeMap node overhead): C {} B, B {} B",
        natural.c_cache_max, natural.b_cache_max
    );
    println!();

    let max_synthetic_units = synthetic_recorders[4].max_units;
    let max_synthetic_view = synthetic_recorders[4].current.summary(10.0).max_b;
    let mut integrity = integrity_tests(max_synthetic_units);
    integrity.max_view_bytes = integrity.max_view_bytes.max(max_synthetic_view);
    integrity.size_ceiling_ok = integrity.max_view_bytes < 64 * 1024;
    println!("## Integrity");
    println!();
    println!(
        "100 seeds: A/C/B reconstructed every view with zero mismatches (panic otherwise); D is not reconstructable by design."
    );
    println!(
        "Edge cases: no units {}, one unit {}, 200+ units {} (synthetic 100/side incl. summons), reconnect C {} B {}, delta-loss views mismatched {} then keyframe recovered {}, missing static id rejected C {} B {}, hidden data isolated {}, max view {} B (< 64 KiB {})",
        integrity.edge_no_units,
        integrity.edge_one_unit,
        integrity.edge_200_units,
        integrity.reconnect_c,
        integrity.reconnect_b,
        integrity.delta_loss_mismatches,
        integrity.delta_loss_recovered,
        integrity.missing_static_c,
        integrity.missing_static_b,
        integrity.hidden_ok,
        integrity.max_view_bytes,
        integrity.size_ceiling_ok
    );
    println!();

    // Raw JSON for the report and scripts.
    println!("## JSON");
    println!("```json");
    println!("{{");
    println!("  \"meta\": {{");
    println!(
        "    \"natural_seeds\": {NATURAL_SEEDS}, \"synthetic_ticks\": {SYNTHETIC_TICKS}, \"keyframe_views\": {KEYFRAME_VIEWS},"
    );
    println!(
        "    \"wire_overhead_min_b\": {WIRE_OVERHEAD_MIN}, \"wire_overhead_mid_b\": {WIRE_OVERHEAD_MID}, \"wire_overhead_max_b\": {WIRE_OVERHEAD_MAX},"
    );
    println!("    \"payload_note\": \"view bytes include the 1-byte ServerMsg variant tag; section bytes exclude it from their own field sums\"");
    println!("  }},");
    println!("  \"synthetic\": [{}],", synthetic.join(","));
    println!("  \"natural\": {{");
    println!("    \"matches\": {},", natural.matches);
    println!("    {},", size_json("current", &natural.current, 10.0));
    println!("    {},", size_json("a", &natural.a, 10.0));
    println!("    {},", size_json("b", &natural.b, 10.0));
    println!("    {},", size_json("c", &natural.c, 10.0));
    println!("    {},", size_json("d", &natural.d, 10.0));
    println!(
        "    \"sections\": {{{},{},{},{}}},",
        sections_json("a", natural.sec_a),
        sections_json("b", natural.sec_b),
        sections_json("c", natural.sec_c),
        sections_json("d", natural.sec_d)
    );
    println!(
        "    \"h10\": {{ {}, {}, \"build_encode_two_us\": {:.3}, \"alloc_two_calls\": {}, \"alloc_two_bytes\": {}, \"alloc_shared_calls\": {}, \"alloc_shared_bytes\": {}, \"alloc_samples\": {} }},",
        timing_json("two_view_for", &natural.h10_two),
        timing_json("shared", &natural.h10_shared),
        shared_two,
        natural.h10_alloc_two.0,
        natural.h10_alloc_two.1,
        natural.h10_alloc_shared.0,
        natural.h10_alloc_shared.1,
        natural.h10_alloc_samples
    );
    println!(
        "    \"cache_max_bytes\": {{\"c\": {}, \"b\": {}}},",
        natural.c_cache_max, natural.b_cache_max
    );
    println!(
        "    \"client_sync_us\": {{{}, {}}},",
        timing_json("clone_linear", &natural.sync_current),
        timing_json("arc_sorted", &natural.sync_poc)
    );
    let header_json: Vec<String> = natural
        .shares
        .header_fields
        .iter()
        .map(|(name, bytes)| format!("\"{name}\":{bytes}"))
        .collect();
    let event_json: Vec<String> = natural
        .shares
        .event_counts
        .iter()
        .map(|(name, count)| format!("\"{name}\":{count}"))
        .collect();
    println!("    \"header_fields\": {{{}}},", header_json.join(","));
    println!("    \"event_counts\": {{{}}},", event_json.join(","));
    println!(
        "    \"timing_us\": {{{},{},{},{},{},{},{},{},{},{},{}}},",
        timing_json("build_current", &natural.build_current),
        timing_json("encode_current", &natural.encode_current),
        timing_json("build_a", &natural.build_a),
        timing_json("encode_a", &natural.encode_a),
        timing_json("build_b", &natural.build_b),
        timing_json("encode_b", &natural.encode_b),
        timing_json("build_c", &natural.build_c),
        timing_json("encode_c", &natural.encode_c),
        timing_json("rebuild_a", &natural.rebuild_a),
        timing_json("rebuild_b", &natural.rebuild_b),
        timing_json("rebuild_c", &natural.rebuild_c)
    );
    println!(
        "    \"frequency\": {{\"current\":{{\"10hz_bytes_s\":{:.2},\"8hz_bytes_s\":{:.2},\"5hz_bytes_s\":{:.2}}},\"c\":{{\"10hz_bytes_s\":{:.2},\"8hz_bytes_s\":{:.2},\"5hz_bytes_s\":{:.2}}}}},",
        s10.bytes_s, s8.bytes_s, s5.bytes_s, c10.bytes_s, c8.bytes_s, c5.bytes_s
    );
    println!(
        "    \"integrity\": {{\"reconnect_c\":{},\"reconnect_b\":{},\"delta_loss_mismatches\":{},\"delta_loss_recovered\":{},\"missing_static_c\":{},\"missing_static_b\":{},\"hidden_ok\":{},\"edge_no_units\":{},\"edge_one_unit\":{},\"edge_200_units\":{},\"max_view_bytes\":{},\"size_ceiling_ok\":{}}}",
        integrity.reconnect_c,
        integrity.reconnect_b,
        integrity.delta_loss_mismatches,
        integrity.delta_loss_recovered,
        integrity.missing_static_c,
        integrity.missing_static_b,
        integrity.hidden_ok,
        integrity.edge_no_units,
        integrity.edge_one_unit,
        integrity.edge_200_units,
        integrity.max_view_bytes,
        integrity.size_ceiling_ok
    );
    println!("  }}");
    println!("}}");
    println!("```");
}
