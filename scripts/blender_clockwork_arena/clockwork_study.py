"""Assemble the editable Clockwork Bastion and its two animated auxiliary seats."""

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
import clockwork_architecture as architecture
from clockwork_auxiliaries import place
import clockwork_mechanics as mechanics
from clockwork_motion import GEAR_LAYER, place_drive
import clockwork_scenery as scenery
import clockwork_terrain as terrain

ARENA = "01  Clockwork Bastion"
BOARD = "02  Clockwork component board"
ROAD = "02  BattleLine and metal-bound slate curbs"
HOLDS = "03  Clockwork Strongholds"
FIXTURES = "04  Lamps and foundry rails"
BACKDROP = "05  Boilers, pipes and coastal cranes"
UNDERSTORY = "06  Sparse heath and foundry supplies"
COAST = "07  Offshore foundry and supply rafts"
SITES_LAYER = "08  Mineiros and Mercadores"
SCENIC_LAYERS = (FIXTURES,BACKDROP,UNDERSTORY,COAST,GEAR_LAYER)
OVERVIEW_TIME = 24.37


def kit():
    makers = {
        "Foundry soil plateau":terrain.plateau, "Layered foundry cliffs":terrain.cliffs,
        "Cold foundry sea":terrain.sea, "Foundry shallows":terrain.shelf,
        "Slate road foundation":terrain.road_bed, "Metal-bound slate curb":terrain.curb,
        "Clockwork Stronghold":architecture.stronghold, "Clockwork foundry":architecture.foundry,
        "Foundry island foundation":scenery.foundry_islet, "Crane islet":scenery.crane_islet,
        "24-tooth brass gear":lambda group: mechanics.gear(group,24),
        "16-tooth brass gear":lambda group: mechanics.gear(group,16),
        "Horizontal steam boiler":mechanics.boiler, "Copper pipe manifold":mechanics.pipe_cluster,
        "Coastal foundry crane":mechanics.crane, "Riveted supply chest":mechanics.crate,
        "Amber iron lantern":mechanics.lamp, "Foundry iron railing":mechanics.railing,
        "Slate rubble":scenery.rubble, "Sparse foundry grass":scenery.grass,
        "Dry foundry tree":scenery.dead_tree, "Waterlogged supply raft":scenery.supply_raft,
        "Foundry sea ripples":scenery.ripple,
        "Blue Stronghold banners":lambda group: architecture.standard(group,"blue"),
        "Red Stronghold banners":lambda group: architecture.standard(group,"red"),
    }
    for i in range(4):
        makers[f"Clockwork flagstone {i+1}"] = lambda group,i=i: terrain.slab(group,i)
    assets = {name:prototype(name) for name in makers}
    for name,make in makers.items():
        make(assets[name])
    return assets


def assemble(scene,assets):
    rng = Random(705)

    def put(group,name,at,scale=1,angle=0):
        return instance(group,assets[name],name,at,scale,angle)

    place_lane(scene,put,rng)
    place_fixtures(scene,put)
    place_backdrop(scene,put,rng)
    place_understory(scene,put,rng)
    place_coast(scene,put,rng)
    place(forest.group(scene,SITES_LAYER))


def place_lane(scene,put,rng):
    land = forest.group(scene,"01  Earth, foundry cliffs and clear sea")
    road = forest.group(scene,ROAD)
    castles = forest.group(scene,HOLDS)
    for name in ("Foundry soil plateau","Layered foundry cliffs","Cold foundry sea","Foundry shallows"):
        put(land,name,(0,0,0))
    put(road,"Slate road foundation",(LANE_LENGTH/2,0,0))
    for i in range(18):
        x = i-.50
        for j in range(4):
            variant = 1+rng.choices(range(4),(8,3,6,2))[0]
            put(road,f"Clockwork flagstone {variant}",(x+(j%2)*.009,(j-1.5)*1.238,0),
                angle=rng.uniform(-.006,.006))
        for s in (-1,1):
            put(road,"Metal-bound slate curb",(x,s*(LANE_WIDTH/2+.078),0),angle=s*.008)
    for x,seat in zip(HOLD_CENTERS,("Blue","Red"),strict=True):
        put(castles,"Clockwork Stronghold",(x,0,0))
        put(castles,f"{seat} Stronghold banners",(x,0,0))


