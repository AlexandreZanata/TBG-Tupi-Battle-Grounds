# P16 — Animation sets per class (Archetype)

**Depends on:** P15 signed off.
**Branch:** `phase/p16-class-animation`.
**Difficulty:** high (animation quality); the system parts are medium.

## Goal

Every `Archetype` moves in its own way — a Tank marches and bashes, a DPS
runs and combos, a Ranged unit draws and releases, a Control unit
channels, a Support unit blooms — on both body plans, with professional
timing that matches the simulation exactly. Characters in P22–P29 then
only tune and add signature details.

## Deliverable

- Clip library v2: shared clips + one clip set per Archetype + `Large`
  variants (ART-DIRECTION §4), keyed on rig v2.
- Animation controller: state selection from simulation events,
  crossfades, alternating attacks, directional hits, death variants,
  crowd phase offsets, hit-stop.
- `?gallery=classes`: each Archetype set on a gray-box Humanoid and Large
  body side by side, plus timing overlays.

## Read first (only these)

- `AGENTS.md`
- `docs/ART-DIRECTION.md` §4 (rig and motion language) and §9 (budgets)
- `docs/phases/P15-art-direction-and-rig.md` report
- Client: `src/art/anim.rs`, `src/art/clips.rs`, `src/art/rig.rs`,
  `src/art/springs.rs`, `src/visuals/cues.rs`, `src/sync/events.rs`
- Core (read only): `content.rs` (`WINDUP`, unit speeds and attack intervals)
- Bevy examples: `animation/easing_functions.rs`, `animation/eased_motion.rs`

## Do NOT

- Change the simulation, its timing, or the order of events.
- Add Bevy's animation feature, glTF, or skinning.
- Make any clip longer than the unit's attack interval (an attack clip must
  finish before the next `AttackStart`; scale it down if needed).

## Steps

### 1. Clip data model v2
`art/clips/` folder: `mod.rs` (types), `shared.rs`, `tank.rs`, `dps.rs`,
`ranged.rs`, `control.rs`, `support.rs` (one Archetype per file; key tables are
content and may be dense). A clip = duration, looping, per-joint tracks of
eased keys, events (`Strike`, `Release`, `Footstep`), and a `Large`
timing multiplier (default 1.25).

### 2. Shared clips
`Spawn` (step out + settle, 0.45 s), `Hit` (directional: from the
attacker's side; additive over the current clip, 0.25 s), `DeathA`
(fall backwards), `DeathB` (crumple to knees then forward), `Evolve`
(gather → burst pose, 0.6 s), `Victory`, `Defeat`.

### 3. Archetype sets
For each Archetype, `Idle`, `Move`, `AttackA`, `AttackB` (alternating;
Ranged/Control/Support may reuse A), with the events and contact times of
ART-DIRECTION §4:
- Tank: shield bash (A) and overhead swing (B), `Strike` at 0.30 s; heavy
  march with visible weight transfer; idle with shield raised.
- DPS: fast two-hit combo, `Strike` at 0.30 s on the second hit (the first
  is a feint swing), short dash-in on the first frames.
- Ranged: draw (0.00–0.14) → anchor (0.14–0.18) → `Release` at 0.18 →
  follow-through and lower the bow; a reload/nock idle variant.
- Control: raise staff → channel glow → thrust, `Release` at 0.22.
- Support: lift → bloom (hands open) → point at ally, `Release` at 0.22.
- `Large`: same sets with the multiplier, wider stance, and footfall
  events that trigger ground dust (P21) and a 2 px camera-free screen
  shake on the closest unit only.

### 4. Controller
`art/controller.rs`:
- Base locomotion = `Move` if the unit's x changed between the last two
  views, else `Idle`; crossfade 0.12 s.
- `AttackStart` → next of `AttackA`/`AttackB`; time-scaled so its event
  lands after `AttackStart` arrives at: `Strike` 300 ms (melee), `Release`
  180 ms (Ranged, the projectile then flies ~120 ms) or 220 ms (Control,
  Support) — ART-DIRECTION §4 — regardless of the authored duration.
- `Hit` → additive directional `Hit`; `Death` → `DeathA` or `DeathB`
  (by unit id parity); `Evolved` → `Evolve`; match end → `Victory` for the
  winner's units, `Defeat` for the loser's.
- Hit-stop: on `Hit`, the target's animation pauses 50 ms (client only).
- Crowd: each unit has a phase offset from its id; idle/move never sync.
- **Check:** unit tests: every attack clip's `Strike`/`Release` lands at
  its contract time ± 1 frame (16 ms) after scaling; alternation A/B;
  Large multiplier.

### 5. Gallery and timing overlay
`?gallery=classes`: five rows (Archetypes) × two body plans, all clips
looping; `&timing` draws a small bar under each figure with the strike/
release marker. Contact sheets for every attack at 0/25/50/75 %.

### 6. Hook every live unit
Map each unit's Archetype and body plan (two-slot → `Large`) to its set,
so all 8 current figures already move with the new language before their
individual refinement.

## Acceptance checklist

- [ ] All verification commands from `AGENTS.md` pass; `trunk build --release` passes
- [ ] Timing tests pass; contact sheets in `docs/media/p16/`
- [ ] Live match: no foot sliding on any unit at normal speed (record a
      short slow-motion capture or contact sheet)
- [ ] Animation CPU: 30 units at 60 FPS desktop and ≥ 30 FPS mid-range phone
- [ ] Owner approves the five class sets (report)

## Risks and what to do

| Problem | Action |
|---------|--------|
| A clip reads well alone but not at phone size | Exaggerate key poses 20–30 %; review at 390×844 first |
| Strike time and simulation drift under lag | Timing is relative to `AttackStart` arrival; the view buffer already delays both equally |
| Too many unique clips to maintain | Keep sets per Archetype; per-unit uniqueness comes from props and signature extras only |

## Reviewer checklist

- No clip changes simulation behavior or timing.
- Every attack clip's event time is asserted by a test.
- Large units visibly heavier, not just slower.

## Phase report

_Executor fills this in: timing tables, test output, screenshots, owner
approval, deviations, blocked items._
