# P21 — Combat effects, projectiles, and sound kit

**Depends on:** P20 reviewed.
**Branch:** `phase/p21-vfx-kit`.
**Difficulty:** medium.

## Goal

Every hit, heal, status, evolution, and death reads instantly and feels
weighty — with a shared, pooled, budgeted effects kit that the eight
character phases (P22–P29) plug into. Medieval and family-friendly: dust,
sparks, splinters, bone chips, leaves, soul-fire — no blood.

## Deliverable

- `vfx/` kit v2 with pooling, tiers, and a fixed live-effect budget.
- Projectiles: arrow, crossbow bolt, heavy bolt (Ogre Archer), hex orb,
  petal stream, ground ripple (roots).
- Impacts, status visuals (slow, root), heal, evolve, synergy, death
  variants per faction, Stronghold debris, Large-unit footstep dust.
- Synthesized medieval sound set v2 (variants per cue) via `make_sfx.rs`.

## Read first (only these)

- `AGENTS.md`
- `docs/ART-DIRECTION.md` §1–§4, §9
- Client: `src/vfx.rs`, `src/vfx/effects.rs`, `src/visuals/cues.rs`,
  `src/sync/events.rs`, `src/audio.rs`, `examples/make_sfx.rs`

## Do NOT

- Add textures, particle crates, or post-processing; effects are small
  vertex-colored meshes (unlit where they glow).
- Show information the simulation did not send (effects follow events).
- Add music (out of scope; no track exists).

## Steps

### 1. Pool and budget
`vfx/pool.rs`: preallocated entities per effect mesh (hidden when idle);
`spawn` takes from the pool, never allocates in steady state. Live effect
cap: 150 (High), 100 (Medium), 60 (Low); when full, drop the oldest
cosmetic effect first (never a projectile).

### 2. Projectiles
`vfx/projectiles.rs`: one model + flight rule each — arrow (team-color
fletching, slight arc), bolt (flat, fast), heavy bolt (arc, spin, dust
trail), hex orb (curving, soul-fire glow, 3 trailing motes), petal stream
(5 petals along a curve to the ally), ground ripple (raised dirt bumps
traveling to the target). Launch on the attacker's `Release` clip event
(P16), arrive at the `Hit` event; if `Hit` arrives first, snap to impact.

### 3. Impacts and statuses
Melee: dust + 4 sparks (metal) or splinters (wood) by attacker; ranged:
small dust + stuck arrow for 0.5 s (Medium/High). Slow: two violet-green
spectral chains at the target's feet while slowed. Root: 3 roots wrapping
the legs while rooted. Heal: 6 petals rising + soft green glow on the
ally. Splash: ground ring sized to the splash radius from `tbg-core`
content (exact radius — it is information).

### 4. Evolve, synergy, death
Evolve: gold light rays + faction-colored burst + 0.15 s white flash on
the figure. Synergy activation: faction crest pulse above each affected
unit. Death by faction: Humans dust + dropped weapon; Undead bones scatter;
Machines gears and steam pop; Plants leaves and petals. Stronghold: debris
chunks + smoke per damage state (replaces P18 placeholders).

### 5. Hit feedback
Hit flash (existing) + 50 ms hit-stop on the target (P16) + a 1-frame
scale squash (0.95) on heavy hits (two-slot attackers and Stronghold
bolts). Reduced motion (P19 setting) disables squash and hit-stop.

### 6. Sound set v2
Extend `make_sfx.rs` (original synthesis only): bow twang (3 variants),
crossbow thunk, heavy bolt release, shield bash, sword/mace hits (3),
saw whir loop + saw hit, bone crack, hex whisper, bloom chime, roots
rumble, ogre grunt, footstep thud (Large), Stronghold hit and collapse,
evolve fanfare, synergy chime, victory/defeat stingers. Random variant +
±5 % pitch per play; per-sound throttle (existing). Total audio ≤ 450 KB.

### 7. Gallery
`?gallery=vfx`: every projectile, impact, status, and death in a grid on
loop; `&slow=0.25` supported.

## Acceptance checklist

- [ ] All verification commands from `AGENTS.md` pass; `trunk build --release` passes
- [ ] Zero per-frame allocations in steady state (count pool misses in the
      overlay; must be 0 after warm-up in a busy match)
- [ ] Busy match FPS within 5 % of P20 on a mid-range phone (Low and Medium)
- [ ] Audio ≤ 450 KB; first-load assets ≤ 1.5 MB
- [ ] Owner approves feel in a live match (report)

## Reviewer checklist

- Splash ring radius equals the simulation value.
- Effects are triggered only by simulation events or clip events.

## Phase report

_Executor fills this in._
