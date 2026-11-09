"""Export the playable scenery, Strongholds and event-driven auxiliary clips."""

import argparse
import importlib
from pathlib import Path
import sys

import bpy
from mathutils import Matrix

sys.path.insert(0, str(Path(__file__).resolve().parent))
from mesh_pack import Batch, IDENTITY, pose, write

SCRIPTS = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(SCRIPTS / "blender_forest_arena"))
import auxiliary_study
import build as forest
from aux_parts import END, FPS
from hold_parts import FORMS
from stronghold_study import MODELS

THEMES = {
    "forest": ("forest", "build"),
    "tundra": ("tundra", "tundra_study"),
    "volcanic": ("volcanic", "forge_study"),
    "ruins": ("ruins", "ruins_study"),
    "storm": ("storm", "storm_study"),
    "celestial": ("celestial", "celestial_study"),
    "clockwork": ("clockwork", "clockwork_study"),
    "desert": ("desert", "desert_study"),
}
SAMPLE_RATE = 8


def scene_reset():
    bpy.ops.wm.read_factory_settings(use_empty=True)
    return bpy.context.scene


def load_theme(theme):
    folder, module = THEMES[theme]
    sys.path.insert(0, str(SCRIPTS / f"blender_{folder}_arena"))
    return importlib.import_module(module)


def scenery(output):
    deps = bpy.context.evaluated_depsgraph_get()
    batches = {}
    for inst in deps.object_instances:
        obj = inst.object
        if obj.type != "MESH":
            continue
        owner = inst.parent.original if inst.is_instance else obj.original
        layer = owner.users_collection[0].name[:2]
        if layer not in {"01", "02", "04", "05", "06", "07", "09", "10"}:
            continue
        materials = {mat.name for mat in obj.data.materials}
        water = layer == "01" and any(
            word in owner.name.lower() for word in ("sea", "lagoon", "water", "shelf", "basin")
        )
        group = "water" if water else "terrain"
        matrix = inst.matrix_world.copy()
        track = (pose(IDENTITY),)
        if "Angular speed" in owner:
            group = {"Foundry central drive": "gear0", "Foundry left follower": "gear1",
                     "Foundry right follower": "gear2"}[owner.name]
            transform = owner.matrix_world.copy()
            matrix = transform.inverted() @ matrix
            track = (pose(transform),)
        elif not water and materials and materials <= {"fire", "glow", "ember"}:
            group = "light"
        batches.setdefault(group, Batch(track, units=512)).add(obj, matrix)
    if not {"water", "terrain"} <= batches.keys():
        raise ValueError(f"Missing scenery layer in {output}")
    write(output, batches)


def classic_skin(output, theme):
    batches = {"blue": {}, "red": {}}
    deps = bpy.context.evaluated_depsgraph_get()
    for i, inst in enumerate(deps.object_instances):
        if not inst.is_instance or inst.object.type != "MESH":
            continue
        owner = inst.parent.original
        if not owner.users_collection[0].name.startswith("03"):
            continue
        x = owner.matrix_world.translation.x
        seat = "blue" if x < 8.0 else "red"
        loss = 6 + i % 14 if max(inst.object.dimensions) < .57 else 21
        mesh = batches[seat].setdefault(f"wear{loss}", Batch())
        mesh.add(inst.object, Matrix.Translation((-x, 0, 0)) @ inst.matrix_world)
    for seat, skin in batches.items():
        write(output / f"{theme}-classic-{seat}.tbgd", skin)


def animated_ancestor(obj, root):
    while obj != root and obj.animation_data is None:
        obj = obj.parent
    return obj


def auxiliary_parts(collection, root):
    grouped = {}
    for obj in collection.all_objects:
        if obj.type == "MESH":
            pivot = animated_ancestor(obj.parent, root)
            grouped.setdefault(pivot, []).append(obj)
    return grouped


