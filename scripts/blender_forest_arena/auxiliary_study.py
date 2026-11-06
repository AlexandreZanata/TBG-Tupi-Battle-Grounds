"""Stage both animated War Auxiliaries on the approved Verdant Forest."""

import argparse
from pathlib import Path
import sys

import bpy

sys.path.insert(0,str(Path(__file__).resolve().parent))
import build as forest
from aux_merchant import merchant
from aux_miner import miner
from aux_parts import END, FPS
from hold_parts import empty

SITES = (
    ("Mineiro · west",miner,(-5.05,2.48),False,(1.07,0,2.42,0),"blue"),
    ("Mineiro · east",miner,(20.73,1.26),True,(1.07,0,2.24,0),"red"),
    ("Mercador · west",merchant,(-1.10,-5.63),False,(0,.70,0,2.01),"blue"),
    ("Mercador · east",merchant,(16.30,-5.85),True,(0,.70,0,2.20),"red"),
)

STILLS = (
    ("forest-auxiliaries","Overview",26.0),
    ("miner-construction","Mineiro",8.30),
    ("miner-working","Mineiro",24.24),
    ("merchant-construction","Mercador",2.22),
    ("merchant-delivery","Mercador",24.55),
)


def place(scene):
    for name,model,(x,y),mirror,bridge,seat in SITES:
        group = forest.group(scene,name)
        root = empty(group,name+" precinct",(x,y,0))
        root.scale.x = -1 if mirror else 1
        root["Seat"] = "B" if mirror else "A"
        root["Design"] = "Verdant Forest offshore study"
        model(group,root,bridge,seat)


def cameras(scene):
    return {
        "Overview":forest.camera(scene,"Overview",(4.75,-26.8,24.6),(8,-.8,-.28),35.1),
        "Mineiro":forest.camera(scene,"Mineiro",(-2.57,-2.46,4.31),(-4.41,2.46,.53),5.75),
        "Mercador":forest.camera(scene,"Mercador",(1.02,-12.02,4.51),(-.93,-6.45,-.08),6.55),
    }


def timeline(scene):
    scene.render.fps = FPS
    scene.frame_start,scene.frame_end = 1,END*FPS+1
    for time,name in ((0,"Build starts"),(3,"Merchant open"),(5.2,"Merchant delivery 1"),
                      (10,"Merchant level 2"),(12,"Miner begins work"),(13,"Merchant delivery 2"),
                      (17,"Miner level 2"),(19,"Merchant level 3"),(22,"Miner level 3"),
                      (23.1,"Merchant delivery 3"),(28.2,"Completion pose")):
        scene.timeline_markers.new(name,frame=round(time*FPS)+1)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("output",type=Path)
    parser.add_argument("--draft",action="store_true")
    parser.add_argument("--project-only",action="store_true")
    opts = parser.parse_args(sys.argv[sys.argv.index("--")+1:])
    out = opts.output.resolve()
    out.mkdir(parents=True,exist_ok=True)
    bpy.ops.wm.read_factory_settings(use_empty=True)
    scene = bpy.context.scene
    scene.name = "Forest · animated War Auxiliaries"
    forest.assemble(scene,forest.kit())
    place(scene)
    views = cameras(scene)
    forest.lighting(scene)
    forest.render_setup(scene,1550,970,22)
    timeline(scene)
    scene.camera = views["Overview"]
    scene.frame_set(round(26*FPS)+1)
    for area in bpy.context.screen.areas:
        if area.type=="VIEW_3D":
            area.spaces.active.region_3d.view_perspective = "CAMERA"
    bpy.ops.wm.save_as_mainfile(filepath=str(out/"verdant-forest-auxiliaries.blend"),compress=True)
    if opts.project_only:
        return
    for file,camera,time in STILLS:
        scene.camera = views[camera]
        scene.frame_set(round(time*FPS)+1)
        scene.render.resolution_percentage = 48 if opts.draft else 100
        scene.cycles.samples = 8 if opts.draft else 22
        scene.render.filepath = str(out/(file+".png"))
        bpy.ops.render.render(write_still=True)


if __name__=="__main__":
    main()
