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

## Verify

```bash
./scripts/verify.sh
```

## Unity

See [unity/README.md](../unity/README.md) — menu **TBG → Character Studio**.