def place_fixtures(scene,put):
    group = forest.group(scene,FIXTURES)
    for s in (-1,1):
        for x in (1.13,5.22,10.78,14.87):
            put(group,"Amber iron lantern",(x,s*2.99,-.044),.96)
        for x in (2.30,3.46,12.54,13.70):
            put(group,"Foundry iron railing",(x,s*3.11,-.040),angle=s*.031)
        for x in (6.46,9.54):
            put(group,"Riveted supply chest",(x,s*3.08,-.034),.72)


def place_backdrop(scene,put,rng):
    group = forest.group(scene,BACKDROP)
    for x,y,scale,angle in ((2.21,4.39,.94,.15),(5.57,4.96,.88,-.18),
                            (10.52,4.95,.90,.11),(13.79,4.42,.95,-.16)):
        put(group,"Horizontal steam boiler",(x,y,-.025),scale,angle)
    for x,y in ((.29,3.78),(4.07,4.70),(8.03,5.14),(12.10,4.68),(15.69,3.73)):
        put(group,"Copper pipe manifold",(x,y,-.021),rng.uniform(.87,1.19),rng.uniform(-.23,.23))
    for x,y,angle in ((-.53,4.29,-.63),(16.26,4.31,2.45)):
        put(group,"Coastal foundry crane",(x,y,-.023),.91,angle)
    for x,y in ((1.19,4.76),(6.50,5.41),(9.53,5.40),(14.72,4.73)):
        put(group,"Dry foundry tree",(x,y,-.017),rng.uniform(.76,1.04),rng.uniform(-.48,.49))


def place_understory(scene,put,rng):
    group = forest.group(scene,UNDERSTORY)
    for i in range(56):
        x,y = rng.uniform(-1.30,17.30),rng.uniform(3.62,5.50)
        s = 1 if i%3==0 else -1
        coast = ((x-8)/11.8)**2+(y/6.13)**2
        approach = s<0 and y>3.90 and min(abs(x-1.42),abs(x-14.58))<1.15
        if coast>.91 or approach:
            continue
        name = "Slate rubble" if i%5==0 else "Sparse foundry grass"
        put(group,name,(x,s*y,-.047+(y-3.4)*.029),rng.uniform(.68,1.18),rng.uniform(-.40,.42))
    for x,y,scale,angle in ((4.35,-4.60,.88,-.12),(6.50,-5.11,.75,.14),
                            (9.61,-5.14,.77,-.12),(11.84,-4.62,.86,.17)):
        put(group,"Horizontal steam boiler",(x,y,-.028),scale,angle)
    for x,y,scale in ((3.41,-4.19,.70),(7.84,-4.86,.81),(12.72,-4.23,.72)):
        put(group,"Copper pipe manifold",(x,y,-.022),scale,rng.uniform(-.31,.28))


def place_coast(scene,put,rng):
    group = forest.group(scene,COAST)
    level = terrain.WATERLINE
    put(group,"Foundry island foundation",(8.10,9.62,level+.30))
    factory = (8.10,9.72,level+.817)
    put(group,"Clockwork foundry",factory)
    place_drive(put,forest.group(scene,GEAR_LAYER),
                (factory[0],factory[1]-1.180,factory[2]+1.770))
    for x,y,angle in ((-.72,7.82,.18),(16.75,8.10,2.86)):
        put(group,"Crane islet",(x,y,level+.02),angle=angle)
        put(group,"Coastal foundry crane",(x,y,level+.713),.82,angle)
    for x,y,scale,angle in ((-2.29,-5.62,.90,.25),(5.09,-7.48,.83,-.23),
                            (10.79,-7.82,.90,.17),(18.29,-5.38,.85,-.29)):
        put(group,"Waterlogged supply raft",(x,y,level+.020),scale,angle)
    for x,y in ((-4.39,-4.40),(-3.06,-6.50),(4.31,-7.73),(6.60,-8.18),(8.11,-8.85),
                (9.60,-9.27),(11.93,-7.32),(18.15,-6.62),(20.13,-3.93),(-2.17,6.39),
                (1.14,7.26),(3.59,7.67),(12.54,7.67),(14.93,7.35),(18.72,6.94)):
        put(group,"Foundry sea ripples",(x,y,level+.013),rng.uniform(.92,1.47),rng.uniform(-.39,.25))


def lighting(scene):
    forest.lighting(scene)
    sky = scene.world.node_tree.nodes["Background"]
    sky.inputs["Color"].default_value = (.53,.60,.65,1)
    sky.inputs["Strength"].default_value = .52
    sun = next(obj.data for obj in scene.objects if obj.type=="LIGHT" and obj.data.type=="SUN")
    sun.color = (1,.88,.72)
    sun.energy = 2.25
    sun.angle = .20


