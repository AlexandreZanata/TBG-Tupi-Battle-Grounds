# Character content (versioned)

Unit definitions and generated assets for TBG. **Source of truth:** `definitions/*.json`.

The **Character Studio** (Unity Editor) creates, edits, and deletes files here. Do not hand-edit JSON without matching the schema in `_schema.example.json`.

## Layout

| Folder | Purpose | Committed |
|--------|---------|-----------|
| `definitions/` | Unit parameters (JSON) — studio CRUD | Yes |
| `cards/` | Card images rendered from procedural 3D | Yes |
| `previews/` | Optional viewport / test captures | Yes |

## Naming convention

```
{archetype}_{faction}_{nn}.json
```

Examples: `tank_human_01.json`, `dps_undead_01.json`

## Tools

```bash
pip install -r tools/requirements.txt
python3 -m tools.character_studio list
python3 -m tools.character_studio export-all-cards
./scripts/verify.sh
```

## Related docs

- [procedural-character-generator.md](../docs/design/procedural-character-generator.md)
- [character-creation-guide.md](../docs/design/character-creation-guide.md)
- Agent tasks: `.local/phases/01-procedural-character-studio/` (gitignored)
