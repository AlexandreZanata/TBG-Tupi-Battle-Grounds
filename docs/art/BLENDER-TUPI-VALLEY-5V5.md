# Tupi Valley 5v5 Blender study

This study rebuilds the 5v5 MOBA map as an editable miniature. It follows
the same assembly rules as the [Verdant Forest study](BLENDER-FOREST-ARENA.md)
and the [modular Stronghold study](BLENDER-STRONGHOLDS.md), but the layout is
read from `moba/core-moba/src/map.rs`: a 200 × 200 valley, three lanes, the
diagonal river, two bridges, the central plaza, two base plateaus, 22 Towers
and eight jungle camps.

The shared 5v5 match camera is used for the review frame: 32° vertical
field of view, 56° pitch and 95 map units of distance. The second scene is a
component board for inspecting the new Tupi prototypes beside the reused
library pieces.

![Tupi Valley 5v5 from the match camera](../media/blender-tupi-valley-5v5/tupi-valley-match.png)

## Assembly

Every prototype is a separate collection marked as a Blender asset. Repeated
parts are collection instances; the scene keeps seven named layers:

- **Valley floor and river.** A 200 × 200 meadow, a diagonal water ribbon,
  sandy banks, ripples and the river bed.
- **Three lanes, bridges and plaza.** Lane beds, worn flagstones, kerbs,
  two stone bridge instances with rails and lanterns, and the plaza island
  with a separately editable crystal.
- **Base plateaus and Strongholds.** Grass courts, stone rims, stairs, Nexus
  plinths, the Classic Stronghold and its team banners.
- **Towers and team pennants.** Editable plinth, shaft, crown, finial and
  cloth objects in each Team's Tower collection, with separate crystal
  collections. Both are instanced at the 11 Blue and 11 Red positions.
- **Jungle camps.** Clearing, outcrops, ferns and a kind-tinted camp crystal
  for Wolves, Golems, Jaguar and Frogs.
- **Forest edge.** Pines, oaks and understory placed outside lanes, water,
  bases, Towers and camps with a fixed seed.
- **Shoreline understory and props.** Braziers, fences, supplies and the
  component-board studio.

The reusable library comes from
`docs/media/blender-strongholds/verdant-forest-strongholds.blend`:
Classic Stronghold, Blue and Red banners, Full/Sparse pine, Coastal oak, Fern,
Shrub, Wildflowers, Meadow grass, Mossy rocks, Tide rocks, Brazier,
Split-rail fence and Supplies. The new Tupi prototypes are the lane
flagstones/kerbs, bridge deck, plaza island, base plateau, Nexus plinth,
Tower per team, camp per kind and the Stronghold crest.

## Reproduce

Use Blender 4.5 LTS from the repository root:

```sh
blender -b -t 4 --python-exit-code 1 \
  --python scripts/blender_tupi_valley_5v5/build.py -- /tmp/tbg-tupi-5v5 \
  --runtime moba/client-moba/assets/valley
```

Add `--project-only` after the output directory to skip rendering.
`--runtime` optionally exports compact per-piece meshes with baked vertex
colors for the shared Bevy client. The command writes:

- `tupi-valley-5v5.blend` with `01  Tupi Valley 5v5` and
  `02  Tupi Valley component board`.
- `tupi-valley-overview.png` (square map overview).
- `tupi-valley-match.png` (16:9 match-framed review).
- `tupi-valley-kit.png` (16:9 component board).
- `metrics.json` with collection names, object counts, triangle counts and
  the match-camera values.

## Observations

Blender 4.5.15 rendered the assembled map, match camera and 24-item
component board. The two scenes, placements and mesh exports are recorded
in [metrics.json](../media/blender-tupi-valley-5v5/metrics.json).

Reopen and verify the saved assembly and baked assets:

```sh
blender -b -t 2 /tmp/tbg-tupi-5v5/tupi-valley-5v5.blend --python-exit-code 1 \
  --python scripts/blender_tupi_valley_5v5/verify.py -- /tmp/tbg-tupi-5v5 \
  moba/client-moba/assets/valley
```

![Tupi Valley 5v5 overview](../media/blender-tupi-valley-5v5/tupi-valley-overview.png)

![Separate Tupi Valley components](../media/blender-tupi-valley-5v5/tupi-valley-kit.png)

## Runtime scope

The match uses baked Blender bridge decks, the plaza and its crystal,
four Jungle Camp kits, the Nexus plinth, and Blue and Red Towers. The
runtime places them at shared map coordinates. Tower damage selects one
of three baked crowns using the Team's observed HP. The animated crystal
and structure lifetime continue to follow the existing snapshot-driven
presentation. The runtime still generates the terrain, lanes, forest,
base courts and animated water. The editable Blender file includes the
complete assembly for art review.

Troops and Minions use the shared 1v1 Blender `FigureLibrary` meshes,
rigs, clips, sockets and materials with the existing 3× map adapter.
