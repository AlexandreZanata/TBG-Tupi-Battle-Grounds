"""Assemble Desert Relics from separate assets and stage both auxiliary seats."""

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
import desert_architecture as architecture
from desert_auxiliaries import place
import desert_scenery as scenery
import desert_terrain as terrain

ARENA = "01  Desert Relics"
BOARD = "02  Desert Relics component board"
ROAD = "02  BattleLine and sandstone curbs"
HOLDS = "03  Desert Strongholds"
FIXTURES = "04  Oasis lanterns and carved rails"
BACKDROP = "05  Palms and pointed obelisks"
UNDERSTORY = "06  Sand drifts, pottery and oasis grass"
COAST = "07  Sun temple and sandstone skerries"
SITES_LAYER = "08  Mineiros and Mercadores"
SCENIC_LAYERS = (FIXTURES,BACKDROP,UNDERSTORY,COAST)
OVERVIEW_TIME = 24.37


def kit():
    makers = {
        "Desert sand plateau":terrain.plateau, "Sandstone coastal cliffs":terrain.cliffs,
        "Blue desert sea":terrain.sea, "Sunlit sand shallows":terrain.shelf,
        "Packed sand road bed":terrain.road_bed, "Sandstone curb":terrain.curb,
        "Desert Stronghold":architecture.stronghold, "Stepped sun temple":architecture.temple,
        "Reclining sandstone guardian":architecture.guardian, "Pointed sandstone obelisk":architecture.obelisk,
        "Tall coastal palm":scenery.palm, "Young oasis palm":lambda group:scenery.palm(group,young=True),
        "Open terracotta jar":scenery.pot, "Woven cloth and pottery":scenery.pottery_group,
        "Weathered sandstone":scenery.sandstone_outcrop, "Low sand drift":scenery.dune,
        "Sparse oasis grass":scenery.reeds, "Sun temple island":scenery.temple_islet,
        "Sandstone skerry":scenery.coastal_skerry, "Desert sea ripples":scenery.ripple,
        "Oasis lantern":scenery.oasis_lamp, "Turquoise stone balustrade":scenery.balustrade,
        "Blue Stronghold banners":lambda group:architecture.standard(group,"blue"),
        "Red Stronghold banners":lambda group:architecture.standard(group,"red"),
    }
    for i in range(4):
        makers[f"Desert flagstone {i+1}"] = lambda group,i=i:terrain.slab(group,i)
    assets = {name:prototype(name) for name in makers}
    for name,make in makers.items():
        make(assets[name])
    return assets


def assemble(scene,assets):
    rng = Random(808)

    def put(group,name,at,scale=1,angle=0):
        return instance(group,assets[name],name,at,scale,angle)

    place_lane(scene,put,rng)
    place_fixtures(scene,put)
    place_backdrop(scene,put,rng)
    place_understory(scene,put,rng)
    place_coast(scene,put,rng)
    place(forest.group(scene,SITES_LAYER))


def place_lane(scene,put,rng):
    land = forest.group(scene,"01  Desert sand, sandstone cliffs and blue sea")
    road = forest.group(scene,ROAD)
    castles = forest.group(scene,HOLDS)
    for name in ("Desert sand plateau","Sandstone coastal cliffs","Blue desert sea","Sunlit sand shallows"):
        put(land,name,(0,0,0))
    put(road,"Packed sand road bed",(LANE_LENGTH/2,0,0))
    for i in range(18):
        x = i-.50
        for j in range(4):
            variant = 1+rng.choices(range(4),(8,3,6,2))[0]
            put(road,f"Desert flagstone {variant}",(x+(j%2)*.009,(j-1.5)*1.238,0),
                angle=rng.uniform(-.006,.006))
        for side in (-1,1):
            put(road,"Sandstone curb",(x,side*(LANE_WIDTH/2+.077),0),angle=side*.008)
    for x,seat in zip(HOLD_CENTERS,("Blue","Red"),strict=True):
        put(castles,"Desert Stronghold",(x,0,0))
        put(castles,f"{seat} Stronghold banners",(x,0,0))


def place_fixtures(scene,put):
    group = forest.group(scene,FIXTURES)
    for side in (-1,1):
        for x in (1.11,5.24,10.76,14.89):
            put(group,"Oasis lantern",(x,side*2.99,-.041),.95)
        for x in (2.28,3.45,12.55,13.72):
            put(group,"Turquoise stone balustrade",(x,side*3.11,-.039),angle=side*.029)
        for x in (6.44,9.56):
            put(group,"Woven cloth and pottery",(x,side*3.16,-.030),.79)


def place_backdrop(scene,put,rng):
    group = forest.group(scene,BACKDROP)
    for x,y,scale in ((1.53,4.11,.92),(5.32,4.68,.99),(10.68,4.69,.98),(14.51,4.10,.91)):
        put(group,"Pointed sandstone obelisk",(x,y,-.025),scale,rng.uniform(-.14,.14))
    for x,y in ((.22,4.27),(3.14,4.86),(4.21,5.32),(7.03,5.59),(9.09,5.53),
                (11.89,5.26),(12.82,4.90),(15.74,4.28)):
        put(group,"Tall coastal palm",(x,y,-.018),rng.uniform(.84,1.16),rng.uniform(-.47,.48))
    for x,y in ((2.28,3.83),(6.26,4.47),(9.84,4.46),(13.72,3.84),(16.72,3.61)):
        put(group,"Young oasis palm",(x,y,-.023),rng.uniform(.79,.93),rng.uniform(-.59,.49))


