# P7 — Content, feel, and balance

**Depends on:** P6 reviewed, look approved by the owner.
**Branch:** `phase/p7-content-feel-balance` (step 1 may use one branch per
faction: `phase/p7-art-<faction>`).
**Difficulty:** low–medium; step 5 needs care.

## Goal

The MVP is fun and readable: all 8 units have final figurines and
evolutions, effects make combat legible, there is sound, and the balance
meets the targets in GAME-DESIGN §6.

## Deliverable

A playtest-ready build deployed to the P5 environment, a balance report,
and a playtest report.

## Read first (only these)

- `AGENTS.md`
- `docs/GAME-DESIGN.md` §5, §6
- `docs/ARCHITECTURE.md` §7
- Client: `src/art/` (from P6), `visuals.rs`, `ui/hud.rs`, `ui/evolve.rs`
- `crates/tbg-core/src/content.rs`, `examples/balance.rs`
- Bevy examples: `audio/audio.rs`, `audio/audio_control.rs`

## Do NOT

- Change rules or code in `tbg-core` other than numbers in `content.rs`
  (step 5).
- Touch the server, protocol, or network code.
- Use any asset whose license is not CC0 (sounds) or SIL OFL (font).

## Steps

### 1. Final roster art (one agent per faction file, can run in parallel)
Files: `src/art/roster/{humans,undead,machines,plants}.rs` only.
For each unit: base + 4 evolved forms following the Shieldbearer
quality bar and P6 checklist. Look guide:

| Unit | Silhouette idea | Branch A look | Branch B look |
|------|-----------------|---------------|---------------|
| Archer | hooded, longbow taller than body | Longbow/Marksman: longer bow, quiver, feather cap | Volley/Arrow Rain: twin quivers, shorter bow, scarf |
| Bone Brute (2 slots) | hunched skeleton giant, huge club | Ossified/Bone Wall: bone plates, shield of ribs | Butcher/Reaper: cleaver → scythe |
| Hexer | thin, hood, glowing staff (`#7CF29A`) | Withering/Doom: skull staff, bigger glow | Plague/Pestilence: censer with green cloud spheres |
| Sawbot | squat robot, circular saw arm (`#FF8A3D` accents) | Overclock/Shredder: twin saws, exhaust pipes | Plating/Juggernaut: thick armor slabs |
| Siege Walker (2 slots) | two-legged mech with shoulder cannon | Longshot/Artillery: long barrel | Shrapnel/Concussion: triple barrel / drum |
| Bloomkeeper | small, flower head (`#F28AB2`), leaf cape | Blossom/Grove: bigger bloom, petal ring | Bramble/Thornwarden: thorny bark armor |
| Rootcaller | bark-skinned, root whips | Deep Roots/Strangle: long root arms | Spores/Overgrowth: mushroom cap, spore puffs |

Gallery screenshots per faction (desktop + phone) into `docs/media/p7/`.

### 2. Effects
File: `src/vfx.rs` (new; called from `visuals.rs`). Pooled entities (reuse,
don't spawn/despawn every hit). Projectile per ranged unit (arrow, hex
bolt, cannon ball), splash ring on the ground, slow (blue droplets) and
root (brown vines around feet) status visuals, heal sparkles, evolve burst,
synergy activation pulse on that faction's units, Stronghold hit sparks and
collapse debris.

### 3. Audio
Enable the bevy `audio` feature. About 12 effects (deploy, melee hit,
ranged shot, cannon, heal, evolve, synergy, Stronghold hit, Stronghold
destroyed, victory, defeat, button) and one music loop, all **CC0** (for
example Kenney audio packs), OGG mono 22 kHz, ≤ 600 KB total, in
`crates/tbg-client/assets/audio/`. Add each source and license to
`THIRD_PARTY_NOTICES.md`. Audio starts only after the first tap (iOS
rule). Mute toggle in the menu, remembered in `localStorage`.

### 4. UX polish
Shop cards show a faction color strip and an archetype icon (simple
shapes); long-press (500 ms) on a card or unit shows stats and "Strong
against / Weak against" text from a small table in `ui/`; synergy panel
listing own factions with level and bonus text; three first-match hints
(buy, evolve, synergy) shown once (remembered in `localStorage`); result
screen stats: units bought, evolutions, damage dealt to the enemy
Stronghold.

### 5. Balance pass
1. Run `cargo run --release -p tbg-core --example balance -- 2000`.
2. Compare with GAME-DESIGN §6 harness-measurable targets: median duration
   2.5–4 min; Stronghold kills ≥ 60%; overtime ≤ 30%; draws ≤ 2%; every
   unit purchase share ≥ 5%; no faction-majority win rate outside 45–55%.
3. Change **one** number at a time in `content.rs`, re-run, keep the change
   if it moves toward the targets. Max 20 iterations; log each iteration
   (change → key metrics) in the report.
4. In the same commit as each kept change, update the matching table in
   GAME-DESIGN §5.

### 6. Playtest round (owner organizes, executor supports)
Deploy with `bash deploy/deploy.sh`. At least 4 people, at least 10 online
matches. After each match each player answers: readable? (1–5) · fun?
(1–5) · why did you lose/win? · play again? (yes/no). Combine with SQLite
records (`duration_s`, reasons, purchases). Fix the top three issues found
(each fix is its own commit). The owner confirms or changes the D-14
values in `docs/DECISIONS.md`.

## Acceptance checklist

- [ ] All verification commands from `AGENTS.md` pass; `trunk build --release` passes
- [ ] Balance harness meets the step-5 targets (paste the final output)
- [ ] Playtest: average readability ≥ 4/5; "play again" ≥ 50%
- [ ] ≥ 30 FPS on both phones in a busy match with VFX and audio
- [ ] Compressed download total (WASM + assets) within ARCHITECTURE §10 budgets
- [ ] Every asset listed in `THIRD_PARTY_NOTICES.md` with license

## Risks and what to do

| Problem | Action |
|---------|--------|
| Art differs between faction agents | Same palette file, same P6 checklist; reviewer compares gallery screenshots side by side |
| Balance oscillates | Smaller steps (5%); change cost before stats |
| Audio silent on iPhone | Start audio inside the first tap handler |

## Reviewer checklist

- Only numbers changed in `tbg-core`, mirrored in GAME-DESIGN §5.
- Licenses recorded; no non-CC0 audio.
- VFX pooled; FPS evidence on phones.

## Phase report

*(Executor fills this in at the end.)*

- Commands run and results:
- Balance iterations table and final harness output:
- Playtest summary (players, matches, scores, top issues fixed):
- Deviations:
- Blocked items:
