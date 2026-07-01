# MB-Lab Web Authoring

> **Goal:** Realistic human bases (not blocky placeholders) with MB-Lab-style controls in the browser.  
> **Engine:** [MB-Lab 1.8.1](https://github.com/animate1978/MB-Lab) inside Blender — not reimplemented in JavaScript.

## Why not model in the browser?

MB-Lab is a **Blender add-on** (~15k lines Python, morph engine, skin shaders, rig). The browser UI sends a **recipe JSON**; Blender headless applies morphs and exports GLB.

```text
Browser (MB-Lab Studio)
  → POST /api/assembly/mblab/generate
  → Blender + MB-Lab add-on
  → library/bases/{id}/body.glb OR library/exports/{unitId}.glb
  → Three.js preview
```

## MB-Lab status

| Fact | Impact |
|------|--------|
| Repo [archived July 2024](https://github.com/animate1978/MB-Lab) | Pin **1.8.1**; no new features upstream |
| Final release **1.8.1** | Blender **4.0+** |
| Successor **Charmorph** | Evaluate later; not in TBG yet |

## Phases

| Phase | Scope | Status |
|-------|--------|--------|
| **A** | Install MB-Lab; author 4 archetype `.blend` templates; export to `library/bases/` | Manual + scripts |
| **B** | MB-Lab Studio web UI + job queue + preview | This change |
| **C** | Full morph parity (face, hands, expressions) via `Humanoid` API | Future |

## Content paths

```text
content/characters/mblab/
  projects/          # male.blend, female.blend, … (MB-Lab sessions)
  recipes/           # saved web recipes (optional)
content/characters/library/bases/   # game-ready GLB bases
```

## Template authoring (Phase A)

1. `./scripts/vendor-mblab.sh`
2. `./scripts/install-mblab.sh`
3. Blender → MB-Lab → create humanoid per archetype (see `mblab_bases.md`)
3. **Do not finalize** until proportions match table; use **Base Skeleton** rig only
4. Save: `content/characters/mblab/projects/{male,female,small,large}.blend`
5. `./scripts/export-mblab-base.sh male` (repeat for each base)

## Live preview + gear + GLB export (MB-Lab Studio)

| Feature | Endpoint / path |
|---------|-----------------|
| Live morph (debounced) | `POST /api/assembly/mblab/generate` → `library/exports/mblab_live_preview.glb` |
| Preview base | `GET /api/assembly/bases/{id}/mesh` |
| Download body | `GET /api/assembly/exports/mblab_live_preview/mesh` |
| Assembled (body + parts) | `POST /api/assembly/export` |

### Professional modular pipeline

1. **Body** — MB-Lab Studio: morph realistic base (male/female/small/large).
2. **Parts** — `content/characters/library/parts/{category}/{id}.json` + `.glb` rigged to `TBG_*` socket bone.
3. **Preview** — browser attaches parts to bones (`hand_r` → `TBG_Hand.R`, etc.).
4. **Export** — Blender merges base + parts → `library/exports/{unitId}.glb`.

See `content/characters/library/sockets.json` and [modular-character-assembler.md](./modular-character-assembler.md).

## Web recipe schema

See `content/characters/library/_schema.mblab_recipe.json`.

## License

MB-Lab is AGPL/GPL — comply when distributing modified add-on code. Game meshes exported from your sessions are your art assets; verify [MB-Lab license](https://github.com/animate1978/MB-Lab/blob/master/license.txt) for your ship model.