def place_understory(scene,put,rng):
    group = forest.group(scene,UNDERSTORY)
    for i in range(61):
        x,y = rng.uniform(-1.33,17.33),rng.uniform(3.63,5.51)
        side = 1 if i%3==0 else -1
        shore = ((x-8)/11.8)**2+(y/6.13)**2
        approach = side<0 and y>3.90 and min(abs(x-1.42),abs(x-14.58))<1.16
        if shore>.91 or approach:
            continue
        name = ("Weathered sandstone","Low sand drift","Sparse oasis grass","Sparse oasis grass")[i%4]
        put(group,name,(x,side*y,-.048+(y-3.4)*.028),rng.uniform(.69,1.15),rng.uniform(-.42,.39))
    for x,y,scale,angle in ((4.29,-4.69,.96,-.17),(11.82,-4.66,.98,.19)):
        put(group,"Young oasis palm",(x,y,-.022),scale,angle)
    for x,y,scale in ((6.19,-4.93,.90),(9.77,-5.02,.87),(3.52,-4.17,.73),(12.59,-4.14,.74)):
        put(group,"Woven cloth and pottery",(x,y,-.027),scale,rng.uniform(-.28,.31))


def place_coast(scene,put,rng):
    group = forest.group(scene,COAST)
    level = terrain.WATERLINE
    put(group,"Sun temple island",(8.10,9.69,level+.302))
    put(group,"Stepped sun temple",(8.10,9.91,level+.789))
    for x,y,angle in ((5.94,8.44,.11),(10.26,8.44,-.11)):
        put(group,"Reclining sandstone guardian",(x,y,level+.787),1.30,angle)
    for x,y,scale in ((4.99,9.70,.77),(11.17,9.70,.77)):
        put(group,"Pointed sandstone obelisk",(x,y,level+.785),scale)
    for x,y,scale,angle in ((-.82,7.79,1.18,.13),(16.89,8.10,1.15,-.17),(8.12,-8.67,1.10,.19)):
        put(group,"Sandstone skerry",(x,y,level+.013),scale,angle)
    put(group,"Young oasis palm",(8.21,-8.58,level+.595),.73,.18)
    for x,y,scale,angle in ((-2.28,-5.43,.94,.20),(4.65,-7.62,.96,-.25),
                            (11.38,-7.51,.94,.12),(18.27,-5.38,.97,-.22)):
        put(group,"Sandstone skerry",(x,y,level+.015),scale,angle)
    for x,y,scale in ((5.24,10.16,.84),(10.93,10.21,.84)):
        put(group,"Young oasis palm",(x,y,level+.785),scale)
    for x,y in ((-4.28,-4.49),(-3.01,-6.48),(4.04,-7.79),(6.16,-8.31),(7.38,-10.07),
                (9.45,-9.73),(11.99,-7.46),(18.15,-6.52),(20.13,-3.98),(-1.95,6.56),
                (1.28,7.09),(3.88,7.68),(12.45,7.67),(14.81,7.22),(18.73,6.81)):
        put(group,"Desert sea ripples",(x,y,level+.014),rng.uniform(.91,1.46),rng.uniform(-.40,.27))


def lighting(scene):
    forest.lighting(scene)
    sky = scene.world.node_tree.nodes["Background"]
    sky.inputs["Color"].default_value = (.57,.66,.75,1)
    sky.inputs["Strength"].default_value = .49
    sun = next(obj.data for obj in scene.objects if obj.type=="LIGHT" and obj.data.type=="SUN")
    sun.color = (1,.89,.72)
    sun.energy = 2.40
    sun.angle = .18


def component_board(assets,samples):
    scene = bpy.data.scenes.new(BOARD)
    group = forest.group(scene,"Separate Desert Relics component samples")
    box(group,"Deep turquoise desert studio tabletop",(0,0,-.32),(130,130,.26),"dr-turquoise-dark")
    examples = (
        ("Desert Stronghold",.60,0),("Stepped sun temple",.44,0),
        ("Reclining sandstone guardian",1.49,.32),("Pointed sandstone obelisk",1.05,0),
        ("Tall coastal palm",.86,0),("Young oasis palm",1.16,0),
        ("Open terracotta jar",2.03,0),("Woven cloth and pottery",1.66,0),
        ("Weathered sandstone",1.56,0),("Low sand drift",1.44,.02),
        ("Sparse oasis grass",2.02,0),("Turquoise stone balustrade",1.58,0),
        ("Oasis lantern",1.74,0),("Sandstone skerry",1.19,.10),
        ("Desert flagstone 1",1.20,0),("Sandstone curb",1.69,0),
    )
    for i,(name,scale,z) in enumerate(examples):
        x,y = (i%4-1.5)*3.2,(1.5-i//4)*2.90
        box(group,"Desert component platform",(x,y,-.114),(2.85,2.48,.21),"dr-shadow",.055)
        instance(group,assets[name],name,(x,y+.16,z),scale,.14)
        forest.label(group,name,(x,y-1.06,.014))
    scene.camera = forest.camera(scene,"Desert components",(0,-11.4,24),(0,0,.17),15.0)
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
    views["Overview"].location = Vector((4.22,-26.2,25.15))
    forest.aim(views["Overview"],(8,1.37,-.30))
    views["Overview"].data.ortho_scale = 34.7
    views["Temple"] = forest.camera(scene,"Sun temple and guardians",(7.00,5.80,5.90),(8.10,9.70,1.00),8.5)
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
    bpy.ops.wm.save_as_mainfile(filepath=str(out/"desert-relics.blend"),compress=True)
    shots = (
        (scene,"Overview",OVERVIEW_TIME,(1740,1050),"desert-overview"),
        (scene,"Match",OVERVIEW_TIME,(1600,900),"desert-match"),
        (scene,"Temple",OVERVIEW_TIME,(1200,900),"desert-temple"),
        (board,None,0,(1536,1320),"desert-kit"),
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
