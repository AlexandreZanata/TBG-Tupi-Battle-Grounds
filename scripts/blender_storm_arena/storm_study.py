"""Assemble the Storm Valley kit, editable scenery and animated auxiliary sites."""

import argparse
import json
from pathlib import Path
from random import Random
import sys

import bpy
from mathutils import Vector

HERE = Path(__file__).resolve().parent
sys.path[:0] = [str(HERE),str(HERE.parent/"blender_forest_arena"),str(HERE.parent/"blender_tundra_arena")]
import build as forest
from auxiliary_study import timeline
from aux_parts import FPS
from mesh import box, instance, prototype
from terrain import HOLD_CENTERS, LANE_LENGTH, LANE_WIDTH
from tundra_study import cameras as reference_cameras, render
from storm_auxiliaries import place
import storm_scenery as scenery
import storm_structures as structures
import storm_terrain as terrain

ARENA = "01  Storm Valley"
BOARD = "02  Storm Valley component board"
ROAD = "02  BattleLine and silver slate curbs"
HOLDS = "03  Storm Strongholds"
FIXTURES = "04  Copper fixtures and narrow railings"
BACKDROP = "05  Wind-bent pines and conductor ridge"
UNDERSTORY = "06  Silver grass and quartz slate"
COAST = "07  Slate skerries and offshore relay"
SITES_LAYER = "08  Mineiros and Mercadores"
HORIZON = "09  Low storm banks"
SCENIC_LAYERS = (FIXTURES,BACKDROP,UNDERSTORY,COAST,HORIZON)
OVERVIEW_TIME = 24.37


def kit():
    makers = {
        "Slate heath plateau":terrain.plateau, "Layered slate cliffs":terrain.cliffs,
        "Dark open sea":terrain.sea, "Broken coastal surf":terrain.shelf,
        "Storm road foundation":terrain.road_bed, "Slate curb":terrain.curb,
        "Storm Stronghold":structures.stronghold, "Ring conductor":structures.conductor,
        "Offshore relay deck":structures.relay_plinth, "Copper-caged lantern":structures.electric_lamp,
        "Low grounding coil":structures.ground_coil, "Riveted iron railing":structures.railing,
        "Wind-bent pine":scenery.wind_pine, "Silver grass":scenery.silver_grass,
        "Quartz slate boulder":scenery.slate_boulder, "Slate skerry":scenery.reef,
        "Relay islet":scenery.relay_islet, "Broken whitecaps":scenery.whitecap,
        "Low storm bank":scenery.cloud_bank, "Wrecked dock":scenery.wreck,
        "Blue Stronghold banners":lambda group: structures.standard(group,"blue"),
        "Red Stronghold banners":lambda group: structures.standard(group,"red"),
    }
    for i in range(4):
        makers[f"Storm flagstone {i+1}"] = lambda group,i=i: terrain.slab(group,i)
    assets = {name:prototype(name) for name in makers}
    for name,make in makers.items():
        make(assets[name])
    return assets


def assemble(scene, assets):
    rng = Random(508)

    def put(group,name,at,scale=1,angle=0):
        return instance(group,assets[name],name,at,scale,angle)

    place_lane(scene,put,rng)
    place_fixtures(scene,put)
    place_backdrop(scene,put,rng)
    place_understory(scene,put,rng)
    place_coast(scene,put,rng)
    place(forest.group(scene,SITES_LAYER))
    place_horizon(scene,put)


def place_lane(scene,put,rng):
    land = forest.group(scene,"01  Slate heath, cliffs and open sea")
    road = forest.group(scene,ROAD)
    castles = forest.group(scene,HOLDS)
    for name in ("Slate heath plateau","Layered slate cliffs","Dark open sea","Broken coastal surf"):
        put(land,name,(0,0,0))
    put(road,"Storm road foundation",(LANE_LENGTH/2,0,0))
    for i in range(18):
        x = i-.50
        for j in range(4):
            variant = 1+rng.choices(range(4),(8,3,6,2))[0]
            put(road,f"Storm flagstone {variant}",(x+(j%2)*.009,(j-1.5)*1.238,0),
                angle=rng.uniform(-.006,.006))
        for sign in (-1,1):
            put(road,"Slate curb",(x,sign*(LANE_WIDTH/2+.076),0),angle=sign*.008)
    for x,seat in zip(HOLD_CENTERS,("Blue","Red"),strict=True):
        put(castles,"Storm Stronghold",(x,0,0))
        put(castles,f"{seat} Stronghold banners",(x,0,0))


