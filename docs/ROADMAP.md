# Roadmap — TBG 2.0

> The index of all phases. Each phase has its own executable file in
> [`docs/phases/`](phases/) written for an **executor agent** (a cheaper
> model such as DeepSeek) that follows instructions literally. A
> **reviewer** (a strong model such as Claude or Codex) checks every phase
> before the next one starts.

## Order of work

The game must be **playable and online-validated with placeholder visuals
before any 3D character work starts.** Risky foundations first; art last.

```
P0 Simplify ✅
 ├─► P1 Browser + network spike ─┐
 └─► P2 Core simulation ─────────┴─► P3 Skeleton game (local, vs bot, placeholder shapes)
                                       └─► P4 Online 1v1 + latency metrics + local bot validation
                                             └─► P5 Real-environment validation (your VPS + domain)
                                                   └─► P6 Visual slice (3D figurines, arena, animation)
                                                         └─► P7 Content, feel, balance
                                                               └─► P8 Public MVP launch
```

P1 and P2 touch different crates and may run at the same time on two
branches. Every other phase starts only after the previous one is reviewed.

**After the MVP — two suites, strictly in this order.** The competitive
online suite must be validated on a real server (P14 gate) before any of
the visual suite starts. Inside the visual suite, the style lock (P15)
gates everything; each character gets its own validated phase.

```
Competitive online                     Medieval visual overhaul (after P14 sign-off)
P9  Real-time fast path + reconnect    P15 Art direction, rig v2, style lock (Archer)
 └► P10 Server efficiency               └► P16-A Desktop lobby + arena ─► P16-B Open army ─► P16-C Balance lab ─► P16 Class animations
     └► P11 Accounts (name + key)           └► P22–P29 One phase per character and its Evolutions
         └► P12 Ranked + Elo rating             └► P30 Fluidity + performance (client)
             └► P13 Global ranking + stats          └► P31 Consistency, playtest, v0.2.0
                 └► P14 Real-env validation (GATE)
                     └► P30-B Server capacity on minimal hardware (tbg-server only;
                          may run between character phases)

Archived 2026-10-28 by the owner (D-31): P17 arena for every ratio, P18
Stronghold styles, P19 UI kit + cards, P20 lobby restyle, P21 effects kit.
Their files stay in docs/phases/ for reference only. Final-quality phases
for Nester and Rampart are written after P16-B is reviewed.
```

**Strategic expansion — after P16-D (Proposed, D-34).** P16-D now has a
written plan: an offline universal pre-balance value model (PVE) that
measures marginal attributes, abilities, Evolutions, economy, Strongholds,
Augments, and Arcanas before full matches, and resolves the P16-C findings
(`saver`, heavy spam, Evolution value, faction spread). It is offline only
and changes no production rule by itself. P16-E is not used. P16-F turns
the game into a deeper strategy game
(GAME-DESIGN §10): uncapped Energy with Interest, Siege instead of a
time-up decision, Shop Reserve, four mechanical Stronghold kinds with
Fortifications, and Augment rounds. Every mechanic enters the `Ruleset`
disabled, passes a lab gate (§10.9), and is switched on online only by the
owner in the phase that integrates it. Each phase gates the next.
Writing these specs approves no value and enables nothing.

```
P16-C reviewed ─► P16-D Pre-balance value model (engine implemented; must fix or explain `saver`)
 └► P16-F1 Uncapped Energy, Interest (Siege/Collapse dropped, D-41)
     └► P16-F2 Shop Reserve                               (core+lab+HUD; switch)
         └► P16-F3 Stronghold kinds + Fortifications      (core+lab, offline)
             └► P16-F4 Kinds online: selection, store, lobby, HUD, 3D (switch)
                 └► P16-F5 Augment rounds + first 14      (core+lab, offline)
                     └► P16-F6 Full catalog of 40, adversarial balance (offline)
                         └► P16-F7 Augments online, War Council UI (switch)
                             └► P16-F8 Validation, load, playtest, release decision
```

