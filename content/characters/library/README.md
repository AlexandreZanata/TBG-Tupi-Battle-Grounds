# Modular character library

> **Goal:** Produce a new game-ready character in **&lt;30 minutes** — no per-unit rigging or weight paint.

Assembly flow (web UI → Blender headless → export):

```text
Library → Body → Head → Parts → Materials → Export → Done
```

## Layout

```text
library/
├── _schema.base.json   # Base manifest JSON Schema
├── _schema.part.json   # Part manifest JSON Schema
├── sockets.json        # Socket id → TBG bone table
├── bases/              # Body templates (rig once)
│   ├── male/
│   ├── female/
│   ├── small/
│   └── large/
├── parts/              # Socket-attached interchangeable pieces
│   ├── heads/
│   ├── hair/
│   ├── hats/
│   ├── armor/
│   ├── weapons/
│   └── …
├── materials/          # Palette + faction tint profiles
├── rigs/               # One canonical armature per base (TBG_*)
└── exports/            # Assembled unit GLBs (game + showcase)
```

## Authoring rules

1. **Rig once per base** — MB-Lab generates humanoid + armature; promote to `rigs/{base_id}/`.
2. **Parts use sockets** — each part JSON declares `socket` bone name; no per-part skinning.
3. **Materials are swappable** — body keeps UV0; parts may share atlas or per-part albedo.
4. **Export is deterministic** — `assemble_character.py` merges selection → single skinned GLB.

## Manifest schemas

| Schema | Path |
|--------|------|
| Base body | [`_schema.base.json`](_schema.base.json) |
| Part | [`_schema.part.json`](_schema.part.json) |

### Base example

```json
{
  "id": "male",
  "label": "Male body",
  "heightM": 1.8,
  "bodyClass": "HumanoidLight",
  "mesh": "body.glb",
  "rig": "../../rigs/male/armature.glb"
}
```

### Part example

```json
{
  "id": "head_round_01",
  "category": "heads",
  "baseCompat": ["male", "female"],
  "socket": "TBG_Head",
  "mesh": "head_round_01.glb",
  "thumbnail": "head_round_01.webp"
}
```

## Socket bone table

| Socket id | Bone | Part categories |
|-----------|------|-----------------|
| `head` | `TBG_Head` | heads, hair, hats, noses, beards |
| `head_top` | `TBG_Head` | hats (crown offset) |
| `spine_chest` | `TBG_Spine1` | armor, backpacks, wings |
| `spine_pelvis` | `TBG_Spine` | tails |
| `hand_r` | `TBG_Hand.R` | weapons |
| `hand_l` | `TBG_Hand.L` | shields (future) |

See [`sockets.json`](sockets.json) for machine-readable definitions.

## Seed dev fixtures

```bash
./scripts/seed_library.sh
```

Generates blocky TBG-skinned bases + box part meshes for pipeline development. Replace with MB-Lab exports per [`mblab_bases.md`](../../tools/assembly_studio/blender/mblab_bases.md).

## Tools

| Tool | Path |
|------|------|
| Web assembler UI | `tools/character_studio/static/assembly-studio.html` |
| Blender merge/export | `tools/assembly_studio/blender/` |
| MB-Lab base authoring | `tools/assembly_studio/blender/mblab_bases.md` |
| Library contract tests | `tests/assembly_studio/test_library_contract.py` |

## Related docs

- [modular-character-assembler.md](../../../docs/design/modular-character-assembler.md)
- [ADR-007](../../../docs/adr/ADR-007-modular-character-assembler.md)
- Agent tasks: `.local/phases/05-modular-character-assembler/`