def place_fixtures(scene,put):
    group = forest.group(scene,FIXTURES)
    for sign in (-1,1):
        for x in (1.10,5.19,10.81,14.90):
            put(group,"Copper-caged lantern",(x,sign*2.99,-.047),.94)
        for x in (2.28,3.45,12.55,13.72):
            put(group,"Riveted iron railing",(x,sign*3.08,-.040),angle=sign*.037)
        for x in (6.41,9.59):
            put(group,"Low grounding coil",(x,sign*3.06,-.039),.91)


def place_backdrop(scene,put,rng):
    group = forest.group(scene,BACKDROP)
    for x,y,scale in ((1.41,4.45,.89),(5.69,4.92,.78),(10.39,4.91,.78),(14.61,4.42,.90)):
        put(group,"Ring conductor",(x,y,-.033),scale,rng.uniform(-.19,.19))
    for x,y in ((.11,4.14),(2.29,4.50),(3.16,4.73),(4.51,5.14),(6.84,5.58),(7.59,5.70),
                (8.74,5.62),(9.43,5.43),(11.64,5.12),(12.71,4.80),(13.47,4.42),(15.74,4.22),(16.47,3.84)):
        put(group,"Wind-bent pine",(x,y,-.031),rng.uniform(.75,1.23),rng.uniform(-.29,.16))


def place_understory(scene,put,rng):
    group = forest.group(scene,UNDERSTORY)
    for i in range(71):
        x,y = rng.uniform(-1.5,17.5),rng.uniform(3.55,5.59)
        side = 1 if i%3==0 else -1
        shore = ((x-8)/11.8)**2+(y/6.12)**2
        approach = side<0 and y>3.88 and min(abs(x-1.42),abs(x-14.58))<1.14
        if shore>.92 or approach:
            continue
        name = "Quartz slate boulder" if i%6==0 else "Silver grass"
        put(group,name,(x,side*y,-.052+(y-3.40)*.028),rng.uniform(.66,1.27),rng.uniform(-.25,.20))
    for x,y,scale in ((4.32,-4.68,1.14),(6.25,-4.98,.88),(9.72,-5.06,.96),(11.90,-4.56,1.11)):
        put(group,"Quartz slate boulder",(x,y,-.024),scale,rng.uniform(-.32,.24))


def place_coast(scene,put,rng):
    group = forest.group(scene,COAST)
    level = terrain.WATERLINE
    put(group,"Relay islet",(8.01,9.69,level+.31))
    put(group,"Offshore relay deck",(8.01,9.64,level+.614))
    for x,y,scale,angle in ((6.47,9.72,.74,-.13),(8.01,9.61,1.16,0),(9.55,9.72,.74,.13)):
        put(group,"Ring conductor",(x,y,level+.976),scale,angle)
    for x,y,scale in ((-4.79,-3.89,.83),(-2.76,-5.91,.91),(4.61,-7.56,.84),
                      (7.70,-8.39,1.27),(10.62,-8.12,.96),(18.38,-5.61,1.05),(20.52,-3.86,.78),
                      (2.46,8.45,1.26),(12.79,8.08,1.01),(15.43,8.75,.85)):
        put(group,"Slate skerry",(x,y,level+.021),scale,rng.uniform(-.31,.28))
    for x,y,angle in ((4.41,-6.68,.23),(11.32,-6.96,-.31),(3.23,7.43,.70),(13.89,7.16,-.57)):
        put(group,"Wrecked dock",(x,y,level+.055),angle=angle)
    for x,y in ((-5.26,-5.49),(-3.11,-7.08),(-2.38,-9.08),(.40,-10.36),(3.57,-8.82),
                (5.81,-7.69),(7.03,-10.18),(9.06,-8.92),(11.68,-9.47),(14.42,-10.69),
                (16.68,-8.91),(18.39,-6.98),(21.29,-5.06),(-3.06,7.04),(3.86,8.12),
                (5.20,7.18),(9.42,7.38),(12.55,7.91),(15.58,7.26),(18.63,6.79)):
        put(group,"Broken whitecaps",(x,y,level+.013),rng.uniform(.80,1.43),rng.uniform(-.26,.18))


def place_horizon(scene,put):
    group = forest.group(scene,HORIZON)
    for x,y,z,s in ((-4.16,12.81,.27,1.08),(2.88,14.32,.45,1.29),(10.34,14.78,.42,1.24),
                    (16.77,12.97,.21,1.23),(24.02,14.43,.32,1.02)):
        put(group,"Low storm bank",(x,y,z),(s,.96,.87),.08)


