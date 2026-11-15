"""Assemble and render the Tupi Valley 5v5 Blender study.

The map geometry mirrors `moba/core-moba/src/map.rs`; repeated scenery comes
from the owner-approved Verdant Forest Blender library. Run from the repo root:

    blender -b -t 4 --python-exit-code 1 --python scripts/blender_tupi_valley_5v5/build.py -- /tmp/tbg-tupi-5v5
"""

import argparse
import json
import sys
from math import atan2, cos, hypot, pi, radians, sin, sqrt, tau
from pathlib import Path
from random import Random

import bpy
from mathutils import Vector

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(Path(__file__).resolve().parent))
sys.path.insert(0, str(ROOT / "scripts" / "blender_forest_arena"))
import mesh  # noqa: E402
from mesh import box, instance, material, pebble, prototype, surface, tube  # noqa: E402

SIZE = 200
BLUE = (24, 24)
RED = (176, 176)
PLAZA = (100, 100)
PLAZA_RADIUS = 12
BASE_RADIUS = 26
LANE_HALF_WIDTH = 3.6
WATER_HALF_WIDTH = 8.2
BRIDGE_LENGTH = 18.0
BRIDGE_TOP = 0.425
NEXUS_RADIUS = 5.5
CAMP_RADIUS = 7.0
TOP = [(24, 24), (20, 50), (20, 110), (25, 164), (36, 175), (90, 180), (150, 180), (176, 176)]
MID = [(24, 24), (60, 60), (100, 100), (140, 140), (176, 176)]
BOT = [(24, 24), (50, 20), (110, 20), (164, 25), (175, 36), (180, 90), (180, 150), (176, 176)]
LANES = {"Top": TOP, "Mid": MID, "Bot": BOT}
# Blue towers from map.rs; Red is mirrored across x + y = SIZE.
BLUE_TOWERS = [
    ("outer", "Top", (25, 112)), ("inner", "Top", (25, 78)), ("base", "Top", (26, 50)),
    ("outer", "Mid", (73, 79)), ("inner", "Mid", (56, 62)), ("base", "Mid", (36, 50)),
    ("outer", "Bot", (112, 25)), ("inner", "Bot", (78, 25)), ("base", "Bot", (50, 26)),
    ("nexus", None, (31, 44)), ("nexus", None, (44, 31)),
]
BLUE_CAMPS = [(48, 86), (72, 112), (86, 48), (112, 72)]
CAMP_KINDS = ["Wolves", "Golems", "Jaguar", "Frogs"]
CAMP_TINT = {"Wolves": "gold", "Golems": "ore", "Jaguar": "poppy", "Frogs": "water-light"}
BRIDGES = [((30.5, 169.5), radians(45)), ((169.5, 30.5), radians(45))]
LIBRARY = ROOT / "docs/media/blender-strongholds/verdant-forest-strongholds.blend"
LIBRARY_COLLECTIONS = [
    "Classic Stronghold", "Blue Stronghold banners", "Red Stronghold banners",
    "Full pine", "Sparse pine", "Coastal oak", "Fern", "Shrub", "Wildflowers",
    "Meadow grass", "Mossy rocks", "Tide rocks", "Brazier", "Split-rail fence", "Supplies",
]


def group(scene, name):
    collection = bpy.data.collections.new(name)
    scene.collection.children.link(collection)
    return collection


def mirror(point):
    return (SIZE - point[1], SIZE - point[0])


def towers():
    rows = []
    for team, flip in (("Blue", False), ("Red", True)):
        for tier, lane, point in BLUE_TOWERS:
            at = mirror(point) if flip else point
            rows.append((team, tier, lane, at))
    return rows


def camps():
    rows = []
    for index, point in enumerate(BLUE_CAMPS):
        rows.append((point, CAMP_KINDS[index]))
    for index, point in enumerate(BLUE_CAMPS):
        rows.append((mirror(point), CAMP_KINDS[index]))
    return rows


def point_segment_distance(p, a, b):
    ax, ay = a
    bx, by = b
    px, py = p
    dx, dy = bx - ax, by - ay
    length = dx * dx + dy * dy
    if length == 0:
        return hypot(px - ax, py - ay)
    t = max(0.0, min(1.0, ((px - ax) * dx + (py - ay) * dy) / length))
    return hypot(px - (ax + t * dx), py - (ay + t * dy))


