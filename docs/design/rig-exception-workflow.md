# Rig Exception Workflow

> Manual rig path only. Default: auto `TBG_*` pipeline via `import_game_ready_unit.py`.  
> **Related:** [biped-game-ready-pipeline](biped-game-ready-pipeline.md) (Phase 04b), [RIG-SPEC.md](../../tools/blender/RIG-SPEC.md)

---

## When to set `rigException: true`

Set in `content/characters/definitions/{id}.json` when **any** of:

- Not biped (quadruped, serpent, floating blob)
- Extra limbs (four arms, no legs)
- Skeleton cannot fit `TBG_*` T-pose hull (extreme hunch >45°, detached head)
- `bodyClass: "Organic"` with `TBG_Tail` / wings driving gameplay
- `bodyClass: "Mechanical"` with hinge joints not approximated by humanoid chain

**Do not** set for orcs, demons, or human-proportion robots — use auto pipeline with wider bone envelopes or Mixamo fallback.

---

## Manual checklist (Blender)

1. Import showcase GLB into `tools/blender/templates/{bodyClass}_template.blend`
2. Confirm [RIG-SPEC.md](../../tools/blender/RIG-SPEC.md) deformation bone names
3. Add optional bones (`TBG_Tail`, `TBG_Wing.L`) as children of `TBG_Spine1` only if needed
4. Weight paint: shoulders, elbows, knees, hips, cape/cloth
5. Run `validate_game_glb.py --check-weights` (or `TBG_WEIGHT_GATE=1`)
6. Bake or assign custom clips if standard retarget deforms badly
7. Document reason in definition JSON:

```json
{
  "rigProfile": "tbg_biped_auto",
  "rigException": true,
  "rigExceptionReason": "plant_trunk_lower_body — spine chain extended",
  "bodyClass": "Organic"
}
```

The import CLI blocks auto-rig when `rigException: true` and prints this workflow path.

---

## Approval

Exception units require **one** Animation Studio review recording before merging the game GLB. Store capture metadata under `.local/phases/04b-biped-game-ready-pipeline/artifacts/{id}/` (local only).

---

## Seed roster status (Phase 04b)

| Scope | Status |
|-------|--------|
| `fixed_test_imports.json` (25 demo pilots) | **No organic exception candidate** — all are standard Human bipeds on `HumanoidLight` / `HumanoidHeavy` |
| Future Phase 06 roster | Nominated first organic exception: **`dps_undead_01`** (`bodyClass: Organic`, Undead faction) when mesh is authored |

Until an exception unit is rigged, no manual session artifacts are required.

---

**Updated:** 2026-06-30