def lighting(scene):
    forest.lighting(scene)
    sky = scene.world.node_tree.nodes["Background"]
    sky.inputs["Color"].default_value = (.42,.50,.64,1)
    sky.inputs["Strength"].default_value = .59
    sun = next(obj.data for obj in scene.objects if obj.type=="LIGHT" and obj.data.type=="SUN")
    sun.color = (.88,.92,1)
    sun.energy = 1.83
    sun.angle = .31


def component_board(assets,samples):
    scene = bpy.data.scenes.new(BOARD)
    group = forest.group(scene,"Separate Storm Valley component samples")
    box(group,"Midnight slate tabletop",(0,0,-.32),(130,130,.26),"storm-slate-deep")
    examples = (
        ("Storm Stronghold",.60,0),("Ring conductor",.94,0),("Wind-bent pine",.97,0),
        ("Offshore relay deck",.48,0),("Copper-caged lantern",1.72,0),("Low grounding coil",1.69,0),
        ("Riveted iron railing",1.59,0),("Quartz slate boulder",1.43,0),("Slate skerry",1.21,.132),
        ("Wrecked dock",1.60,.08),("Broken whitecaps",1.14,0),("Relay islet",.36,.14),
        ("Silver grass",1.70,0),("Storm flagstone 1",1.19,0),("Storm flagstone 4",1.19,0),("Slate curb",1.69,0),
    )
    for i,(name,scale,z) in enumerate(examples):
        x,y = (i%4-1.5)*3.2,(1.5-i//4)*2.90
        box(group,"Storm component platform",(x,y,-.114),(2.85,2.48,.21),"storm-slate-edge",.057)
        instance(group,assets[name],name,(x,y+.16,z),scale,.14)
        forest.label(group,name,(x,y-1.06,.014))
    scene.camera = forest.camera(scene,"Storm components",(0,-11.4,24),(0,0,.17),15.0)
    lighting(scene)
    forest.render_setup(scene,1536,1320,samples)
    return scene


def main():
    args = argparse.ArgumentParser()
    args.add_argument("output",type=Path)
    args.add_argument("--samples",type=int,default=20)
    args.add_argument("--draft",action="store_true")
    options = args.parse_args(sys.argv[sys.argv.index("--")+1:])
    out = options.output.resolve()
    out.mkdir(parents=True,exist_ok=True)
    bpy.ops.wm.read_factory_settings(use_empty=True)
    terrain.register_palette()
    scene = bpy.context.scene
    scene.name = ARENA
    assets = kit()
    assemble(scene,assets)
    views = reference_cameras(scene)
    views["Overview"].location = Vector((4.15,-26.2,24.25))
    forest.aim(views["Overview"],(8,2.22,-.29))
    views["Overview"].data.ortho_scale = 35.4
    lighting(scene)
    timeline(scene)
    scene.frame_set(round(OVERVIEW_TIME*FPS)+1)
    scene.camera = views["Overview"]
    forest.render_setup(scene,1740,1050,options.samples)
    board = component_board(assets,options.samples)
    metrics = forest.measure(scene,assets)
    (out/"metrics.json").write_text(json.dumps(metrics,indent=2)+"\n")
    for area in bpy.context.screen.areas:
        if area.type=="VIEW_3D":
            area.spaces.active.region_3d.view_perspective = "CAMERA"
    bpy.ops.wm.save_as_mainfile(filepath=str(out/"storm-valley.blend"),compress=True)

    shots = (
        (scene,"Overview",OVERVIEW_TIME,(1740,1050),"storm-overview"),
        (scene,"Match",OVERVIEW_TIME,(1600,900),"storm-match"),
        (board,None,0,(1536,1320),"storm-kit"),
        (scene,"Mineiro A",24.24,(900,730),"mineiro-a"),
        (scene,"Mineiro B",24.24,(900,730),"mineiro-b"),
        (scene,"Mercador A",OVERVIEW_TIME,(900,730),"mercador-a"),
        (scene,"Mercador B",OVERVIEW_TIME,(900,730),"mercador-b"),
    )
    for shot,view,time,size,filename in shots:
        if options.draft:
            size = tuple(round(d*.53) for d in size)
        render(shot,views[view] if view else board.camera,time,size,
               8 if options.draft else options.samples,out/(filename+".png"))


if __name__=="__main__":
    main()