def lane_distance(p):
    return min(point_segment_distance(p, a, b) for line in LANES.values()
               for a, b in zip(line, line[1:]))


def river_distance(p):
    return point_segment_distance(p, (8, 192), (192, 8))


def near_base(p):
    return min(hypot(p[0] - b[0], p[1] - b[1]) for b in (BLUE, RED)) < BASE_RADIUS + 2


def near_plaza(p):
    return hypot(p[0] - PLAZA[0], p[1] - PLAZA[1]) < PLAZA_RADIUS + 3


def near_tower(p):
    return any(hypot(p[0] - at[0], p[1] - at[1]) < 6.5 for _, _, _, at in towers())


def near_camp(p):
    return any(hypot(p[0] - at[0], p[1] - at[1]) < 9.0 for at, _ in camps())


def clear_of_layout(p):
    return (4 < p[0] < SIZE - 4 and 4 < p[1] < SIZE - 4
            and lane_distance(p) > LANE_HALF_WIDTH + 2.2
            and river_distance(p) > WATER_HALF_WIDTH + 1.8
            and not near_base(p) and not near_plaza(p)
            and not near_tower(p) and not near_camp(p))


def load_library():
    if not LIBRARY.exists():
        raise SystemExit(f"Blender library not found: {LIBRARY}")
    with bpy.data.libraries.load(str(LIBRARY), link=False) as (source, target):
        target.collections = [name for name in LIBRARY_COLLECTIONS if name in source.collections]
    loaded = {collection.name: collection for collection in target.collections}
    missing = [name for name in LIBRARY_COLLECTIONS if name not in loaded]
    if missing:
        print("missing library collections:", ", ".join(missing), flush=True)
    for collection in loaded.values():
        collection.asset_mark()
    return loaded


def ribbon(collection, name, points, width, z, key, shade=None):
    half = width / 2
    vertices, faces, colors = [], [], []
    for a, b in zip(points, points[1:]):
        dx, dy = b[0] - a[0], b[1] - a[1]
        length = hypot(dx, dy)
        nx, ny = -dy / length * half, dx / length * half
        offset = len(vertices)
        vertices.extend([
            (a[0] + nx, a[1] + ny, z), (a[0] - nx, a[1] - ny, z),
            (b[0] - nx, b[1] - ny, z), (b[0] + nx, b[1] + ny, z),
        ])
        faces.append((offset, offset + 1, offset + 2, offset + 3))
        colors.append(shade or key)
    return surface(collection, name, vertices, faces, colors)


def segment_box(collection, name, a, b, width, height, z, key, bevel=0.04):
    dx, dy = b[0] - a[0], b[1] - a[1]
    length = hypot(dx, dy)
    center = ((a[0] + b[0]) / 2, (a[1] + b[1]) / 2, z)
    return box(collection, name, center, (length, width, height), key, bevel, atan2(dy, dx))


def cap_river_approach(x, y, limit):
    shift = max(0.0, (x + y) / sqrt(2) - limit) / sqrt(2)
    return x - shift, y - shift


def disc(collection, name, center, radius, z, key, rings=1, height=0.0, accent=True,
         river_limit=float("inf")):
    segments = 40
    vertices, faces, colors = [], [], []
    for ring in range(rings + 1):
        r = radius * (1 - ring / (rings + 1))
        for i in range(segments):
            a = tau * i / segments
            x, y = cap_river_approach(radius * cos(a), radius * sin(a), river_limit)
            factor = r / radius
            vertices.append((center[0] + factor * x, center[1] + factor * y, z + height * ring))
    for ring in range(rings):
        for i in range(segments):
            j = (i + 1) % segments
            a = ring * segments + i
            b = ring * segments + j
            c = (ring + 1) * segments + j
            d = (ring + 1) * segments + i
            faces.append((a, b, c, d))
            colors.append(key if (i + ring) % 5 or not accent else "stone-light")
    center_index = len(vertices)
    vertices.append((center[0], center[1], z + height * rings))
    inner = rings * segments
    for i in range(segments):
        faces.append((center_index, inner + i, inner + (i + 1) % segments))
        colors.append(key)
    return surface(collection, name, vertices, faces, colors)