Whether the visual track (P16, P22–P31) waits for P16-F8 or interleaves
after P16-F4 is owner decision O-1 (D-34); two agents never work on the
same files at once. Arcanas (P32–P38) stay after P31 unchanged; P32's
baseline then includes every switched-on P16-F mechanic, and Arcana
bonuses join the same additive formula (BR-X01).

**Optional gameplay extra — only after every existing phase (P0–P31).**
Arcanas extend the released game; they are not a prerequisite for P13,
P14, the visual suite, or `v0.2.0`. P32 starts only after P31 is reviewed
and its owner checkpoints are satisfied. Each extra phase gates the next.
Writing these specs did not authorize implementation; **D-40 (owner,
2026-10-29) did**: Arcanas are integrated and live in every mode with
Proposed values. **D-42/P40** then expanded the live system to three
five-node families, twelve points and supremes. P32–P38 stay as the
checklist for what is still open (full P34 matrix, P38 human/security
gates); P40 records the tree, migration, editor and lab work.

```
P31 reviewed + owner sign-off
 └► P32 Combat contracts, risk audit, refreshed baseline
     └► P33 Deterministic Arcanas (offline experiment)
         └► P34 Simulation, adversarial search, balance gate
             └► P35 Authoritative match integration and versioned records
                 └► P36 Arcanas editor, practice, desktop and phone UI
                     └► P37 Account presets and ranked audit integration
                         └► P38 Human playtest, performance, release gate
```

**Lobby extras — independent (P39).** Quick Challenge by link and rematch
reuse guest sessions, private-room pairing, the match task, reconnect, and
the result screen. They add no queue and no second server. P39 may run
after P14; do not run it concurrently with P16-F4/F7 or P16-A/P16-B (same
lobby/menu files). It gates no existing phase.