def component_board(assets,samples):
    scene = bpy.data.scenes.new(BOARD)
    group = forest.group(scene,"Separate Clockwork Bastion components")
    box(group,"Cold foundry display tabletop",(0,0,-.32),(130,130,.26),"cw-iron-dark")
    examples = (
        ("Clockwork Stronghold",.60,0),("Clockwork foundry",.45,0),
        ("24-tooth brass gear",.94,1.06),("16-tooth brass gear",1.24,.90),
        ("Horizontal steam boiler",1.52,0),("Copper pipe manifold",1.42,0),
        ("Coastal foundry crane",.95,0),("Riveted supply chest",1.69,0),
        ("Amber iron lantern",1.71,0),("Foundry iron railing",1.58,0),
        ("Waterlogged supply raft",1.55,.02),("Dry foundry tree",1.06,0),
        ("Slate rubble",1.63,0),("Sparse foundry grass",1.97,0),
        ("Clockwork flagstone 1",1.20,0),("Metal-bound slate curb",1.69,0),
    )
    for i,(name,scale,z) in enumerate(examples):
        x,y = (i%4-1.5)*3.2,(1.5-i//4)*2.90
        box(group,"Clockwork component platform",(x,y,-.114),(2.85,2.48,.21),"cw-rock-dark",.054)
        instance(group,assets[name],name,(x,y+.16,z),scale,.14)
        forest.label(group,name,(x,y-1.06,.014))
    scene.camera = forest.camera(scene,"Clockwork components",(0,-11.4,24),(0,0,.17),15.0)
    lighting(scene)
    forest.render_setup(scene,1536,1320,samples)
    return scene


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("output",type=Path)
    parser.add_argument("--samples",type=int,default=20)
    parser.add_argument("--draft",action="store_true")
    args = parser.parse_args(sys.argv[sys.argv.index("--")+1:])
    out = args.output.resolve()
    out.mkdir(parents=True,exist_ok=True)
    bpy.ops.wm.read_factory_settings(use_empty=True)
    terrain.register_palette()
    scene = bpy.context.scene
    scene.name = ARENA
    assets = kit()
    assemble(scene,assets)
    views = reference_cameras(scene)
    views["Overview"].location = Vector((4.18,-26.1,25.1))
    forest.aim(views["Overview"],(8,1.32,-.29))
    views["Overview"].data.ortho_scale = 34.7
    views["Foundry"] = forest.camera(scene,"Foundry mechanism",(6.91,1.27,5.37),(8.10,9.53,1.31),8.2)
    lighting(scene)
    timeline(scene)
    scene.frame_set(round(OVERVIEW_TIME*FPS)+1)
    scene.camera = views["Overview"]
    forest.render_setup(scene,1740,1050,args.samples)
    board = component_board(assets,args.samples)
    metrics = forest.measure(scene,assets)
    (out/"metrics.json").write_text(json.dumps(metrics,indent=2)+"\n")
    for area in bpy.context.screen.areas:
        if area.type=="VIEW_3D":
            area.spaces.active.region_3d.view_perspective = "CAMERA"
    bpy.ops.wm.save_as_mainfile(filepath=str(out/"clockwork-bastion.blend"),compress=True)

    shots = (
        (scene,"Overview",OVERVIEW_TIME,(1740,1050),"clockwork-overview"),
        (scene,"Match",OVERVIEW_TIME,(1600,900),"clockwork-match"),
        (scene,"Foundry",OVERVIEW_TIME,(1200,900),"clockwork-foundry"),
        (board,None,0,(1536,1320),"clockwork-kit"),
        (scene,"Mineiro A",24.24,(900,730),"mineiro-a"),
        (scene,"Mineiro B",24.24,(900,730),"mineiro-b"),
        (scene,"Mercador A",OVERVIEW_TIME,(900,730),"mercador-a"),
        (scene,"Mercador B",OVERVIEW_TIME,(900,730),"mercador-b"),
    )
    for shot,view,time,size,filename in shots:
        if args.draft:
            size = tuple(round(d*.53) for d in size)
        render(shot,views[view] if view else board.camera,time,size,8 if args.draft else args.samples,
               out/(filename+".png"))


if __name__=="__main__":
    main()