def build_prototypes(library):
    assets = {}

    def proto(name):
        assets[name] = prototype(name)
        return assets[name]

    # Cobble variants reuse the library's worn sandstone look.
    for index, tone in enumerate(("sand", "sand-light", "sand-dark")):
        box(proto(f"Tupi flagstone {index + 1}"), "Hand-dressed flag", (0, 0, 0),
            (1.95, 1.95, 0.075), tone, 0.06)
    for name, key in (("Tupi curb", "curb"), ("Tupi curb lip", "curb-light")):
        box(proto(name), "Shaped road shoulder", (0, 0, 0), (2.0, 0.38, 0.18), key, 0.035)

    bridge = proto("Tupi bridge deck")
    box(bridge, "Stone bridge deck", (0, 0, BRIDGE_TOP - 0.21),
        (BRIDGE_LENGTH, LANE_HALF_WIDTH * 2, 0.42), "stone", 0.10)
    for side in (-1, 1):
        rail = side * (LANE_HALF_WIDTH + 0.12)
        box(bridge, "Bridge stringer", (0, side * 3.32, 0.0),
            (BRIDGE_LENGTH, 0.30, 0.30), "wood-dark", 0.04)
        for x in (-7.8, -2.6, 2.6, 7.8):
            box(bridge, "Bridge pier", (x, side * 3.32, -0.98),
                (0.85, 0.85, 2.4), "stone-cool", 0.07)
        box(bridge, "Bridge handrail", (0, rail, 1.28),
            (BRIDGE_LENGTH, 0.18, 0.18), "wood", 0.04)
        for x in (-8.5, -5.6, -2.8, 0.0, 2.8, 5.6, 8.5):
            box(bridge, "Handrail post", (x, rail, 0.88),
                (0.20, 0.20, 1.0), "oak", 0.035)
        for x in (-7.8, 7.8):
            box(bridge, "Bridge lantern", (x, rail, 1.42), (0.24, 0.24, 0.18), "gold", 0.04)
            tube(bridge, "Lantern flame", [(x, rail, 1.51), (x, rail, 1.68)],
                 [0.10, 0.02], "fire", sides=6)

    plaza = proto("Tupi plaza island")
    tube(plaza, "Plaza retaining wall", [(0, 0, -1.4), (0, 0, 0.23)],
         [PLAZA_RADIUS, PLAZA_RADIUS], "stone-cool", sides=40, accent="stone")
    for index, (radius, z) in enumerate(((12.0, 0.26), (10.8, 0.28), (4.2, 0.30), (3.4, 0.32))):
        disc(plaza, "Plaza course", (0, 0), radius, z, "stone" if index % 2 else "stone-light")
    for index in range(8):
        a = tau * index / 8 + tau / 16
        box(plaza, "Plaza bollard", (10.8 * cos(a), 10.8 * sin(a), 0.49),
            (0.55, 0.55, 0.46), "stone-light", 0.08, a)
    tube(plaza, "Plaza crystal base", [(0, 0, 0.30), (0, 0, 1.16)],
         [2.15, 0.98], "stone-cool", sides=10)
    crystal = proto("Tupi plaza crystal")
    tube(crystal, "Plaza crystal", [(0, 0, 1.16), (0, 0, 4.1), (0, 0, 5.7)],
         [0.06, 1.1, 0.05], "water-light", sides=8, accent="glow")

    plateau = proto("Tupi base plateau")
    disc(plateau, "Base earthwork", (0, 0), BASE_RADIUS, 0.30, "stone-cool", rings=4, accent=False)
    disc(plateau, "Base court", (0, 0), BASE_RADIUS - 3.0, 0.52, "turf", rings=1, accent=False)
    for index in range(20):
        a = tau * index / 20
        box(plateau, "Plateau rim", ((BASE_RADIUS - 0.6) * cos(a), (BASE_RADIUS - 0.6) * sin(a), 0.42),
            (0.9, 2.2, 0.36), "stone" if index % 2 else "stone-light", 0.05, a)
    for index in range(3):
        box(plateau, "Base stair", (0, -BASE_RADIUS + 1.6 + index * 1.1, 0.32 + index * 0.08),
            (8.0, 1.05, 0.18 + index * 0.16), "stone-light", 0.04)

    plinth = proto("Tupi Nexus plinth")
    tube(plinth, "Nexus plinth", [(0, 0, 0), (0, 0, 1.05), (0, 0, 1.2)],
         [NEXUS_RADIUS, NEXUS_RADIUS, NEXUS_RADIUS - 0.2], "stone-cool", sides=32, accent="stone")
    disc(plinth, "Nexus inlay", (0, 0), NEXUS_RADIUS - 0.4, 1.21, "stone-light")

    for team, tint in (("Blue", "blue"), ("Red", "red")):
        tower = proto(f"Tupi tower {team}")
        box(tower, "Tower plinth", (0, 0, 0.45), (3.4, 3.4, 0.9), "stone-cool", 0.12)
        box(tower, "Tower court", (0, 0, 1.0), (2.9, 2.9, 0.3), "stone", 0.08)
        tube(tower, "Tower shaft", [(0, 0, 1.1), (0, 0, 4.4), (0, 0, 5.3)],
             [1.35, 1.05, 0.92], "stone", sides=10, accent="stone-light")
        box(tower, "Tower crown", (0, 0, 5.55), (2.5, 2.5, 0.55), "stone-light", 0.08)
        for index, x in enumerate((-1.0, -0.35, 0.35, 1.0)):
            for side, y in enumerate((-1.0, 1.0)):
                box(tower, f"{team} crown merlon {index * 2 + side}", (x, y, 5.95),
                    (0.42, 0.42, 0.42), "stone", 0.04)
        crystal = proto(f"Tupi tower crystal {team}")
        tube(crystal, "Tower crystal", [(0, 0, 6.45), (0, 0, 7.5), (0, 0, 8.7)],
             [0.05, 0.75, 0.04], tint, sides=7, accent="glow")
        tube(tower, "Tower finial", [(0, 0, 5.8), (0, 0, 6.35)], [0.06, 0.04], "gold", sides=6)
        for x in (-1.2, 1.2):
            tube(tower, "Pennant staff", [(x, -1.25, 3.1), (x, -1.25, 4.7)], [0.035, 0.025], "iron", sides=6)
            cloth(tower, "Tower pennant", tint, (x + 0.26, -1.25, 4.65), 0.50, 0.42)

    stronghold = proto("Tupi stronghold crest")
    tube(stronghold, "Crest spire", [(0, 0, 0), (0, 0, 2.3), (0, 0, 2.8)], [0.04, 0.42, 0.03],
         "gold", sides=8, accent="trim")
    pebble(stronghold, "Crest orb", (0, 0, 2.55), (0.34, 0.34, 0.30), "glow", sides=10)

    for kind, at in zip(CAMP_KINDS, BLUE_CAMPS, strict=True):
        camp = proto(f"Tupi camp {kind}")
        river_limit = (SIZE - sum(at)) / sqrt(2) - WATER_HALF_WIDTH - 0.35
        disc(camp, "Camp clearing", (0, 0), CAMP_RADIUS, 0.08, "path-bed",
             rings=2, accent=False, river_limit=river_limit)
        for index, (x, y, scale) in enumerate(((-6.5, 2.4, 1.8), (6.2, -2.3, 2.1),
                                               (2.4, 6.5, 1.5), (-3.6, -5.6, 1.7))):
            x, y = cap_river_approach(x, y, river_limit - 1.0)
            instance(camp, library["Mossy rocks"], "Camp outcrop", (x, y, 0.02), scale, index)
        tube(camp, "Camp totem", [(0, -6.5, 0.10), (0, -6.5, 2.0), (0, -6.5, 2.35)],
             [0.42, 0.30, 0.24], "stone-cool", sides=7)
        tube(camp, "Camp crystal", [(0, -6.5, 2.1), (0, -6.5, 3.3), (0, -6.5, 3.7)],
             [0.04, 0.55, 0.03], CAMP_TINT[kind], sides=6, accent="glow")
        for index in range(5):
            a = tau * index / 5
            x, y = cap_river_approach(6.7 * cos(a), 6.7 * sin(a), river_limit - 1.0)
            instance(camp, library["Fern"], "Camp fern", (x, y, 0.02), 1.8 + index * 0.12, a)
    return assets