| Phase | File | Milestone at the end | Status | Reviewed |
|-------|------|----------------------|--------|----------|
| P0 | — | Lean repository, plan | done | yes |
| P1 | [P1-browser-network-spike.md](phases/P1-browser-network-spike.md) | Bevy runs in desktop + phone browsers; browser ↔ server WebSocket RTT shown | done (owner phone test pending) | yes |
| P2 | [P2-core-simulation.md](phases/P2-core-simulation.md) | All game rules headless, tested, bot vs bot | done | yes |
| P3 | [P3-skeleton-game.md](phases/P3-skeleton-game.md) | **Skeleton game:** full match vs bot in the browser with placeholder shapes | done (owner phone test pending) | yes |
| P4 | [P4-online-multiplayer.md](phases/P4-online-multiplayer.md) | **Online 1v1** with reconnect, latency/bandwidth metrics, bot load test validated locally | done (two-device test pending) | yes |
| P5 | [P5-real-environment.md](phases/P5-real-environment.md) | **Validated on the owner's VPS + domain** over HTTPS/WSS, real devices, measured capacity | code + local rehearsal done; **VPS validation pending** | yes (local) |
| P6 | [P6-visual-slice.md](phases/P6-visual-slice.md) | One polished 3D figurine + final arena + animation library, human-approved | done (all 8 units); **owner look approval pending** | yes |
| P7 | [P7-content-feel-balance.md](phases/P7-content-feel-balance.md) | 8 units with art, VFX, audio, balance targets met, playtest done | done (bot balance); **human playtest pending** | yes |
| P8 | [P8-public-launch.md](phases/P8-public-launch.md) | Public URL shared with first players | audit + security review done; **LICENSE, VPS smoke, tag pending owner** | yes (local) |
| P9 | [P9-realtime-fast-path.md](phases/P9-realtime-fast-path.md) | Instant-feeling taps (ack views, `TCP_NODELAY`); auto-resume, reconnect countdown, hidden tabs stay connected | done (phone check in P14) | yes |
| P10 | [P10-server-efficiency.md](phases/P10-server-efficiency.md) | Measured, profiled server: most matches per core | done | yes |
| P11 | [P11-accounts.md](phases/P11-accounts.md) | Unique player name + account key; login on any device | done (owner: name list, two devices) | yes |
| P12 | [P12-ranked-and-rating.md](phases/P12-ranked-and-rating.md) | Ranked queue apart from casual; Elo from both ratings | done (two-device ranked in P14) | yes |
| P13 | [P13-global-ranking.md](phases/P13-global-ranking.md) | Global ranking + player stats in game and on `/ranking/` | done (phones + VPS in P14) | yes |
| P14 | [P14-competitive-validation.md](phases/P14-competitive-validation.md) | **Gate:** competitive game validated on a real server, data-safety drill, security review | local rehearsal + fixes done; **real-server run and owner sign-off pending** | yes (local) |
| P15 | [P15-art-direction-and-rig.md](phases/P15-art-direction-and-rig.md) | Medieval style locked; rig v2; Archer style-lock approved | done; merged with P16-A (owner approved Archer v2 and the desktop look); phone FPS pending | yes |
| P16-A | [P16A-desktop-lobby-and-arena.md](phases/P16A-desktop-lobby-and-arena.md) | Archer v2 default; desktop lobby and arena match the owner's references | done; desktop look owner-approved; physical-phone check pending | yes |
| P16-B | [P16B-open-army-and-new-units.md](phases/P16B-open-army-and-new-units.md) | No army cap; five Shop offers; Nester and Rampart; large-army balance and performance measured | merged and on the test VPS (owner, 2026-10-29) with placeholder art; **final art, client FPS, fun check, balance decision pending** | yes (code) |
| P16-C | [P16C-balance-lab.md](phases/P16C-balance-lab.md) | Millions of headless matches proven equal to 1×; ruleset as data; opponent pool, duels, Nash meta, search; reviewed balance proposal; lab contract tests | lab, contracts, 1× proof done; 8-number proposal applied (Proposed); **owner: mechanic for banked waves/spam, then playtest** | yes (self-review) |
| P16-D | [P16D-prebalance-value-model.md](phases/P16D-prebalance-value-model.md) | Offline PVE engine: one-sided full-match value, Energy shadow price, interaction canaries, held-out validation; P16-C findings still open | engine implemented and measured; **owner balance decision pending**; P16-F1 blocked by `saver` | — |
| P16-F1 | [P16F1-economy-and-siege.md](phases/P16F1-economy-and-siege.md) | Uncapped Energy, low-balance Interest, Siege/Collapse, no time-up decision; lab-gated switch; HUD | **owner revision D-41**: economy active, open-ended matches (no Siege/Collapse); lab gate not run; PR open from `phase/p16f-infinite-economy`, review pending | — |
| P16-F2 | [P16F2-shop-reserve.md](phases/P16F2-shop-reserve.md) | Hold one Shop offer for 20 s; lab comparison; HUD | planned | — |
| P16-F3 | [P16F3-stronghold-kinds-core.md](phases/P16F3-stronghold-kinds-core.md) | Per-seat Stronghold, four kinds, Bounty, Fortifications; lab matrices (offline) | **superseded** by the owner's 2026-10-29 spec (D-46), shipped with P16-F4 on `phase/p17-strongholds` | — |
| P16-F4 | [P16F4-stronghold-kinds-online.md](phases/P16F4-stronghold-kinds-online.md) | Selection frozen at queue, persistence, lobby, Fortify panel, four 3D kinds | **done** as D-46 (Treasury, Bastion, Artillery, Command; kind-exclusive branches): core, server, lab matrix, lobby picker, Fortify HUD, modular 3D | — |
| P16-F5 | [P16F5-augments-core.md](phases/P16F5-augments-core.md) | Augment rounds, mirrored draws, Forgo, the first 14 Augments; Δ value per Augment (offline) | **owner revision D-41**: infinite rounds with a battle pause, all 40 active; Δ table not measured; same PR | — |
| P16-F6 | [P16F6-augment-catalog.md](phases/P16F6-augment-catalog.md) | All 40 Augments, adversarial triples, tuned catalog (offline) | catalog implemented (D-41); adversarial balance open | — |
| P16-F7 | [P16F7-augments-online.md](phases/P16F7-augments-online.md) | War Council on desktop and phones, records, practice bot | **owner revision D-41**: online, modal War Council pause on desktop and phone; records open; same PR | — |
| P16-F8 | [P16F8-strategic-validation.md](phases/P16F8-strategic-validation.md) | Full gate, long matches, capacity, human playtest, release decision | planned; every gate open after D-41 | — |
| P16 | [P16-class-animation-sets.md](phases/P16-class-animation-sets.md) | Professional clip sets per Archetype, simulation-timed | planned | — |
| P17 | [P17-arena-and-background.md](phases/P17-arena-and-background.md) | Living medieval battlefield filling every screen ratio | **archived** (owner, 2026-10-28; D-31) | — |
| P18 | [P18-strongholds.md](phases/P18-strongholds.md) | Three selectable cosmetic Stronghold styles | **archived** (owner, 2026-10-28; D-31) | — |
| P19 | [P19-ui-kit-and-cards.md](phases/P19-ui-kit-and-cards.md) | Medieval UI kit, one standardized card, accessibility | **archived** (owner, 2026-10-28; D-31) | — |
| P20 | [P20-lobby-and-ranking-page.md](phases/P20-lobby-and-ranking-page.md) | Medieval lobby (casual/ranked sections) and ranking pages | **archived** (owner, 2026-10-28; D-31) | — |
| P21 | [P21-combat-vfx-and-sound.md](phases/P21-combat-vfx-and-sound.md) | Pooled effects, projectiles, statuses, sound set v2 | **archived** (owner, 2026-10-28; D-31) | — |
| P22 | [P22-unit-archer.md](phases/P22-unit-archer.md) | Archer final (reference character) | planned | — |
| P23 | [P23-unit-shieldbearer.md](phases/P23-unit-shieldbearer.md) | Shieldbearer final | planned | — |
| P24 | [P24-unit-ogre-archer.md](phases/P24-unit-ogre-archer.md) | Ogre Archer (`SiegeWalker`) final | planned | — |
| P25 | [P25-unit-bone-brute.md](phases/P25-unit-bone-brute.md) | Bone Brute final | planned | — |
| P26 | [P26-unit-hexer.md](phases/P26-unit-hexer.md) | Hexer final | planned | — |
| P27 | [P27-unit-sawknight.md](phases/P27-unit-sawknight.md) | Sawknight (`Sawbot`) final | planned | — |
| P28 | [P28-unit-bloomkeeper.md](phases/P28-unit-bloomkeeper.md) | Bloomkeeper final | planned | — |
| P29 | [P29-unit-rootcaller.md](phases/P29-unit-rootcaller.md) | Rootcaller final | planned | — |
| P30 | [P30-fluidity-and-performance.md](phases/P30-fluidity-and-performance.md) | Auto quality, no hitches, device matrix met | planned | — |
| P30-B | [P30B-server-capacity.md](phases/P30B-server-capacity.md) | Most players per minimal server; measured before/after on 1 vCPU / 1 GB | planned | — |
| P31 | [P31-consistency-and-release.md](phases/P31-consistency-and-release.md) | One visual family; playtest; tag `v0.2.0` | planned | — |
| P32 | [P32-arcana-baseline-and-combat-contracts.md](phases/P32-arcana-baseline-and-combat-contracts.md) | Refreshed baseline, reproduced combat risks, explicit rule decisions | planned (extra after P31) | — |
| P33 | [P33-arcana-deterministic-core.md](phases/P33-arcana-deterministic-core.md) | Six bounded options, deterministic modifiers, offline experiment only | planned | — |
| P34 | [P34-arcana-simulation-and-balance.md](phases/P34-arcana-simulation-and-balance.md) | Paired-seed matrix, bot-policy checks, adversarial search, reviewed tuning | planned | — |
| P35 | [P35-arcana-authoritative-integration.md](phases/P35-arcana-authoritative-integration.md) | Validated and frozen match presets, reconnect, versioned match records | planned | — |
| P36 | [P36-arcana-editor-and-practice.md](phases/P36-arcana-editor-and-practice.md) | Complete editor and accurate previews on desktop and phones | planned | — |
| P37 | [P37-arcana-presets-and-ranked-records.md](phases/P37-arcana-presets-and-ranked-records.md) | One durable preset per account, guest parity, atomic ranked metadata | planned | — |
| P38 | [P38-arcana-validation-and-release.md](phases/P38-arcana-validation-and-release.md) | Human, security, performance, and real-server gates; owner release decision | planned | — |
| P39 | [P39-quick-challenge-and-rematch.md](phases/P39-quick-challenge-and-rematch.md) | One-click link challenge; both-accept rematch; private rooms, queues, and reconnect preserved | implemented; 29 online tests and browser evidence; **owner physical-phone/PWA check pending** | PR #2 follow-up fixes pending |
| P40 | [P40-arcana-trees.md](phases/P40-arcana-trees.md) | Fifteen-node trees, twelve points, three supremes, preset migration, editor and lab evidence | implemented; audited on `phase/p40-review` (Quickdraw cap fixed, Deadeye re-measured to +0.25 tile); War and Vitality builds trail the default, browser look check open | — |

