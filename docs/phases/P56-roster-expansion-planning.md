# P56 — Roster expansion planning (21 units, 3 Factions)

**Planning only — no gameplay code, assets, animations or production behavior were changed.**
**Depends on:** remote `main` at `6ef155b`; reviewed integration of
`release/roster-online` (PR #16) before any implementation; conditional
reuse of P54/P55 art kits after their own integration and re-audit.
**Branch:** `phase/p56-roster-expansion-planning` (from freshly fetched
`origin/main` at `6ef155b`).
**Status:** planning documents only; implementation requires separate
owner authorization.
**Difficulty:** medium. Audit, modeling and documentation; no code.

> Owner directive 2026-10-30 is binding: **text controls names, Faction,
> Archetype, functions, abilities and numbers; every PNG is visual
> reference only.** The Monsters are Grumak, Rasgarr, Esporax and
> Murmúrio. Proper names keep their source diacritics; all other prose
> is ASCII.

## Read first

- `AGENTS.md`
- `docs/GAME-DESIGN.md` §11 (inactive numerical registry, Proposed)
- `docs/roster/P56-AUDIT-AND-INTEGRATION.md` (inventory, main audit,
  shared competitive contracts, integration matrix, Factions, interactions,
  performance reasoning)
- `docs/roster/P56-CATALOG.md` (21 technical sheets, art/animation briefs,
  comparative worksheet)
- `docs/ROADMAP.md`, `docs/DECISIONS.md`, `docs/ARCHITECTURE.md`

Do not read `archive/`. Do not start from `phase/p30d-server-performance`
or any other development branch.

## Goal

Produce an executable expansion plan for 21 text-sourced character
concepts and 3 new Factions that a future executor can implement without
reinventing the main decisions, reusing the real engine wherever
possible and flagging every structural cost honestly.

## Non-goals (explicit exclusions)

No functional characters, no `tbg-core`/server/client edits, no protocol
change, no 3D models, no executable animations, no assets, no balance
change, no new Faction integration, no Shop pool update, no production
migration, no new dependency, no deploy, no merge, no change to `main`.
The only permitted change in this phase is documentation.

## Diagnosis (audited main)

Remote `main` `6ef155b` is an integer, deterministic, authoritative
1v1 engine: 20 Hz, 16,000 milli-tile BattleLine, six-tick windups,
single Slow/Root slot, Guard-first targeting, five distinct uniform Shop
offers over ten purchased kinds, four Factions, forty Augments, fifteen
Arcana nodes, selectable Stronghold kinds, versioned binary protocol.
Unit absorption shields, per-victim heal-cut/poison/bleed/mark slots,
drill fusion, seven-Faction arrays and UnitFx shields exist only in
unintegrated branches (notably PR #16) or not at all. Full evidence is
in the audit §2; document drift in older prose does not override the
code table there.

## Roster (21 purchased concepts, 3 drill summons, 3 new Factions)

Text sources T01–T12; images V01–V10 are visual reference only.
`M.` = Machines, `Pl.` = Plants.

| ID | Text name | Proposed Faction / Archetype | Ability | Cat. |
|---|---|---|---|---|
| R01 | Bastion | M. / Tank | Muralha Viva (anchored self-reduction window) | B |
| R02 | Demolidor | M. / Tank | Esmagamento em Área (caster-centered unit-only pulse) | B |
| R03 | Crowcaller | Undead / Ranged | Marca do Corvo (mark + heal-cut + own bonus) | B |
| R04 | Gatilho | Humans / Ranged | Tiro Cruzado (fourth-shot secondary) | B |
| R05 | Matraca & Tico | Humans / Tank | Bagunça em Dobro (independent timer bomb + Slow) | B |
| R06 | Capivardo | Pl. / Tank | Barrigada do Brejo (caster-centered + Slow, A3 window) | B |
| R07 | Tridente | Pl. / Tank | Rasgo Sangrento (area + one bleed slot) | B |
| R08 | Grumak | Monsters / Tank | Couraça Primordial (damage-reactive capped reduction) | B |
| R09 | Rasgarr | Monsters / Dps | Frenesi Predatório (same-target ramp, bounded) | B |
| R10 | Esporax | Monsters / Ranged | Cuspe Corrosivo (third-shot flat secondaries) | B |
| R11 | Murmúrio | Monsters / Control | Pulsação do Medo (ordinary Slow, no displacement) | B |
| R12 | Sentinela Solar | Celestials / Tank | Barreira Solar (ally Unit shield; dep. on new resource) | C |
| R13 | Oráculo Luminar | Celestials / Control | Pulso Harmônico (basic-damage weakening slot) | B |
| R14 | Arqueira Celeste | Celestials / Ranged | Flecha da Purificação (strip Unit shield, empowered hit) | C |
| R15 | Justiçar | Celestials / Control | Chuva Celeste (single area pulse, B3 Slow) | B |
| R16 | Mandíbulo | Insects / Tank | Carapaça Erguida (timed self-reduction + B retaliation) | B |
| R17 | Louva | Insects / Dps | Golpe Preciso (fourth same-target bonus) | B |
| R18 | Vespira | Insects / Ranged | Ferrão Venenoso (third-shot one-slot poison) | B |
| R19 | Fiandeira | Insects / Control | Teia Adesiva (damage + Root, B3 queued Slow) | B |
| R20 | Crematumba | Undead / Tank | Fogo Profano (forward friendly-fire flame + cut) | B |
| R21 | Brocatanque | M. / Ranged | Ninho de Brocas (capped drills + deterministic fusion) | C |

Result: **0 A, 18 B, 3 C** against audited main (Faction infrastructure
is a shared dependency, not a fourth C). Three drill forms (Microbroca,
Duobroca, Megabroca) are summons, never Shop purchases or Evolutions.
New Factions: Monsters (damage +5/+10%, completion hypothesis), Celestials
(max HP +5/+10%, T09), Insects (movement +5/+10% within the +50% cap,
T10); all use the existing 2/4 living-conventional thresholds. Full
sheets, art/animation briefs and numbers live in the catalog and
GAME-DESIGN §11; they are not duplicated here.

## Proposed architecture (future implementation only)

- Faction append after Humans/Undead/Machines/Plants in one coordinated
  contract change (`Faction`, Synergy, Ruleset, views, lab, client
  icons/selection, serialization fixtures); one derived count from
  `FACTIONS`; never touch four-option Shop/Augment arrays.
- Three focused modules only if needed: `roster_actions.rs`
  (timers/counters), `roster_marks.rs` (marks/cuts), `drill.rs` (fusion
  lifecycle). Split further only for cohesion; no generic ability
  framework, physics, stealth, flight, armor or aura systems.
- Reuse reviewed PR #16 semantics once integrated: UnitFx shield
  (R12/R14), poison/cleanse/ward ordering (R18), friendly-damage reward
  path (R20), summon-family separation (R21). Do not reimplement them in
  parallel.
- One appended ability-start/resolution cue plus shield/fusion cues for
  reconnect completeness; no protocol version reserved now.

## Dependencies

1. PR #16 reviewed, integrated and re-audited on a new main SHA
   (mandatory gate; if it does not integrate, stop and revise this plan).
2. P54/P55 art-kit integration for final kit dispatch (offline combat
   work does not wait for it).
3. Owner approvals: Bastion base completion, R20/R21 completion values,
   Monsters damage Synergy numbers, R12/R14 shield-vs-simplification
   choice, R21 no-fusion fallback, 0.6/N Shop-gate floor, any threshold or
   economy change.
4. No new dependency, migration or deploy in this phase.

## Future execution waves (require authorization; each wave is test-gated)

- W0 — Shared contracts: heal-cut/weakening/poison/bleed/mark slots,
  timed self-reduction cap accounting, counted/cadence helpers, queued
  post-Root Slow, per-caster mark state, view/event extensions planned.
- W1 — Faction contract change (seven Factions, Synergy tables, views,
  lab parsing, fixtures, protocol bump with old-peer rejection).
- W2 — Timer/control core: R11, R13, R19, R16, R01, R06 (reuse Slow/Root/
  reduction primitives; `dr-cap`, `web-fear`, `bastion-wall` cases).
- W3 — Counted strikers: R04, R10, R17, R09, R03, R18 (counter discipline,
  `secondary-guard`, `dot-mix`, `brew-new-dots` cases).
- W4 — Area pulses: R02, R05, R07, R15, R20 (unit-only enumeration,
  friendly-fire reward rules, `pulse-summons`, `crema-friendly`,
  `friendly-plates` cases).
- W5 — Shield pair R12/R14 on the integrated resource (replace/refresh/
  weaker-ignored, strip-before-damage, Stronghold exclusion,
  `purification-boundary`, `celestial-loop` cases).
- W6 — Fusion line R21 (ownership/TTL, eight-cap, pair ordering, damaged
  merge, inheritance, no recursive summon; `summon-families`,
  `merge-afflictions` cases).
- W7 — Evolution trees for all 21 (stored tier 1/2, accumulated
  base-relative deltas, absolute ability replacement, snapshot at summon
  birth for drills).
- W8 — Low-poly models/materials per catalog (shared vertex-color
  material, cached form meshes, team readability, no new lights/shaders).
- W9 — Individual animation profiles per catalog (pose samplers, 0.30 s
  windup / 0.50 s strike / 0.60 s special timing, Low degradations).
- W10 — Deterministic tests, lab matrices (duel/support/Tank/summon/
  control/Evolution/Arcanas/performance per sheet) and gate confirmation.
- W11 — Controlled release (protocol bump, matching client/server,
  replay-format note, capacity/frame budgets, owner fun/look sign-off).

Rollback per wave: keep the wave behind a contract flag off by default;
a failing gate reverts the wave commit, never the thresholds. No wave
starts while its dependency gate is red.

## Future test matrix (specified now, implemented later)

Per unit: duel vs cost-equivalent, synergy composition, counter
composition, support pairing, Tank pairing, summon interaction, control
interaction, Evolution buy-vs-base ablation, Arcanas interaction,
performance under hundreds of units and maximum procs
(`reconnect-effects` everywhere). Cross-cutting: `crow-control`,
`cut-overlap`, `same-target-melee`, `control-chain`, `role-eligibility`,
`seven-factions`, `justicar-clump`. Lab uses 256 paired screening seeds
and 2048 disjoint confirmation pairs, Energy-equal duel budgets
20/40/60, reaction intervals 1/3/5 views; incomplete games are censored
at the lab limit, never scored as draws. Thresholds need owner approval;
never tune gates to pass.

## Acceptance criteria (this planning phase)

1. All 12 text sources and 9 PNGs inventoried with SHA-256 prefixes.
2. Every image bound to the correct text character; no V08 image-only
   label adopted as a unit (see audit reconciliation).
3. 21 purchased concepts modeled; 3 drill summons never counted as
   purchases; no invented 22nd character.
4. Every sheet has identity, base stats, ability spec, two Evolution
   paths, art brief and animation plan.
5. Every ability classified A/B/C with the smallest viable adaptation;
   structural alternatives carry a simpler fallback and an owner decision.
6. No new physics, movement, collision, economy, Shop, Augment or Arcana
   system proposed.
7. Faction proposal appends enums with thresholds, caps and wire/replay
   consequences stated.
8. Matrices cover integration, interactions and exploitation scenarios.
9. Performance section names scans, caps, view/event costs and required
   before/after workloads.
10. No code, asset, animation, protocol, balance or migration changed.
11. Links resolve; phase/catalog/audit/GAME-DESIGN §11 agree on names,
    counts and categories.
12. Roadmap updated; report records real command output.

## Risks

- Balance: shield/heal stall loops (R01/R12/R13, celestial-loop),
  anti-heal density (R03/R20), summon-group value (R02/R05/R06/R15),
  same-target dominance (R09 vs R17), fusion scaling (R21), Shop dilution
  at 31/46 kinds. All are Proposed hypotheses with lab cases, never
  proven balance.
- Technical: R12/R14 need the Unit shield resource; R21 needs the fusion
  lifecycle; Crematumba is O(n) per 10-tick pulse and its online wave
  needs benchmarks before authorization.
- Product: friendly fire (R20) may read as griefing despite being the
  text identity; Murmúrio/Justiçar/Oráculo sheets look weak in isolation
  and must prove a compositional niche or return to the owner.

## Pending owner decisions

Base completion for Bastion; cost/HP/movement for Crematumba;
production/radius completions for Brocatanque; Monsters damage Synergy
values; shield-vs-simplification for R12/R14; no-fusion fallback for
R21; Shop-gate floor (proposed 0.6/N per kind); any threshold, economy
or release approval. Structural simplifications that would erase an
identity are recommendations, not approvals.

## Verification (this phase; docs only)

```bash
cargo fmt --all -- --check
cargo clippy --workspace -- -D warnings
cargo check -p tbg-client -p tbg-core --target wasm32-unknown-unknown
git diff --check
git status --short
```

`cargo test --workspace` was also run for regression context; the
codebase is untouched so results mirror the base tree except for the
sandbox account-socket limitation recorded in the report. No browser,
device or VPS check is claimed.

## Reviewer checklist

- [ ] Branch is from `origin/main` at `6ef155b`; no code/asset/config
  change in the diff.
- [ ] 21 sheets present; Monsters are Grumak, Rasgarr, Esporax, Murmúrio;
  texts govern, images are reference only.
- [ ] 0 A / 18 B / 3 C classification and shared dependencies match the
  audit matrix.
- [ ] GAME-DESIGN §11 values are all marked Proposed/inactive; no live
  rule, pool or gate changed.
- [ ] Roadmap entry added; links resolve; English prose (proper names
  excepted).
- [ ] Phase report contains real command output, including the accounts
  sandbox failure.

## Phase report

Planning work on `phase/p56-roster-expansion-planning`, audited base
`6ef155bc9540527c99086aee0d9726908ca6a041`. Commit follows this report;
no PR yet and no push per the standing instruction.

### Commands and real results

Recorded 2026-10-30 in `/tmp/tbg-roster-expansion-planning`:

```text
cargo fmt --all -- --check
  exit 0
cargo clippy --workspace -- -D warnings
  Checking tbg-core, tbg-server, tbg-balance, tbg-client; Finished dev profile; exit 0
cargo check -p tbg-client -p tbg-core --target wasm32-unknown-unknown
  Finished dev profile; exit 0 (warnings: none; file lock contention only)
git diff --check
  exit 0
examples/army_load (release, pre-existing baseline, no roster code):
  10/side: step mean 3.9 us; 20: 9.2 us; 40: 25.0 us; 60: 47.0 us; 100: 110.2 us
  view bytes avg/max at 100/side: 2963 / 7532; 29.6 KB/s per player
cargo test --workspace (regression context; tree is docs-only):
  balance/contract/value/client suites pass (incl. c7 102.71 s pass);
  online/ranked/leaderboard/lab-equivalence pass;
  tbg-server accounts suite: 4 passed, 8 failed with
  `Os { code: 1, kind: PermissionDenied, message: "Operation not permitted" }`
  at tests/accounts.rs:46 (socket dir creation blocked in this sandbox).
  Same failure shape as the earlier P39 sandbox note; unrelated to this
  docs-only diff (no .rs file touched). Full logs: /tmp/tbg-p56-tests.log,
  /tmp/tbg-p56-tests-network.log, /tmp/tbg-p56-clippy.log,
  /tmp/tbg-p56-wasm.log, /tmp/tbg-p56-army-load.log.
```

### Deviations and limits

- Phase number P56 chosen because P41–P55 are reserved or carry
  collisions (audit §2.1); no existing `docs/phases/P56*` file existed.
- `Murmúrio` keeps its source spelling as a proper name; all other
  prose is ASCII per the English-only rule.
- No browser, phone, VPS or human fun/balance evidence is claimed;
  balance numbers are hypotheses awaiting the W10 lab and owner approval.

### Follow-up

Reviewer: re-run the verification block, check the reviewer list, then
commit and open the docs-only PR titled `docs: plan next roster
expansion phase` with the mandatory
`Planning only — no gameplay code, assets, animations or production
behavior were changed.` declaration. Do not merge, push, or implement.