def cloth(collection, name, key, center, width, height):
    x, y, z = center
    vertices = [(x + width * t, y + 0.046 * sin(t * 7 + r * 1.6),
                 z - r * height - (0.1 * height if r == 1 and abs(t) < 0.12 else 0))
                for r in (0, 0.39, 0.73, 1) for t in (-0.5, -0.25, 0, 0.25, 0.5)]
    faces = [(r * 5 + i, r * 5 + i + 1, (r + 1) * 5 + i + 1, (r + 1) * 5 + i)
             for r in range(3) for i in range(4)]
    surface(collection, name, vertices, faces, [key] * len(faces), smooth=True)


def assemble(scene, assets, library):
    rng = Random(505)
    ground = group(scene, "01  Valley meadow, river and jungle floor")
    lanes = group(scene, "02  Three lanes, bridges and plaza")
    bases = group(scene, "03  Base plateaus, Strongholds and Nexus")
    towers_layer = group(scene, "04  Towers and team pennants")
    camps_layer = group(scene, "05  Jungle camps")
    forest = group(scene, "06  Forest edge")
    understory = group(scene, "07  Shoreline understory and props")

    surface(ground, "Tupi Valley meadow", [(-2, -2, -0.02), (202, -2, -0.02),
                                           (202, 202, -0.02), (-2, 202, -0.02)],
            [(0, 1, 2, 3)], ["turf"])
    ribbon(ground, "River bed", [(8, 192), (192, 8)], WATER_HALF_WIDTH * 2, 0.00, "water-dark")
    ribbon(ground, "River surface", [(8, 192), (192, 8)], WATER_HALF_WIDTH * 2 - 0.4, 0.03, "water")
    for sign in (-1, 1):
        offset = WATER_HALF_WIDTH
        ribbon(ground, "River bank", [(8 + sign * offset, 192 + sign * offset),
                                      (192 + sign * offset, 8 + sign * offset)],
               2.6, 0.045, "sand")
    for index in range(18):
        t = index / 17
        x, y = 8 + (192 - 8) * t, 192 + (8 - 192) * t
        box(ground, "River ripple", (x + 1.8 * sin(index * 2.1), y + 1.4 * cos(index * 1.7), 0.045),
            (4.5, 0.26, 0.02), "ripple", 0.01, 0.8 + index * 0.2)

    for name, points in LANES.items():
        for segment, (a, b) in enumerate(zip(points, points[1:])):
            segment_box(lanes, f"{name} lane bed", a, b, LANE_HALF_WIDTH * 2 + 0.8, 0.10, 0.05, "path-bed", 0.05)
            dx, dy = b[0] - a[0], b[1] - a[1]
            length = hypot(dx, dy)
            nx, ny = -dy / length * (LANE_HALF_WIDTH + 0.2), dx / length * (LANE_HALF_WIDTH + 0.2)
            for sign in (-1, 1):
                segment_box(lanes, f"{name} shoulder", (a[0] + sign * nx, a[1] + sign * ny),
                            (b[0] + sign * nx, b[1] + sign * ny), 0.42, 0.20, 0.13,
                            "curb-light", 0.03)
            steps = max(2, int(length / 2.4))
            for step in range(steps):
                t = (step + 0.5) / steps
                x, y = a[0] + dx * t, a[1] + dy * t
                for column in (-2.45, 0.0, 2.45):
                    instance(lanes, assets[f"Tupi flagstone {1 + (step + int(column)) % 3}"],
                             "Worn lane flag", (x + nx * column / (LANE_HALF_WIDTH + 0.2),
                                                y + ny * column / (LANE_HALF_WIDTH + 0.2), 0.11),
                             1.0, atan2(dy, dx) + rng.uniform(-0.03, 0.03))
    for at, angle in BRIDGES:
        instance(lanes, assets["Tupi bridge deck"], "Stone bridge", (at[0], at[1], 0.0), 1.0, angle)
    for key in ("Tupi plaza island", "Tupi plaza crystal"):
        instance(lanes, assets[key], key, (*PLAZA, 0.0), 1.0, 0.0)

    for team, base in (("Blue", BLUE), ("Red", RED)):
        instance(bases, assets["Tupi base plateau"], f"{team} base plateau", (base[0], base[1], 0.0), 1.0, radians(135) if team == "Blue" else radians(-45))
        instance(bases, library["Classic Stronghold"], f"{team} Stronghold",
                 (base[0], base[1], 0.52), 6.0, radians(135) if team == "Blue" else radians(-45))
        instance(bases, library[f"{team} Stronghold banners"], f"{team} Stronghold banners",
                 (base[0], base[1], 0.52), 6.0, radians(135) if team == "Blue" else radians(-45))
        instance(bases, assets["Tupi Nexus plinth"], f"{team} Nexus plinth",
                 (base[0], base[1], 0.52), 1.0, 0.0)
        instance(bases, assets["Tupi stronghold crest"], f"{team} stronghold crest",
                 (base[0], base[1], 1.35), 1.0, 0.0)

    for team, tier, lane, at in towers():
        inside_base = min(hypot(at[0] - b[0], at[1] - b[1]) for b in (BLUE, RED)) < BASE_RADIUS - 1
        z = 0.52 if inside_base else 0.0
        scale = 1.15 if tier == "nexus" else 1.0
        for part in ("tower", "tower crystal"):
            instance(towers_layer, assets[f"Tupi {part} {team}"],
                     f"{team} {tier} {lane or 'Nexus'} {part}", (*at, z), scale, 0.0)

    for at, kind in camps():
        yaw = pi if sum(at) > SIZE else 0.0
        instance(camps_layer, assets[f"Tupi camp {kind}"], f"{kind} camp", (*at, 0.0), 1.0, yaw)

    placed = 0
    while placed < 380:
        p = (rng.uniform(4, SIZE - 4), rng.uniform(4, SIZE - 4))
        if not clear_of_layout(p):
            continue
        tree = "Full pine" if rng.random() < 0.62 else ("Sparse pine" if rng.random() < 0.72 else "Coastal oak")
        scale = rng.uniform(2.2, 3.7) * (1.0 if tree != "Coastal oak" else 0.82)
        instance(forest, library[tree], tree, (p[0], p[1], 0.0), scale, rng.uniform(-tau, tau))
        placed += 1
    placed = 0
    while placed < 700:
        p = (rng.uniform(4, SIZE - 4), rng.uniform(4, SIZE - 4))
        if not clear_of_layout(p):
            continue
        roll = rng.random()
        if roll < 0.38:
            name, scale = "Fern", rng.uniform(1.2, 2.2)
        elif roll < 0.60:
            name, scale = "Shrub", rng.uniform(1.2, 2.0)
        elif roll < 0.78:
            name, scale = "Meadow grass", rng.uniform(1.2, 2.1)
        elif roll < 0.91:
            name, scale = "Wildflowers", rng.uniform(1.1, 2.0)
        else:
            name, scale = "Mossy rocks", rng.uniform(1.1, 2.4)
        instance(understory, library[name], name, (p[0], p[1], 0.02), scale, rng.uniform(-tau, tau))
        placed += 1

    for team, base in (("Blue", BLUE), ("Red", RED)):
        outward = ((PLAZA[0] - base[0], PLAZA[1] - base[1]))
        length = hypot(*outward)
        ux, uy = outward[0] / length, outward[1] / length
        for index in range(3):
            x = base[0] + ux * (BASE_RADIUS + 1.7 + index * 2.0)
            y = base[1] + uy * (BASE_RADIUS + 1.7 + index * 2.0)
            instance(understory, library["Brazier"], "Base brazier", (x, y, 0.34), 1.25, 0.0)
        for index in range(4):
            angle = atan2(uy, ux) + (-0.35 + index * 0.23)
            x = base[0] + cos(angle) * (BASE_RADIUS - 0.5)
            y = base[1] + sin(angle) * (BASE_RADIUS - 0.5)
            instance(understory, library["Split-rail fence"], "Base fence", (x, y, 0.34), 1.9, angle + tau / 4)
        instance(understory, library["Supplies"], "Base supplies", (base[0] + 8.0, base[1] - 5.0, 0.52),
                 1.6, 0.3)

    for at, kind in camps():
        for index in range(6):
            a = tau * index / 6 + 0.2
            x = at[0] + cos(a) * 10.0
            y = at[1] + sin(a) * 10.0
            if clear_of_layout((x, y)) or near_camp((x, y)):
                instance(understory, library["Fern"], "Camp approach fern", (x, y, 0.02),
                         1.5 + index * 0.12, a)
    return ground, lanes, bases, towers_layer, camps_layer, forest, understory


