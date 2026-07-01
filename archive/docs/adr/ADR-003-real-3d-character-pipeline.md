# ADR-003: Real 3D Character Pipeline and C# Character Library

**Status:** Accepted  
**Date:** 2026-06-30  
**Deciders:** Product / engineering (solo dev)

## Context

Phase 01 delivered a **procedural primitive** Character Studio (JSON + Python/Unity + web UI). That pipeline is ideal for rapid iteration, balance validation, and card export — but it has a visual ceiling.

TBG needs a path to **real 3D units**: authored meshes, PBR textures, rigged animation, and gameplay behaviors — without locking into closed commercial DCC tools we cannot customize.

Requirements:

- Open-source tools we can **fork, script, and extend**
- Versioned assets in the repository (or LFS where needed)
- A **first-party C# library** in Unity (not a black-box plugin)
- Same `content/characters/definitions/*.json` contract extended, not replaced
- Mobile-friendly output (low poly, compressed textures, shared rigs where possible)

## Decision

Adopt a **Real 3D Character Pipeline** built on:

| Layer | Choice | Role |
|-------|--------|------|
| **DCC (model + rig + animate)** | [Blender](https://www.blender.org/) (GPL — external tool) | Authoring; custom export add-on in repo |
| **Textures** | [Krita](https://krita.org/) + [Material Maker](https://github.com/RodZill4/material-maker) | Hand-painted + procedural PBR maps |
| **Interchange** | **glTF 2.0** (.glb) | Mesh, skeleton, animations, materials |
| **Unity import** | Unity glTFast (MIT) or Unity built-in glTF | Runtime / editor import |
| **C# library** | `TBG.CharacterLibrary` | Asset loading, materials, behaviors, spawn API |
| **Behaviors** | C# `BehaviorProfile` + Unity Animator | Data-driven; no visual scripting lock-in |
| **Studio** | Extend Character Studio (web + Unity) | Pick procedural vs mesh3d per unit |

Phase 01 procedural pipeline **remains** for prototyping. Production units use `assetMode: "mesh3d"`.

## Consequences

### Positive

- Professional visual target achievable per faction
- Blender Python add-ons and Material Maker graphs are version-controlled and modifiable
- glTF is industry-standard, tool-agnostic
- C# library owned entirely by TBG — no vendor dependency for core character logic

### Negative

- Higher art time per unit than procedural primitives
- Blender GPL: do **not** embed Blender code in the game; use only as offline authoring + export scripts
- Repository size grows (meshes/textures) — may need Git LFS above threshold
- Two pipelines to maintain until procedural is deprecated or limited to dev-only

### Non-goals (this ADR)

- In-engine modeling (keep authoring in Blender)
- Paywalled tools (Substance, Maya, 3ds Max) as required dependencies
- Per-unit unique skeleton (prefer **shared humanoid rig** per archetype family)

## Asset contract extension

```json
{
  "id": "tank_human_01",
  "assetMode": "mesh3d",
  "mesh": "content/characters/meshes/tank_human_01.glb",
  "textures": {
    "albedo": "content/characters/textures/tank_human_01_albedo.png",
    "normal": "content/characters/textures/tank_human_01_normal.png",
    "orm": "content/characters/textures/tank_human_01_orm.png"
  },
  "materialProfile": "humans_pbr_v1",
  "behaviorProfile": "tank_melee_v1",
  "animatorController": "content/characters/animations/TankHumanoid.controller"
}
```

Mechanics fields from Phase 01 (archetype, faction, proportions as **gameplay** scalars) stay in JSON.

## Alternatives considered

| Option | Rejected because |
|--------|------------------|
| Stay procedural only | Visual ceiling for market positioning |
| Commercial asset store packs | Inconsistent style; license friction |
| Godot as runtime | Project is Unity + Nakama; use Godot only as reference OSS architecture |
| FBX-only pipeline | glTF is more open, web-friendly, single-file .glb |
| Embed Blender in Unity | GPL contamination risk; unnecessary complexity |

## Follow-up

- Implement Phase 02 per [real-3d-character-pipeline.md](../design/real-3d-character-pipeline.md)
- Agent tasks: `.local/phases/02-real-3d-character-library/`
