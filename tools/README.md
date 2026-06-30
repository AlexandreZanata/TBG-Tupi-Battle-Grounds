# Character Studio tools

Headless CLI for JSON CRUD, validation, and card PNG export. Unity Editor studio uses the same JSON contract.

## Setup

```bash
pip install -r tools/requirements.txt
```

## CLI

```bash
python3 -m tools.character_studio list
python3 -m tools.character_studio create <id> <Archetype> <Faction> --name "Display Name"
python3 -m tools.character_studio show <id>
python3 -m tools.character_studio delete <id>
python3 -m tools.character_studio export-card <id>
python3 -m tools.character_studio export-all-cards
```

## Web studio (browser)

```bash
./scripts/start-studio.sh
# Open http://127.0.0.1:8765
```

**3D preview** (Three.js): studio lighting, orbit + turntable, embedded GLB PBR with color tint, or external textures. Static meshes get procedural Idle/Attack/Death until rigged export.

**Showcase carousel:** http://127.0.0.1:8765/showcase.html — 2 sculpts per slide, click for modal 3D at +30% scale. GLB decode is cached in-browser per mesh URL.

### Share the carousel with other users

The carousel is a **live web server** (not a single HTML file). Showcase meshes total ~600 MB; viewers stream GLBs from your machine or server.

```bash
# Same Wi‑Fi / office LAN — share the printed http://<your-ip>:8765/showcase.html link
./scripts/share-showcase.sh

# Internet link (requires cloudflared or ngrok)
./scripts/share-showcase.sh tunnel

# Local only
./scripts/share-showcase.sh local
```

**Requirements for viewers:** modern browser (Chrome/Firefox/Safari), stable connection. First 3D load per character may take 10–30 s on slow links.

**Deploy on a VPS:** clone repo with `content/characters/meshes/showcase/`, run `./scripts/share-showcase.sh lan` behind nginx/Caddy with TLS, or use `tunnel` for quick demos.

**Import high-poly sculpts:** `tools/mesh-pipeline/optimize_glb.sh` (visual mode = same look, Draco only). Demo units: `demo_scout_01` … `demo_reaver_01`. Max mesh preview **150 MB**.

API: `/api/characters/{id}/mesh`, `/api/content/characters/...`

Create, edit, delete units and export cards from the browser. Uses the same JSON files under `content/characters/`.

## Verify

```bash
./scripts/verify.sh
```

## Material Maker (texture pipeline)

```bash
python3 tools/material-maker/export_textures.py --id tank_human_01 --faction Humans
python3 tools/material-maker/export_textures.py --all-pilots
```

See [material-maker/README.md](material-maker/README.md).

## Unity animator controllers

```bash
python3 tools/unity/generate_animator_controllers.py
```

Outputs shared body-class controllers to `content/characters/animations/`.

## Pilot mesh placeholders

```bash
python3 tools/unity/provision_pilot_meshes.py
```

Copies `_test_fixture.glb` to each pilot id in `pilot_roster.json` until authored meshes land.

## Unity

See [unity/README.md](../unity/README.md) — menu **TBG → Character Studio**.
