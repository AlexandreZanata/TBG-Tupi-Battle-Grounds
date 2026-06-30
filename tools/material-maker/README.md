# Material Maker — TBG texture pipeline

Procedural PBR maps for faction surfaces. Graph descriptors live in this folder; CI and headless export use `export_textures.py` (Pillow). Artists can recreate graphs in [Material Maker](https://github.com/RodZill4/material-maker) and export PNGs with **File → Export → Unity**.

## Graph descriptors (`.mm`)

| File | Faction | Outputs |
|------|---------|---------|
| `humans_fabric.mm` | Humans | albedo, normal, ORM |
| `undead_decay.mm` | Undead | albedo, normal, ORM, emissive mask |
| `machines_metal.mm` | Machines | albedo, normal, ORM |
| `plants_organic.mm` | Plants | albedo, normal, ORM |

TBG `.mm` files are JSON graph metadata (version, colors, weave/metal/organic parameters). Open in any editor; import parameters into Material Maker manually or use the Python exporter below.

## Export to `content/characters/textures/`

Naming: `{unit_id}_albedo.png`, `{unit_id}_normal.png`, `{unit_id}_orm.png`, optional `{unit_id}_emissive.png`.

```bash
# Single unit (faction inferred from graph or --faction)
python3 tools/material-maker/export_textures.py --id tank_human_01 --faction Humans

# All Phase 02 pilot units
python3 tools/material-maker/export_textures.py --all-pilots

# Default 512²; use --size 1024 for hero units
python3 tools/material-maker/export_textures.py --id tank_human_01 --faction Humans --size 1024
```

ORM packing (Unity / glTF convention): **R** = ambient occlusion, **G** = roughness, **B** = metallic.

## Material profiles

Shader parameters per faction: `content/characters/materials/*_pbr_v1.json`. Loaded in Unity by `MaterialProfileLoader` → `TBG/CharacterPBR`.

## Manual Material Maker workflow

1. Install Material Maker 1.3+.
2. Create a graph matching the faction descriptor (weave / decay / brushed metal / organic veins).
3. Add **Export** nodes for albedo, normal, roughness, metallic, AO — or use **Export → Unity**.
4. Merge roughness + metallic + AO into one ORM PNG (see packing above) if not using TBG export script.
5. Optionally paint albedo details in Krita (`tools/krita/templates/`).
6. Copy PNGs to `content/characters/textures/` with unit id prefix.

## Related

- [real-3d-character-pipeline.md](../../docs/design/real-3d-character-pipeline.md) §2.3
- [content/characters/README.md](../../content/characters/README.md)