## Pending validations (owner)

All code for P1–P8 is implemented and validated on the development PC
(details in each phase report). What still needs the owner:

- [ ] **Phones** (P1, P3, P6, P7): open the game on an Android phone and an iPhone —
      renders, touch works, ≥ 30 FPS in a busy match, sound plays after the first tap.
- [ ] **Two real devices online** (P4): desktop + phone on the same Wi-Fi, a private
      room via link, phone Wi-Fi off 10 s (resumes) and 40 s (forfeit).
- [ ] **VPS + domain** (P5): provide IP, SSH sudo user, domain A record; run
      `bash deploy/deploy.sh --provision`, then the R1–R13 table (bots over `wss://`,
      capacity 100–500 matches, 4G player, restart, backups, firewall).
      A temporary-VM run covered R1–R6, R11, R13 and part of R7–R9 (P5 report,
      "Temporary environment run"); the production host still needs the full table.
- [ ] **Look approval** (P6): gallery and match screenshots on a real screen.
- [ ] **Playtest round** (P7): ≥ 4 people, ≥ 10 online matches; confirm or change D-14 values.
- [ ] **Launch** (P8): add `LICENSE` (MIT intended), uptime checker on `/health`,
      public smoke test, tag `v0.1.0`.

## How a phase is executed (executor agent)

