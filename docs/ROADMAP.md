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
 └► P10 Server efficiency               └► P16 Animation sets per class (Archetype)
     └► P11 Accounts (name + key)           └► P17 Arena + backgrounds for every screen
         └► P12 Ranked + Elo rating             └► P18 Strongholds: 3 styles
             └► P13 Global ranking + stats          └► P19 UI kit + standardized cards
                 └► P14 Real-env validation (GATE)      └► P20 Lobby + ranking pages restyle
                                                            └► P21 Combat VFX + sound kit
                                                                └► P22–P29 One phase per character
                                                                    └► P30 Fluidity + performance
                                                                        └► P31 Consistency, playtest, v0.2.0
```

**Optional gameplay extra — only after every existing phase (P0–P31).**
Arcanas extend the released game; they are not a prerequisite for P13,
P14, the visual suite, or `v0.2.0`. P32 starts only after P31 is reviewed
and its owner checkpoints are satisfied. Each extra phase gates the next.
Writing these specs does not authorize implementation, approve proposed
balance values, or enable Arcanas on the public server.

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
| P15 | [P15-art-direction-and-rig.md](phases/P15-art-direction-and-rig.md) | Medieval style locked; rig v2; Archer style-lock approved | implemented + reviewed; **owner sign-off (look, names, D-22) and phone FPS pending** | yes (local) |
| P16 | [P16-class-animation-sets.md](phases/P16-class-animation-sets.md) | Professional clip sets per Archetype, simulation-timed | planned | — |
| P17 | [P17-arena-and-background.md](phases/P17-arena-and-background.md) | Living medieval battlefield filling every screen ratio | planned | — |
| P18 | [P18-strongholds.md](phases/P18-strongholds.md) | Three selectable cosmetic Stronghold styles | planned | — |
| P19 | [P19-ui-kit-and-cards.md](phases/P19-ui-kit-and-cards.md) | Medieval UI kit, one standardized card, accessibility | planned | — |
| P20 | [P20-lobby-and-ranking-page.md](phases/P20-lobby-and-ranking-page.md) | Medieval lobby (casual/ranked sections) and ranking pages | planned | — |
| P21 | [P21-combat-vfx-and-sound.md](phases/P21-combat-vfx-and-sound.md) | Pooled effects, projectiles, statuses, sound set v2 | planned | — |
| P22 | [P22-unit-archer.md](phases/P22-unit-archer.md) | Archer final (reference character) | planned | — |
| P23 | [P23-unit-shieldbearer.md](phases/P23-unit-shieldbearer.md) | Shieldbearer final | planned | — |
| P24 | [P24-unit-ogre-archer.md](phases/P24-unit-ogre-archer.md) | Ogre Archer (`SiegeWalker`) final | planned | — |
| P25 | [P25-unit-bone-brute.md](phases/P25-unit-bone-brute.md) | Bone Brute final | planned | — |
| P26 | [P26-unit-hexer.md](phases/P26-unit-hexer.md) | Hexer final | planned | — |
| P27 | [P27-unit-sawknight.md](phases/P27-unit-sawknight.md) | Sawknight (`Sawbot`) final | planned | — |
| P28 | [P28-unit-bloomkeeper.md](phases/P28-unit-bloomkeeper.md) | Bloomkeeper final | planned | — |
| P29 | [P29-unit-rootcaller.md](phases/P29-unit-rootcaller.md) | Rootcaller final | planned | — |
| P30 | [P30-fluidity-and-performance.md](phases/P30-fluidity-and-performance.md) | Auto quality, no hitches, device matrix met | planned | — |
| P31 | [P31-consistency-and-release.md](phases/P31-consistency-and-release.md) | One visual family; playtest; tag `v0.2.0` | planned | — |
| P32 | [P32-arcana-baseline-and-combat-contracts.md](phases/P32-arcana-baseline-and-combat-contracts.md) | Refreshed baseline, reproduced combat risks, explicit rule decisions | planned (extra after P31) | — |
| P33 | [P33-arcana-deterministic-core.md](phases/P33-arcana-deterministic-core.md) | Six bounded options, deterministic modifiers, offline experiment only | planned | — |
| P34 | [P34-arcana-simulation-and-balance.md](phases/P34-arcana-simulation-and-balance.md) | Paired-seed matrix, bot-policy checks, adversarial search, reviewed tuning | planned | — |
| P35 | [P35-arcana-authoritative-integration.md](phases/P35-arcana-authoritative-integration.md) | Validated and frozen match presets, reconnect, versioned match records | planned | — |
| P36 | [P36-arcana-editor-and-practice.md](phases/P36-arcana-editor-and-practice.md) | Complete editor and accurate previews on desktop and phones | planned | — |
| P37 | [P37-arcana-presets-and-ranked-records.md](phases/P37-arcana-presets-and-ranked-records.md) | One durable preset per account, guest parity, atomic ranked metadata | planned | — |
| P38 | [P38-arcana-validation-and-release.md](phases/P38-arcana-validation-and-release.md) | Human, security, performance, and real-server gates; owner release decision | planned | — |

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
| P15 | Approves the medieval look, display names, and the pipeline (D-22) |
| P16–P21 | Approves class animations, arena, Strongholds, UI kit, lobby, effects |
| P22–P29 | Approves each character (screenshots + live match) |
| P31 | Runs the playtest; decides balance changes; pushes the tag |
| P32 | Resolves reproduced rule/canon conflicts and approves the experiment's scope |
| P34 | Reviews balance evidence; accepts or rejects the proposed tuning before online integration |
| P36 | Approves the Arcanas editor's look and touch interaction |
| P38 | Plays the extra, approves fun/fairness, and explicitly authorizes public release |

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

## Later (not planned yet)

`Inventory` and cosmetics store (BR-C01–C03) · units 9–12 · rematch and
spectating · rating-based matchmaking and seasons (D-18) · `Reposition`/bench
design · WebGPU build · delta views · additional regions (ARCHITECTURE §12).
