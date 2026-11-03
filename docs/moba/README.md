# TBG Arena 5v5 — MOBA mode

A separate 5v5 game mode of Tupi Battle Grounds: ten strategic Seats with no player avatars, three
lanes, a jungle, automatic Minion waves, the **same champions as the 1v1
game** bought in the same Shop and sent to a Destination, and bots on every
empty seat. Status: **P77 implemented locally, including the owner's strategy-only
revision; focused compile checks recorded in the report. Runtime/captures,
performance and cross-platform acceptance pending; unbalanced.**

![1v1 champions fighting on a lane, 1v1-style HUD](media/p76-champions-lane.webp)

Historical P76 image; P77 no-avatar HUD/map captures are pending.

| Read | For |
|---|---|
| [GAME-DESIGN-MOBA.md](GAME-DESIGN-MOBA.md) | Rules `M-xx`, glossary, Proposed numbers |
| [ARCHITECTURE-MOBA.md](ARCHITECTURE-MOBA.md) | Crates, simulation, network, client, running |
| [CONTRACTS.md](CONTRACTS.md) | What must never change silently |
| [P76 phase spec and report](../phases/P76-moba-5v5-foundation.md) | What was built and validated |
| [P77 spec and handoff](../phases/P77-moba-roles-vision-jungle-map.md) | Three role Shop slots, team vision, jungle monsters, map refinement, strategy-only combat and left-drag camera |
| [Reference map](media/reference-map.webp) · [1v1 battle HUD](media/reference-1v1-battle-hud.webp) | Owner visual targets |

## Play on Linux

```bash
moba/scripts/install-linux-moba.sh     # builds and installs; menu entry "TBG Arena 5v5"
tbg-moba                               # menu → choose a role → "Treinar contra 9 bots"
tbg-moba --play --role mid             # straight into a practice match
tbg-moba-server & tbg-moba --online    # local room; more clients take bot seats
```

Controls: choose a Destination (**Z X C V** = Top/Mid/Bot/Jungle), then
click a Shop card (or **1–3**) to buy and send that champion. Choose another
Destination before the next purchase to split champions across lanes.
The Role limits Shop categories, never where a Troop can go. The card lock
keeps it through redraws; **R** rerolls and **U** upgrades future Troops.
Movement and combat are automatic; there is no player avatar or manual
unit order. **Left-drag empty map ground** pans the camera; left-click/drag
the minimap navigates it. Wheel zooms, arrows/edges/middle-drag pan,
**Space** or **H** on the minimap centres the Team base, **Esc** returns
to the menu. Right click does not issue orders.

## Develop

Use the work-front checks in [VALIDATION.md](../VALIDATION.md) and the
[P77 report](../phases/P77-moba-roles-vision-jungle-map.md#9-report).
Do not run a full workspace suite by default.

Local compilation follows [VALIDATION.md](../VALIDATION.md) (shared lock,
two build jobs). The workspace is separate from the 1v1 one on purpose; see
[ARCHITECTURE-MOBA §1](ARCHITECTURE-MOBA.md#1-isolation-from-the-1v1-game).
