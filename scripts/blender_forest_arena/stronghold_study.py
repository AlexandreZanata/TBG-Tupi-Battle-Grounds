"""Assemble the four Strongholds in the approved Forest and render every form."""

import argparse
import json
from math import cos, radians, sin, tan
from pathlib import Path
import sys

import bpy

sys.path.insert(0,str(Path(__file__).resolve().parent))
import build as forest
from hold_artillery import artillery
from hold_bastion import bastion
from hold_command import command
from hold_parts import FORMS
from hold_treasury import treasury
from mesh import box, instance, prototype
from terrain import HOLD_CENTERS

MODELS = {"Treasury":treasury,"Bastion":bastion,"Artillery":artillery,"Command":command}
PAIRS = (("Treasury","B2","Bastion","B2"),("Artillery","A2","Command","B2"))


def models():
    assets = {}
    for name,build in MODELS.items():
        for form in FORMS:
            group = prototype(f"{name} · {form}")
            group.asset_data.description = f"{name} Stronghold, {form} Fortification form"
            build(group,form,"blue")
            assets[name,form] = group
    return assets


def match_scene(scene,pair,assets):
    left,left_form,right,right_form = pair
    scene.name = f"Forest · {left} and {right}"
    group = forest.group(scene,"Stronghold previews")
    instance(group,assets[left,left_form],left,(HOLD_CENTERS[0],0,0))
    enemy = bpy.data.collections.new(f"{right} · {right_form} · red review")
    MODELS[right](enemy,right_form,"red")
    instance(group,enemy,right,(HOLD_CENTERS[1],0,0),scale=(-1,1,1))
    return scene


def forest_views(assets,samples):
    scene = bpy.context.scene
    existing = forest.kit()
    forest.assemble(scene,existing)
    original = bpy.data.collections["03  Strongholds and architectural banners"]
    scene.collection.children.unlink(original)
    distance = max(13.5/(tan(radians(17.5))*16/9),7.1/tan(radians(17.5)))
    scene.camera = forest.camera(scene,"Desktop match framing",
                                 (8,-distance*cos(radians(55)),distance*sin(radians(55))),(8,0,0))
    forest.lighting(scene)
    forest.render_setup(scene,1700,956,samples)
    other = scene.copy()
    other.use_fake_user = True
    return [match_scene(current,pair,assets) for current,pair in zip((scene,other),PAIRS,strict=True)]


def lineup(name,assets,samples):
    scene = bpy.data.scenes.new(f"Fortifications · {name}")
    stage = forest.group(scene,"Display plinths and forms")
    box(stage,"Studio floor",(0,0,-.29),(180,180,.30),"cliff-shadow")
    for i,form in enumerate(FORMS):
        x = (i-2)*3.28
        box(stage,"Display plinth",(x,-.19,-.080),(2.98,3.42,.160),"distant",.094)
        instance(stage,assets[name,form],f"{name} {form}",(x,0,0),
                 angle=radians(-37 if name in ("Artillery","Bastion") else -23))
        forest.label(stage,form,(x,-1.715,.012))
    aim = .85 if name=="Artillery" else 1.25
    scene.camera = forest.camera(scene,"Fortification overview",(0,-12.8,8.4),(0,0,aim),17.0)
    forest.lighting(scene,studio=True)
    forest.render_setup(scene,1740,565,samples)
    return scene


def mesh_counts(scene,name):
    bpy.context.window.scene = scene
    values = dict.fromkeys(FORMS,0)
    by_mesh = {}
    for inst in bpy.context.evaluated_depsgraph_get().object_instances:
        if not inst.parent or inst.object.type!="MESH":
            continue
        parent = inst.parent.original
        if parent.instance_collection is None or not parent.name.startswith(name+" "):
            continue
        obj = inst.object
        if obj.data.name not in by_mesh:
            obj.data.calc_loop_triangles()
            by_mesh[obj.data.name] = len(obj.data.loop_triangles)
        form = parent.name.rsplit(" ",1)[1]
        values[form] += by_mesh[obj.data.name]
    assert all(values.values()), (name,values)
    return values


def main():
    args = argparse.ArgumentParser()
    args.add_argument("output",type=Path)
    args.add_argument("--samples",type=int,default=22)
    args.add_argument("--draft",action="store_true")
    args.add_argument("--project-only",action="store_true")
    options = args.parse_args(sys.argv[sys.argv.index("--")+1:])
    out = options.output.resolve()
    out.mkdir(parents=True,exist_ok=True)
    bpy.ops.wm.read_factory_settings(use_empty=True)
    assets = models()
    arenas = forest_views(assets,options.samples)
    boards = {name:lineup(name,assets,options.samples) for name in MODELS}
    counts = {name:mesh_counts(scene,name) for name,scene in boards.items()}
    (out/"stronghold-metrics.json").write_text(json.dumps(counts,indent=2)+"\n")
    print(counts,flush=True)
    bpy.context.window.scene = arenas[0]
    for area in bpy.context.screen.areas:
        if area.type=="VIEW_3D":
            area.spaces.active.region_3d.view_perspective = "CAMERA"
    bpy.ops.wm.save_as_mainfile(filepath=str(out/"verdant-forest-strongholds.blend"),compress=True)
    if options.project_only:
        return
    renders = [(name.lower(),scene) for name,scene in boards.items()]
    renders += [("forest-treasury-bastion",arenas[0]),("forest-artillery-command",arenas[1])]
    for file,scene in renders:
        bpy.context.window.scene = scene
        scene.render.resolution_percentage = 52 if options.draft else 100
        scene.render.filepath = str(out/(file+".png"))
        bpy.ops.render.render(write_still=True)


if __name__ == "__main__":
    main()