def aim(obj, at):
    obj.rotation_euler = (Vector(at) - obj.location).to_track_quat("-Z", "Y").to_euler()


def camera(scene, name, origin, target, ortho=None):
    data = bpy.data.cameras.new(name)
    obj = bpy.data.objects.new(name, data)
    scene.collection.objects.link(obj)
    obj.location = origin
    aim(obj, target)
    data.clip_end = 700
    if ortho:
        data.type = "ORTHO"
        data.ortho_scale = ortho
    else:
        data.sensor_fit = "VERTICAL"
        data.angle_y = radians(32)
    return obj


def lighting(scene, studio=False):
    world = bpy.data.worlds.new(scene.name + " ambient")
    scene.world = world
    world.use_nodes = True
    background = world.node_tree.nodes.get("Background")
    background.inputs["Color"].default_value = (0.42, 0.55, 0.72, 1)
    background.inputs["Strength"].default_value = 0.34
    rig = group(scene, "Lighting rig")
    sun = bpy.data.lights.new("Warm valley sun", "SUN")
    obj = bpy.data.objects.new(sun.name, sun)
    rig.objects.link(obj)
    focus = (0, 0, 0) if studio else (100, 100, 0)
    obj.location = (-12, -20, 65) if studio else (-80, -120, 180)
    aim(obj, focus)
    sun.energy, sun.angle = 2.4, radians(8)
    sun.color = (1.0, 0.86, 0.70)
    fill = bpy.data.lights.new("Cool sky bounce", "AREA")
    obj = bpy.data.objects.new(fill.name, fill)
    rig.objects.link(obj)
    obj.location = (10, 18, 22) if studio else (200, 20, 130)
    aim(obj, focus)
    fill.energy, fill.shape, fill.size = (1900, "DISK", 25) if studio else (9000, "DISK", 80)
    fill.color = (0.72, 0.85, 1.0)