def precinct_models(scene, precincts, target, theme):
    models = []
    for precinct in precincts:
        root, = [obj for obj in precinct.objects if obj.get("Seat") in {"A", "B"}]
        kind = "miner" if "Mineiro" in precinct.name else "merchant"
        models.append((root, kind, auxiliary_parts(precinct, root)))
    tracks = {pivot: [] for _, _, grouped in models for pivot in grouped}
    for tick in range(END * SAMPLE_RATE + 1):
        scene.frame_set(1 + tick * FPS // SAMPLE_RATE)
        for root, _, grouped in models:
            for pivot in grouped:
                tracks[pivot].append(pose(relative(pivot, root)))
    scene.frame_set(1)
    for root, kind, grouped in models:
        seat = root["Seat"].lower()
        auxiliary(root, grouped, tracks, target / f"{theme}-{kind}-{seat}.tbgd")


def relative(obj, pivot):
    matrix = IDENTITY.copy()
    while obj != pivot:
        matrix = obj.matrix_parent_inverse @ obj.matrix_basis @ matrix
        obj = obj.parent
    return matrix


def auxiliary(root, grouped, tracks, output):
    deps = bpy.context.evaluated_depsgraph_get()
    batches = {}
    for i, (pivot, objects) in enumerate(grouped.items()):
        batch = Batch(tracks[pivot])
        for obj in objects:
            evaluated = obj.evaluated_get(deps)
            batch.add(evaluated, relative(obj, pivot))
        role = "part"
        if pivot.name.startswith(("Merchant caravel", "Delivered parcel", "Working crane line",
                                  "Hinged treasure lid", "Payout coin")):
            role = "delivery"
        batches[f"{role}{i}"] = batch
    write(output, batches, pose(root.matrix_world)[:3])


def export_theme(theme, target, classic_only):
    scene = scene_reset()
    study = load_theme(theme)
    if theme != "forest":
        study.terrain.register_palette()
    study.assemble(scene, study.kit())
    scene.frame_set(1)
    classic_skin(target, theme)
    if classic_only:
        return
    scenery(target / f"{theme}.tbgd")
    if theme == "forest":
        auxiliary_study.place(scene)
        precincts = [scene.collection.children[site[0]] for site in auxiliary_study.SITES]
    else:
        precincts = list(scene.collection.children["08  Mineiros and Mercadores"].children)
    for layer in list(scene.collection.children):
        if layer not in precincts and not layer.name.startswith("08"):
            scene.collection.children.unlink(layer)
    precinct_models(scene, precincts, target, theme)


def stronghold(target, kind, form, seat):
    scene = scene_reset()
    root = bpy.data.collections.new(kind)
    scene.collection.children.link(root)
    MODELS[kind](root, form, seat)
    bpy.context.view_layer.update()
    deps = bpy.context.evaluated_depsgraph_get()
    batches = {}
    muzzles = {}
    for i, obj in enumerate(root.all_objects):
        if obj.type != "MESH":
            continue
        pivot = obj.parent
        evaluated = obj.evaluated_get(deps)
        if pivot is None:
            size = max(obj.dimensions)
            loss = (6 + i % 14) if size < .57 else (17 + i % 3) if size < .83 else 21
            name = f"wear{loss}"
            batch = batches.setdefault(name, Batch())
            batch.add(evaluated, obj.matrix_world)
            continue
        motion = pivot["Motion"]
        marker = next((child for child in pivot.children if child.name.startswith("Muzzle")), None)
        if marker is not None:
            name = "gun2" if "Sentinel" in pivot.name else "gun"
            muzzles[name] = pose(marker.matrix_local)[:3]
            if obj.name.startswith(("Nocked shaft", "Steel bolt head")):
                name += "bolt"
        else:
            name = ({"Yaw about deck center": "swivel", "Swivel around Stronghold": "orbit",
                     "Hatch": "hatch", "Notch on release": "notch", "Half-tooth advance": "notch",
                     "Turn": "turn", "Observation pivot": "observation"}[motion]
                    + str(list(root.all_objects).index(pivot)))
        batch = batches.setdefault(name, Batch((pose(pivot.matrix_world),)))
        batch.add(evaluated, pivot.matrix_world.inverted() @ obj.matrix_world)
    path = target / f"{kind.lower()}-{form.lower()}-{seat}.tbgd"
    for name, muzzle in muzzles.items():
        batches[name].socket = muzzle
    write(path, batches)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("output", type=Path)
    parser.add_argument("--themes", choices=THEMES, nargs="*", default=THEMES)
    parser.add_argument("--strongholds", action="store_true")
    parser.add_argument("--classic-only", action="store_true")
    args = parser.parse_args(sys.argv[sys.argv.index("--") + 1:])
    args.output.mkdir(parents=True, exist_ok=True)
    for theme in args.themes:
        export_theme(theme, args.output, args.classic_only)
    if args.strongholds:
        for kind in MODELS:
            for form in FORMS:
                for seat in ("blue", "red"):
                    stronghold(args.output, kind, form, seat)


if __name__ == "__main__":
    main()
