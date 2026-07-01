# MB-Lab base body authoring

> MB-Lab is the recommended **free, open-source** base for TBG modular characters.

## Why MB-Lab

| Property | Value |
|----------|-------|
| License | Open source (check add-on repo for current license) |
| Runs in | Blender (no external service) |
| Outputs | Humanoid mesh + armature |
| Parameters | Height, weight, muscles, face, hands, legs |
| Cost | Free |

**Core rule:** Rig **once per base archetype** (`male`, `female`, `small`, `large`). Assembly swaps parts — never re-rig per character.

## Workflow (draft)

1. Install MB-Lab add-on in Blender (pin version in this doc when first authored)
2. Generate humanoid matching base archetype proportions
3. Rename bones to `TBG_*` per [RIG-SPEC.md](../../blender/RIG-SPEC.md)
4. Export showcase GLB → `content/characters/library/bases/{archetype}/body.glb`
5. Export armature → `content/characters/library/rigs/{archetype}/armature.glb`
6. Write `manifest.json`:

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

| Socket id | Bone | Categories |
|-----------|------|------------|
| `head` | `TBG_Head` | heads, hair, hats |
| `spine_chest` | `TBG_Spine2` | armor, backpacks |
| `hand_r` | `TBG_Hand_R` | weapons |
| `hand_l` | `TBG_Hand_L` | shields (future) |

## Status

- [ ] MB-Lab version pinned
- [ ] First `male` base exported
- [ ] Bone rename map validated against RIG-SPEC
