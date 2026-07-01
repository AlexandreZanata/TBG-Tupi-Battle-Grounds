# MB-Lab base body authoring

> MB-Lab is the recommended **free, open-source** base for TBG modular characters.

## Version pin

| Tool | Version |
|------|---------|
| MB-Lab | **1.8.1** (final release) |
| Blender | **4.0.0+** |
| Rig mode | **Base Skeleton** only (no IK / muscle rigs) |

Download: vendored in-repo — `./scripts/vendor-mblab.sh` then `./scripts/install-mblab.sh`

Upstream: [MB-Lab tag 1_8_1](https://github.com/animate1978/MB-Lab/tree/1_8_1) (release **1.8.1**)

## Why MB-Lab

| Property | Value |
|----------|-------|
| License | Open source |
| Runs in | Blender (no external service) |
| Outputs | Humanoid mesh + armature |
| Parameters | Height, weight, muscles, face, hands, legs |
| Cost | Free |

**Core rule:** Rig **once per base archetype** (`male`, `female`, `small`, `large`). Assembly swaps parts — never re-rig per character.

## Export workflow

1. Create character in MB-Lab matching archetype proportions (see table below).
2. Finalize character → ensure feet at **Y = 0**, facing **+Z**.
3. Save template: `content/characters/mblab/projects/{baseId}.blend`
4. Export to library (CLI or browser):

```bash
./scripts/export-mblab-base.sh male
```

Or open **MB-Lab Studio**: `http://127.0.0.1:8765/mblab-studio.html`

Headless steps (rename + export) run inside `export_mblab_project.py`:

```bash
blender content/characters/mblab/projects/male.blend --background \
  --python tools/assembly_studio/blender/export_mblab_project.py -- \
  --base-id male --height-m 1.8 --body-class HumanoidLight --label "Male body"
```

Bone map: [`mblab_bone_map.json`](mblab_bone_map.json) — MB-Lab base skeleton → `TBG_*` per [RIG-SPEC.md](../../archive/tools/blender/RIG-SPEC.md).

5. Verify contract tests: `./scripts/verify.sh`

### CI / dev fixtures (no Blender)

Blocky TBG template seeds placeholder library content for pipeline development:

```bash
./scripts/seed_library.sh
```

Replace with MB-Lab exports when art-ready meshes are authored.

## Archetype targets

| Base id | Height (m) | Body class | Notes |
|---------|------------|------------|-------|
| `male` | 1.8 | HumanoidLight | Medium build |
| `female` | 1.65 | HumanoidLight | Slightly narrower |
| `small` | 1.2 | HumanoidLight | Goblin / child proportions |
| `large` | 2.1 | HumanoidHeavy | Tank / colossus |

## Manifest (`bases/{id}/manifest.json`)

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

## Sockets (attach points for parts)

Full table: [`content/characters/library/sockets.json`](../../../content/characters/library/sockets.json)

| Socket id | Bone | Categories |
|-----------|------|------------|
| `head` | `TBG_Head` | heads, hair, hats, noses, beards |
| `head_top` | `TBG_Head` | hats (optional crown offset) |
| `spine_chest` | `TBG_Spine1` | armor, backpacks, wings |
| `spine_pelvis` | `TBG_Spine` | tails |
| `hand_r` | `TBG_Hand.R` | weapons |
| `hand_l` | `TBG_Hand.L` | shields (future) |

## Status

- [x] MB-Lab version pinned (1.8.1)
- [x] Bone rename map validated against RIG-SPEC
- [x] Dev fixtures for all four bases (`seed_library.sh`)
- [ ] Production MB-Lab meshes replace blocky fixtures
