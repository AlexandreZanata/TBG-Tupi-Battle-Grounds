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
