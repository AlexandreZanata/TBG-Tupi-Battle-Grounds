# TBG Arena 5v5 — MOBA mode

A separate 5v5 game mode of Tupi Battle Grounds: ten strategic Seats with no player avatars, three
lanes, a jungle, automatic Minion waves, the **same champions as the 1v1
game** bought in the same Shop and sent to a Destination, and bots on every
empty seat. Status: **P77 implemented locally, including the owner's strategy-only
revision and M-73–M-76 corrections. 45 focused tests passed; M-76 installed
Linux movement and decision traces are recorded in the P77 report. Complete roster, performance
and cross-platform acceptance pending; unbalanced.**

![1v1 champions fighting on a lane, 1v1-style HUD](media/p76-champions-lane.webp)

Historical P76 image; current P77 native motion evidence is linked from the phase
report. Every 5v5 Troop and Minion now renders the shared 1v1 Blender
champion meshes from the same `FigureLibrary` and `tbgmesh` assets.

| Read | For |
|---|---|
| [GAME-DESIGN-MOBA.md](GAME-DESIGN-MOBA.md) | Rules `M-xx`, glossary, Proposed numbers |
| [ARCHITECTURE-MOBA.md](ARCHITECTURE-MOBA.md) | Crates, simulation, network, client, running |
| [CONTRACTS.md](CONTRACTS.md) | What must never change silently |
| [P76 phase spec and report](../phases/P76-moba-5v5-foundation.md) | What was built and validated |
| [P77 spec and handoff](../phases/P77-moba-roles-vision-jungle-map.md) | Three role Shop slots, team vision, jungle monsters, map refinement, strategy-only combat and left-drag camera |
| [Reference map](media/reference-map.webp) · [1v1 battle HUD](media/reference-1v1-battle-hud.webp) | Owner visual targets |
| [Blender Tupi Valley 5v5 study](../art/BLENDER-TUPI-VALLEY-5V5.md) | Editable arena assembly, reusable parts and shared runtime export |

## Test from the 1v1 Practice panel

Open **Practice vs Bot**, then **Play against bots · 5v5 (test)**. Choose your
Role and start **Practice against 9 bots**. Four bots play on your Team and
five on the other Team. Use **Back to 1v1** from the 5v5 menu to return.
The 1v1 difficulty and Arena settings are for the 1v1 match. The 5v5 test
uses Tupi Valley and the current MOBA bot behavior.

- On **Linux and Windows**, build both clients and set `TBG_MOBA_CLIENT` to
  the absolute path of the companion executable before opening the 1v1
  client. The launcher also detects `tbg-moba` (Linux) or `tbg-moba.exe`
  (Windows) alongside the 1v1 executable.
- For the **desktop browser**, run `moba/scripts/stage-practice-web.sh`
  from the repository root. Serve the generated
  `.local/practice-5v5-web` directory at the server root and open
  `/?practice5v5=1`. For example, use
  `python3 -m http.server --bind 127.0.0.1 --directory .local/practice-5v5-web 8000`
  and open `http://127.0.0.1:8000/?practice5v5=1`.

For a Linux development build with the repository toolchain and native
dependencies installed:

```bash
flock -n -E 75 "/tmp/tbg-validation-$(id -u).lock" env CARGO_BUILD_JOBS=2 bash -eu -c '
  cargo build --locked -p tbg-client --bin tbg-client
  cargo build --locked --manifest-path moba/Cargo.toml -p tbg-moba-client --bin tbg-moba
'
TBG_MOBA_CLIENT="$PWD/moba/target/debug/tbg-moba" target/debug/tbg-client
```

## Play on Linux

```bash
moba/scripts/install-linux-moba.sh     # builds and installs; menu entry "TBG Arena 5v5"
tbg-moba                               # menu → choose a role → "Treinar contra 9 bots"
tbg-moba --play --role mid             # straight into a practice match
tbg-moba-server & tbg-moba --online    # local room; more clients take bot seats
```

Controls: choose a Destination (**Z X C V** = Top/Mid/Bot/Jungle), then
click a Shop card (or **1–5**) to buy and send that champion. Choose another
Destination before the next purchase to split champions across lanes.
Cards 1–3 draw from the Role categories; cards 4–5 draw from the entire
roster. All five are distinct. The Role never limits where a Troop can go. The card lock
keeps it through redraws; **R** rerolls and **U** upgrades future Troops.
Movement and combat are automatic; there is no player avatar or manual
unit order. **Left-drag empty map ground** pans the camera; left-click/drag
the minimap navigates it. Wheel zooms, arrows/middle-drag pan,
**Space** or **H** on the minimap centres the Team base, **Esc** returns
to the menu. Right click does not issue orders. A stationary pointer never pans at the window edge.
Troops keep their Lane/Jungle mission through combat and resume it afterward.
Engagements have explicit pursuit, windup, recovery and replanning phases;
support follows an eligible patient back into range.
Gold receipts distinguish personal bounties from Team Tower rewards.

Vision: the entire allied half is visible. Across the river, allied
Troops/Minions reveal only their sight circles after crossing. Destroying
an enemy Tower permanently reveals its former 9-unit sight circle until
the next match. Map and minimap share these authoritative rules.

## Develop

Use the work-front checks in [VALIDATION.md](../VALIDATION.md) and the
[P77 report](../phases/P77-moba-roles-vision-jungle-map.md#9-report).
Do not run a full workspace suite by default.

Local compilation follows [VALIDATION.md](../VALIDATION.md) (shared lock,
two build jobs). The workspace is separate from the 1v1 one on purpose; see
[ARCHITECTURE-MOBA §1](ARCHITECTURE-MOBA.md#1-isolation-from-the-1v1-game).
