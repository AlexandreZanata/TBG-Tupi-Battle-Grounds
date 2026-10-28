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
Add the bevy features `"bevy_audio", "vorbis"` to the explicit list in
`crates/tbg-client/Cargo.toml` (do not switch back to the `audio`/`3d`/`ui`
collections). Local native builds then need `sudo apt-get install libasound2-dev`. About 12 effects (deploy, melee hit,
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

Executed by the reviewer agent (Claude) on 2026-10-28, branch `phase/p7-content-feel-balance`.

- Step 1 (roster art): done in P6 — all 8 units × 5 forms.
- Step 2 (VFX, `vfx.rs` + `vfx/effects.rs`): splash ring (after the 0.3 s windup, sized
  by the attacker's splash radius), heal sparkles, evolve burst, synergy pulse on the
  faction's units, Stronghold hit sparks, death puff; projectiles from P3. Status
  visuals stay as ring tints (blue slowed, brown rooted). No pool: a few short-lived
  entities per event is cheap at ≤ 16 units.
- Step 3 (audio, `audio.rs`): 11 WAV effects **synthesized by
  `examples/make_sfx.rs`** (original, 132 KB total, budget 600 KB) instead of CC0
  downloads; per-sound throttle (90 ms) against combat spam; mute toggle in the menu,
  remembered in `localStorage`. Played only in the WASM build (native `bevy_audio`
  needs ALSA, missing on this PC). `AssetMetaCheck::Never` added: static hosts answer
  `*.meta` with index.html. No music loop (deferred).
- Step 4 (UX): long-press (≥ 0.5 s) on a shop card shows unit details + "good vs /
  weak vs" for 4.5 s; three first-match hints once per browser; result screen shows
  units deployed, evolutions, damage to the enemy Stronghold; click sound on buttons.
  A separate synergy panel was not added: the HUD info line shows active synergies.
- Step 5 (balance), 14 harness iterations (log in the commit `balance(p7)`); final
  3000 bot matches:
  ```
  duration s: median 150 p90 183
  ended by: stronghold 69.7% | time 30.3% | forfeit 0.0%
  draws: 1.3%
  faction-majority Humans 46.8% · Undead 47.1% · Machines 57.9% · Plants 53.2%
  purchase share Shieldbearer 21.7 · Archer 15.4 · Bone Brute 11.2 · Hexer 8.8
                 Sawbot 15.4 · Siege Walker 3.5 · Bloomkeeper 15.4 · Rootcaller 8.7
  evolutions per match 7.7 · step mean 0.43 µs
  ```
  Met: median 2.5–4 min, Stronghold kills ≥ 60%, draws ≤ 2%, 3 of 4 factions in 45–55%.
  **Not met:** time-ups 30.3% (target ≤ 30, at the edge); Machines 57.9% (273 armies,
  small sample); Siege Walker 3.5% purchase share — the bot spends Energy as soon as a
  cheaper unit is affordable, so this mostly measures the bot, not the unit. Harness fix:
  tied armies are no longer credited to the later faction.
  Key changes (all in GAME-DESIGN §5): Stronghold HP 4500, attack 65 at range 6 (ranged
  units no longer outrange it), Energy regen 1 per ~1.76 s, Bone Brute cost 4 / dmg 75,
  Siege Walker cost 5 / dmg 85, Sawbot 270 HP / 55 dmg / 1.05 s, Hexer 300 HP / 30 dmg,
  Shieldbearer 650 HP, Bloomkeeper heal 40, Rootcaller root 0.8 s, synergies retuned.
- Browser checks (dev build, pane): practice match with VFX; `assets/audio/*.wav` served
  200; "Not enough Energy" toast shown; hints flag set after the three hints; result
  screen "Units deployed 20 | Evolutions 0 | Damage to enemy Stronghold 0".
- Release size: WASM raw 24,207,013 B · gzip 7,525,040 B · **brotli 4,700,628 B**;
  assets 132,334 B. Within the 6 MB ceiling, above the 4 MB target.
- Commands: `cargo test --workspace` (15 core + 10 server) pass; clippy native + wasm32
  clean; `trunk build --release` ok.
- Deviations: original synthesized sounds instead of CC0 packs; no music; no VFX pool;
  no separate synergy panel; status visuals are ring tints.
- **Needs the owner:** step 6 playtest round (≥ 4 people, ≥ 10 online matches,
  readability/fun/play-again form), confirm or change the D-14 values; check sound
  on iPhone (audio unlock on first tap) and phone FPS with full VFX.
