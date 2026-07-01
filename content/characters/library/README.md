# Modular character library

> **Goal:** Produce a new game-ready character in **&lt;30 minutes** — no per-unit rigging or weight paint.

Assembly flow (web UI → Blender headless → export):

```text
Library → Body → Head → Parts → Materials → Export → Done
```

## Layout

```text
library/
├── bases/              # MB-Lab authored body templates (rig once)
│   ├── male/
│   ├── female/
│   ├── small/
│   └── large/
├── parts/              # Socket-attached interchangeable pieces
│   ├── heads/
│   ├── noses/
│   ├── beards/
│   ├── hair/
│   ├── hats/
│   ├── backpacks/
│   ├── armor/
│   ├── weapons/
│   ├── wings/
│   └── tails/
├── materials/          # Palette + faction tint profiles
├── rigs/               # One canonical armature per base (TBG_*)
└── exports/            # Assembled unit GLBs (game + showcase)
```

## Authoring rules

1. **Rig once per base** — MB-Lab generates humanoid + armature inside Blender; promote to `rigs/{base_id}/`.
2. **Parts use sockets** — each part JSON declares `socket` bone name + local offset; no per-part skinning.
3. **Materials are swappable** — body keeps UV0; parts may share atlas or per-part albedo.
4. **Export is deterministic** — `assemble_character.py` merges selection → single skinned GLB.

## Part manifest schema (draft)

```json
{
  "id": "head_round_01",
  "category": "heads",
  "baseCompat": ["male", "female"],
  "socket": "TBG_Head",
  "mesh": "parts/heads/head_round_01.glb",
  "thumbnail": "parts/heads/head_round_01.webp"
}
```

## Tools

| Tool | Path |
|------|------|
| Web assembler UI | `tools/assembly_studio/static/assembly-studio.html` |
| Blender merge/export | `tools/assembly_studio/blender/` (Phase 05) |
| MB-Lab base authoring | `tools/assembly_studio/blender/mblab_bases.md` |

## Related docs

- [modular-character-assembler.md](../../../docs/design/modular-character-assembler.md)
- [ADR-007](../../../docs/adr/ADR-007-modular-character-assembler.md)
- Agent tasks: `.local/phases/05-modular-character-assembler/`