1. Read `AGENTS.md`, then **only** the phase file and the doc sections it
   lists under "Read first".
2. Create a branch `phase/PN-short-name` from `main`.
3. Do the steps **in order**. Each step lists files, an exact spec, the
   check to run, and *Done when*. Do not start step N+1 before step N's
   check passes.
4. Commit after each step (`feat(pN): <step title>`). Never push.
5. Never invent rules, numbers, message types, or dependencies the phase
   file does not name. If the spec is ambiguous, choose the simplest
   reading and write it under **Deviations** in the phase report.
6. If a step fails twice with the same error, stop working on it, write a
   `> blocked:` note in the phase report (what you tried, the exact error,
   your guess), and continue only with steps that do not depend on it.
7. At the end, fill in the **Phase report** at the bottom of the phase
   file (commands run with real output, measurements, deviations,
   blocked items) and commit it. Then stop and hand over for review.

## How a phase is reviewed (reviewer agent)

1. Read the phase file including its report, then `git diff main...phase/PN-*`.
2. Re-run the phase's acceptance commands yourself; never trust the report alone.
3. Check the "Reviewer checklist" in the phase file.
4. Fix small problems directly; send large ones back with exact instructions.
5. Merge to `main`, mark the phase **Reviewed** in the table above, push
   when the owner has asked for it.

## Human checkpoints

