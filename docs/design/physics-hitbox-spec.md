# Physics Hitbox Spec (Runtime)

> Bone-attached colliders for combat hit detection — **not** exported in GLB.  
> **Related:** [rig-exception-workflow.md](rig-exception-workflow.md), [RIG-SPEC.md](../../tools/blender/RIG-SPEC.md)

GLB exports carry the **deformation skeleton and skinned mesh only**. Unity attaches physics colliders at runtime from a hitbox profile.

---

## Design rules

1. Colliders follow **bones**, never mesh triangles.
2. One profile per body class (extend for `Organic` / `Mechanical` exceptions).
3. Weapon reach uses a child collider on the dominant hand bone.
4. Implementation deferred to **Phase 04 Unity client scaffold** — `IHitboxProfile` stub in `com.tbg.character-library`.

---

## Default biped profile (`HumanoidLight` / `HumanoidHeavy`)

| Bone | Collider | Purpose |
|------|----------|---------|
| `TBG_Head` | Sphere | Headshot / aim assist |
| `TBG_Spine1` | Capsule | Torso hits |
| `TBG_UpperArm.L` | Capsule | Left arm swing |
| `TBG_UpperArm.R` | Capsule | Right arm swing |
| `TBG_UpperLeg.L` | Capsule | Left leg hits |
| `TBG_UpperLeg.R` | Capsule | Right leg hits |
| `TBG_Hand.R` (child) | Box | Melee weapon reach (offset along +Z forward) |

Optional later: `TBG_Foot.L/R` capsules for low sweep attacks; `TBG_Spine` capsule for pelvis shots.

---

## C# contract (stub)

| Type | Package path |
|------|----------------|
| `IHitboxProfile` | `Runtime/Physics/IHitboxProfile.cs` |
| `BoneColliderDefinition` | `Runtime/Physics/BoneColliderDefinition.cs` |
| `DefaultBipedHitboxProfile` | `Runtime/Physics/DefaultBipedHitboxProfile.cs` |

Phase 04 will wire profiles into spawn (`Mesh3DCharacterFactory`) and lane combat raycasts.

---

**Updated:** 2026-06-30