def render_setup(scene, width, height, samples):
    scene.render.engine = "CYCLES"
    scene.cycles.samples = samples
    scene.cycles.use_denoising = True
    scene.cycles.max_bounces = 4
    scene.render.resolution_x, scene.render.resolution_y = width, height
    scene.render.resolution_percentage = 100
    scene.render.image_settings.file_format = "PNG"
    scene.view_settings.view_transform = "AgX"
    scene.view_settings.look = "AgX - Medium High Contrast"
    scene.view_settings.exposure = 0.35


def collection_triangles(collection):
    total = 0
    for obj in collection.objects:
        if obj.type == "MESH":
            total += sum(len(poly.vertices) - 2 for poly in obj.data.polygons)
        elif obj.instance_type == "COLLECTION" and obj.instance_collection:
            total += collection_triangles(obj.instance_collection)
    for child in collection.children:
        total += collection_triangles(child)
    return total


def scene_triangles(scene):
    total = 0
    for obj in scene.objects:
        if obj.type == "MESH":
            total += sum(len(poly.vertices) - 2 for poly in obj.data.polygons)
        elif obj.instance_type == "COLLECTION" and obj.instance_collection:
            total += collection_triangles(obj.instance_collection)
    return total


def label(collection, body, at):
    data = bpy.data.curves.new(body, "FONT")
    data.body, data.size, data.align_x = body, 0.45, "CENTER"
    obj = bpy.data.objects.new(body, data)
    collection.objects.link(obj)
    obj.location = at
    data.materials.append(material("linen"))


