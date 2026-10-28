# Roadmap — TBG 2.0 to a public multiplayer MVP

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

| Phase | File | Milestone at the end | Status | Reviewed |
|-------|------|----------------------|--------|----------|
| P0 | — | Lean repository, plan | done | yes |
| P1 | [P1-browser-network-spike.md](phases/P1-browser-network-spike.md) | Bevy runs in desktop + phone browsers; browser ↔ server WebSocket RTT shown | done (owner phone test pending) | yes |
| P2 | [P2-core-simulation.md](phases/P2-core-simulation.md) | All game rules headless, tested, bot vs bot | done | yes |
| P3 | [P3-skeleton-game.md](phases/P3-skeleton-game.md) | **Skeleton game:** full match vs bot in the browser with placeholder shapes | done (owner phone test pending) | yes |
| P4 | [P4-online-multiplayer.md](phases/P4-online-multiplayer.md) | **Online 1v1** with reconnect, latency/bandwidth metrics, bot load test validated locally | done (two-device test pending) | yes |
| P5 | [P5-real-environment.md](phases/P5-real-environment.md) | **Validated on the owner's VPS + domain** over HTTPS/WSS, real devices, measured capacity | code + local rehearsal done; **VPS validation pending** | yes (local) |
| P6 | [P6-visual-slice.md](phases/P6-visual-slice.md) | One polished 3D figurine + final arena + animation library, human-approved | todo | — |
| P7 | [P7-content-feel-balance.md](phases/P7-content-feel-balance.md) | 8 units with art, VFX, audio, balance targets met, playtest done | todo | — |
| P8 | [P8-public-launch.md](phases/P8-public-launch.md) | Public URL shared with first players | todo | — |

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

## After the MVP (not planned until P8 ships)

Accounts, `Inventory`, cosmetics store (BR-C01–C03) · ranking · units
9–12 · rematch and spectating · `Reposition`/bench design · WebGPU build ·
delta views · additional regions (ARCHITECTURE §12).
