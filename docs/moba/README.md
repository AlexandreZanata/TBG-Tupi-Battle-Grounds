# TBG Arena 5v5 — MOBA mode

A separate 5v5 game mode of Tupi Battle Grounds: ten Commanders, three
lanes, a jungle, Troops bought in the Shop and sent to a Destination, and
bots on every empty seat. Status: **P76 foundation — playable offline
against 9 bots and online on a local room server; unbalanced.**

![Lane close-up with the HUD](media/p76-lane-closeup.webp)

| Read | For |
|---|---|
| [GAME-DESIGN-MOBA.md](GAME-DESIGN-MOBA.md) | Rules `M-xx`, glossary, Proposed numbers |
| [ARCHITECTURE-MOBA.md](ARCHITECTURE-MOBA.md) | Crates, simulation, network, client, running |
| [CONTRACTS.md](CONTRACTS.md) | What must never change silently |
| [P76 phase spec and report](../phases/P76-moba-5v5-foundation.md) | What was built, validation, backlog for the next agent |

## Play on Linux

```bash
moba/scripts/install-linux-moba.sh     # builds and installs; menu entry "TBG Arena 5v5"
tbg-moba                               # menu → choose a role → "Treinar contra 9 bots"
tbg-moba --play --role mid             # straight into a practice match
tbg-moba-server & tbg-moba --online    # local room; more clients take bot seats
```

Controls: right click moves your Commander (then it holds there), **A**
returns it to automatic, **Space** centres the camera, arrows/screen edges/
middle drag pan, wheel zooms, **1–5** pick a Troop, **Z X C V** pick
Top/Mid/Bot/Jungle, **Enter** or the gold button buys and sends, **U**
upgrades, **Esc** returns to the menu. The minimap moves the camera (left)
or your Commander (right).

## Develop

```bash
cd moba
cargo fmt --all
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo test --locked --workspace
```

Local compilation follows [VALIDATION.md](../VALIDATION.md) (shared lock,
two build jobs). The workspace is separate from the 1v1 one on purpose; see
[ARCHITECTURE-MOBA §1](ARCHITECTURE-MOBA.md#1-isolation-from-the-1v1-game).