| When | The owner does |
|------|----------------|
| End of P1 | Opens the page on an Android phone and an iPhone; confirms it runs |
| End of P3 | Plays a skeleton match vs bot on desktop and phone |
| End of P4 | Plays an online match against a second device on the LAN |
| Start of P5 | Provides VPS (IP, SSH user with sudo) and a domain/subdomain pointing to it |
| End of P5 | Plays online over the public domain with someone on another network |
| End of P6 | Approves or rejects the look (gallery screenshots) |
| End of P7 | Runs the playtest round and approves the balance values |
| P8 | Chooses and adds the LICENSE; announces the URL |
| P11 | Reviews the blocked-name list; creates an account on two devices |
| P14 | Provides the server; plays ranked with someone on another network; signs off the gate |
| P15 | [P15-art-direction-and-rig.md](phases/P15-art-direction-and-rig.md) | Medieval style locked; rig v2; Archer style-lock approved | done; merged with P16-A (owner approved Archer v2 and the desktop look); phone FPS pending | yes |
| P16-A | [P16A-desktop-lobby-and-arena.md](phases/P16A-desktop-lobby-and-arena.md) | Archer v2 default; desktop lobby and arena match the owner's references | done; desktop look owner-approved; physical-phone check pending | yes |
| P16-B | Approves the new units' look and the Proposed balance values after the playtest |
| P16-C | Approves D-33, the balance proposal (or a mechanic change for heavy spam), then playtests the new values |
| P16-D | Approves the PVE methodology and the balance packet; resolves whether `saver` needs a rule change before P16-F1 |
| P16 | Approves class animations |
| P16-F1 | Siege/Collapse (O-4) and uncapped Energy with Interest (O-5) from the lab table; switches canon |
| P16-F2 | Switches Shop Reserve on |
| P16-F3 | Kind names (O-2), Fortification values (O-9), Banner aura and Ballista variant (O-12) |
| P16-F4 | Default kind (O-3); approves the four Stronghold looks, lobby and Fortify panel; switches canon |
| P16-F5 | Identical options (O-6), three rounds (O-7), public Augments (O-10), tuned values |
| P16-F6 | Approves the revised catalog (O-8) |
| P16-F7 | Approves the War Council on desktop and phones; switches Augments on |
| P16-F8 | Runs the playtest (≥ 8 people, ≥ 24 matches); balance and release decision |
| P22–P29 | Approves each character (screenshots + live match) |
| P31 | Runs the playtest; decides balance changes; pushes the tag |
| P32 | Resolves reproduced rule/canon conflicts and approves the experiment's scope |
| P34 | Reviews balance evidence; accepts or rejects the proposed tuning before online integration |
| P36 | Approves the Arcanas editor's look and touch interaction |
| P38 | Plays the extra, approves fun/fairness, and explicitly authorizes public release |
| P39 | Plays a challenge link on desktop and phone, accepts a rematch, and confirms private rooms still work |

## Ready-to-send brief (use for any phase)

> You are the executor agent for TBG 2.0. Read `AGENTS.md`, then
> `docs/phases/<PHASE FILE>`, and only the documents that file lists under
> "Read first". Work on branch `phase/<PN-name>` created from `main`.
> Follow the steps exactly and in order; run each step's check before
> moving on; commit after each step; never push. Do not add rules,
> numbers, message types, or dependencies that the phase file does not
> name. When finished or blocked, fill in the "Phase report" section of
> the phase file with the real command outputs and measurements, commit
> it, and stop.

## Owner-requested seascape follow-up (2026-10-30)

[Arena seascape](art/ARENA-SEASCAPE.md): implemented and validated locally on `codex/arena-seascape`, based on main `6ef155b`. Owner approved the displayed design and the 12.9 ms reference p95 limit (+15.2% over its measured baseline). Report includes before/after desktop captures, performance/build measurements, local commits and the separate pre-existing online-test exclusion. No push or PR is authorized for this task; broader P30 status is unchanged.

## Later (not planned yet)

`Inventory` and cosmetics store (BR-C01–C03) · units 9–12 · spectating ·
rating-based matchmaking and seasons (D-18) · `Reposition`/bench design ·
WebGPU build · delta views · additional regions (ARCHITECTURE §12).