def component_scene(assets, library, samples):
    scene = bpy.data.scenes.new("02  Tupi Valley component board")
    collection = group(scene, "Editable Tupi Valley instances")
    box(collection, "Studio tabletop", (0, 0, -0.35), (58, 38, 0.30), "cliff-shadow")
    samples_to_show = [
        ("Tupi base plateau", 0.12), ("Tupi Nexus plinth", 0.56), ("Classic Stronghold", 2.1),
        ("Tupi tower Blue", 0.82), ("Tupi tower Red", 0.82), ("Tupi bridge deck", 0.40),
        ("Tupi plaza island", 0.26), ("Tupi plaza crystal", 0.85), ("Tupi camp Wolves", 0.46),
        ("Tupi camp Golems", 0.46), ("Tupi camp Jaguar", 0.46), ("Tupi camp Frogs", 0.46),
        ("Tupi flagstone 1", 1.6), ("Tupi flagstone 2", 1.6), ("Tupi flagstone 3", 1.6),
        ("Tupi curb", 2.1), ("Tupi curb lip", 2.1), ("Tupi stronghold crest", 1.35),
        ("Tupi tower crystal Blue", 0.8), ("Tupi tower crystal Red", 0.8),
        ("Full pine", 2.4), ("Coastal oak", 2.0), ("Mossy rocks", 2.4), ("Brazier", 2.4),
    ]
    for index, (name, scale) in enumerate(samples_to_show):
        x, y = (index % 6 - 2.5) * 9.2, (1.5 - index // 6) * 8.2
        box(collection, "Sample platform", (x, y, -0.105), (8.6, 7.6, 0.20), "distant", 0.08)
        source = assets.get(name) or library.get(name)
        if source is None:
            continue
        lift = -6.45 * scale if name.startswith("Tupi tower crystal") else 0.0
        instance(collection, source, name, (x, y + 0.3, lift + 0.02), scale, 0.0)
        label(collection, name.removeprefix("Tupi ").title(), (x, y - 3.25, 0.03))
    lighting(scene, studio=True)
    render_setup(scene, 1600, 900, samples)
    cam = camera(scene, "Component board camera", (0, -17, 40), (0, 0, 0), ortho=63)
    scene.camera = cam
    return scene


def main():
    args = sys.argv[sys.argv.index("--") + 1:] if "--" in sys.argv else []
    parser = argparse.ArgumentParser()
    parser.add_argument("out", type=Path)
    parser.add_argument("--project-only", action="store_true")
    parser.add_argument("--samples", type=int, default=24)
    parser.add_argument("--preview-width", type=int, default=1600)
    parser.add_argument("--runtime", type=Path, help="export pieces for the Bevy scene")
    options = parser.parse_args(args)
    options.out.mkdir(parents=True, exist_ok=True)

    bpy.ops.wm.read_factory_settings(use_empty=True)
    scene = bpy.context.scene
    scene.name = "01  Tupi Valley 5v5"
    library = load_library()
    assets = build_prototypes(library)
    ground, lanes, bases, towers_layer, camps_layer, forest, understory = assemble(scene, assets, library)
    lighting(scene)
    render_setup(scene, 1600, 900, options.samples)
    overview = camera(scene, "Overview camera", (100, 100, 300), (100, 100, 0), ortho=224)
    match = camera(scene, "Match camera", (100, 46.9, 78.8), (100, 100, 0))
    scene.camera = match
    board = component_scene(assets, library, options.samples)
    if options.runtime:
        from export_runtime import export
        metrics_runtime = export(options.runtime, assets)
    else:
        metrics_runtime = {}

    metrics = {
        "map_size": SIZE,
        "runtime_triangles": metrics_runtime,
        "lanes": {name: len(points) - 1 for name, points in LANES.items()},
        "towers": len(list(towers())),
        "camps": len(list(camps())),
        "collections": sorted(c.name for c in bpy.data.collections),
        "library_collections": sorted(library),
        "scene_objects": len(scene.objects),
        "scene_triangles": scene_triangles(scene),
        "board_objects": len(board.objects),
        "board_triangles": scene_triangles(board),
        "camera": {"fov_degrees": 32, "pitch_degrees": 56, "distance": 95},
    }
    if not options.project_only:
        for scene_to_render, camera_to_use, name, size in (
            (scene, overview, "tupi-valley-overview.png", options.preview_width),
            (scene, match, "tupi-valley-match.png", options.preview_width),
            (board, board.camera, "tupi-valley-kit.png", options.preview_width),
        ):
            bpy.context.window.scene = scene_to_render
            scene_to_render.camera = camera_to_use
            scene_to_render.render.resolution_x = size
            scene_to_render.render.resolution_y = size if name.endswith("overview.png") else size * 9 // 16
            scene_to_render.render.filepath = str(options.out / name)
            bpy.ops.render.render(write_still=True)
            print("rendered", name, flush=True)

    bpy.context.window.scene = scene
    bpy.ops.wm.save_as_mainfile(filepath=str(options.out / "tupi-valley-5v5.blend"), compress=True)
    (options.out / "metrics.json").write_text(json.dumps(metrics, indent=2) + "\n")
    print(json.dumps(metrics, indent=2), flush=True)


if __name__ == "__main__":
    main()
