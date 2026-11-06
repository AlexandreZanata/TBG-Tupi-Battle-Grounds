"""Assemble the Celestial Citadel kit, scenes and both animated auxiliary seats."""

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
import celestial_architecture as architecture
from celestial_auxiliaries import place
import celestial_scenery as scenery
import celestial_terrain as terrain

ARENA = "01  Celestial Citadel"
BOARD = "02  Celestial Citadel component board"
ROAD = "02  BattleLine and ivory curbs"
HOLDS = "03  Celestial Strongholds"
FIXTURES = "04  Gilt fixtures and garden rails"
BACKDROP = "05  Cypress and crystal shrines"
UNDERSTORY = "06  Flowering gardens and planters"
COAST = "07  Citadel islet and waterfalls"
SITES_LAYER = "08  Mineiros and Mercadores"
SCENIC_LAYERS = (FIXTURES,BACKDROP,UNDERSTORY,COAST)
OVERVIEW_TIME = 24.37


def kit():
    makers = {
        "High garden plateau":terrain.plateau, "Ivory coastal terraces":terrain.cliffs,
        "Clear celestial sea":terrain.sea, "Sunlit shore shelf":terrain.shelf,
        "Limestone road foundation":terrain.road_bed, "Ivory curb":terrain.curb,
        "Celestial Stronghold":architecture.stronghold, "Celestial palace":architecture.palace,
        "Ceremonial guardian":architecture.guardian, "Gilt garden arch":architecture.celestial_arch,
        "Citadel island foundation":scenery.citadel_islet, "Waterfall islet":scenery.waterfall_islet,
        "Azure cascade":scenery.waterfall, "Crystal sanctuary":scenery.crystal_garden,
        "Slender cypress":scenery.cypress, "Flowering shrub":scenery.flowering_shrub,
        "Ivory garden planter":scenery.planter, "Gilt azure lantern":scenery.lantern,
        "Garden balustrade":scenery.balustrade, "Sea ripples":scenery.ripple,
        "Crystal beacon":scenery.beacon,
        "Blue Stronghold banners":lambda group: architecture.standard(group,"blue"),
        "Red Stronghold banners":lambda group: architecture.standard(group,"red"),
    }
    for i in range(4):
        makers[f"Celestial flagstone {i+1}"] = lambda group,i=i: terrain.slab(group,i)
    assets = {name:prototype(name) for name in makers}
    for name,make in makers.items():
        make(assets[name])
    return assets


def assemble(scene,assets):
    rng = Random(663)

    def put(group,name,at,scale=1,angle=0):
        return instance(group,assets[name],name,at,scale,angle)

    place_lane(scene,put,rng)
    place_fixtures(scene,put)
    place_backdrop(scene,put,rng)
    place_gardens(scene,put,rng)
    place_coast(scene,put,rng)
    place(forest.group(scene,SITES_LAYER))


def place_lane(scene,put,rng):
    land = forest.group(scene,"01  Garden plateau, pale cliffs and clear sea")
    road = forest.group(scene,ROAD)
    castles = forest.group(scene,HOLDS)
    for name in ("High garden plateau","Ivory coastal terraces","Clear celestial sea","Sunlit shore shelf"):
        put(land,name,(0,0,0))
    put(road,"Limestone road foundation",(LANE_LENGTH/2,0,0))
    for i in range(18):
        x = i-.50
        for j in range(4):
            variant = 1+rng.choices(range(4),(8,3,6,2))[0]
            put(road,f"Celestial flagstone {variant}",(x+(j%2)*.009,(j-1.5)*1.238,0),
                angle=rng.uniform(-.006,.006))
        for s in (-1,1):
            put(road,"Ivory curb",(x,s*(LANE_WIDTH/2+.078),0),angle=s*.008)
    for x,seat in zip(HOLD_CENTERS,("Blue","Red"),strict=True):
        put(castles,"Celestial Stronghold",(x,0,0))
        put(castles,f"{seat} Stronghold banners",(x,0,0))


def place_fixtures(scene,put):
    group = forest.group(scene,FIXTURES)
    for s in (-1,1):
        for x in (1.13,5.27,10.73,14.87):
            put(group,"Gilt azure lantern",(x,s*2.99,-.042),.95)
        for x in (2.28,3.47,12.53,13.72):
            put(group,"Garden balustrade",(x,s*3.12,-.041),angle=s*.033)
        for x in (6.46,9.54):
            put(group,"Crystal sanctuary",(x,s*3.22,-.035),.82)


def place_backdrop(scene,put,rng):
    group = forest.group(scene,BACKDROP)
    for x,y,scale,angle in ((2.46,4.63,.97,.13),(6.08,4.98,.92,.11),
                            (10.05,5.00,.95,-.06),(13.72,4.63,.98,-.11)):
        put(group,"Gilt garden arch",(x,y,-.022),scale,angle)
    for x,y in ((.38,4.27),(1.12,4.43),(3.36,4.98),(4.73,5.37),(7.19,5.63),(8.91,5.59),
                (11.48,5.23),(12.55,4.92),(14.77,4.51),(15.48,4.47),(17.03,3.68)):
        put(group,"Slender cypress",(x,y,-.016),rng.uniform(.80,1.23),rng.uniform(-.49,.44))
    for x,y in ((.42,3.64),(4.39,4.19),(7.98,4.72),(11.71,4.18),(15.57,3.68)):
        put(group,"Crystal beacon",(x,y,-.029),rng.uniform(.83,1.07))


def place_gardens(scene,put,rng):
    group = forest.group(scene,UNDERSTORY)
    for i in range(58):
        x,y = rng.uniform(-1.33,17.33),rng.uniform(3.62,5.49)
        s = 1 if i%3==0 else -1
        coast = ((x-8)/11.80)**2+(y/6.13)**2
        approach = s<0 and y>3.90 and min(abs(x-1.42),abs(x-14.58))<1.14
        if coast>.90 or approach:
            continue
        name = "Ivory garden planter" if i%5==0 else "Flowering shrub"
        put(group,name,(x,s*y,-.055+(y-3.44)*.028),rng.uniform(.66,1.06),rng.uniform(-.41,.43))
    for x,y,scale in ((4.02,-4.56,.94),(6.33,-5.27,1.15),(9.68,-5.28,1.12),(12.08,-4.54,.97)):
        put(group,"Crystal sanctuary",(x,y,-.022),scale,rng.uniform(-.28,.32))


def place_coast(scene,put,rng):
    group = forest.group(scene,COAST)
    level = terrain.WATERLINE
    put(group,"Citadel island foundation",(8.12,9.64,level+.30),angle=.025)
    put(group,"Celestial palace",(8.12,9.88,level+.881),angle=.025)
    for x,y,angle in ((5.97,8.48,.17),(10.31,8.55,-.15)):
        put(group,"Ceremonial guardian",(x,y,level+.886),1.18,angle)
    for x,y,scale,angle in ((-.42,8.07,1.05,.12),(16.39,8.35,1.08,-.20),
                            (8.03,-8.41,.79,-.11)):
        root = (x,y,level+.014)
        put(group,"Waterfall islet",root,scale,angle)
        put(group,"Azure cascade",root,scale,angle)
        put(group,"Crystal beacon",(x+.47,y+.27,root[2]+scale*1.32),scale*.59,angle)
    for x,y in ((5.31,9.34),(10.89,9.40),(6.05,10.52),(10.51,10.47)):
        put(group,"Slender cypress",(x,y,level+.892),rng.uniform(.63,.84))
    for x,y in ((-4.27,-4.56),(-2.10,-6.32),(4.02,-7.59),(5.72,-8.85),(6.87,-10.28),(9.51,-9.67),
                (11.17,-7.83),(18.29,-6.43),(20.17,-3.97),(-1.94,6.76),(1.15,7.05),(3.94,7.80),
                (12.14,7.59),(14.86,7.38),(17.35,7.14),(19.15,6.32)):
        put(group,"Sea ripples",(x,y,level+.013),rng.uniform(.92,1.45),rng.uniform(-.43,.24))


def lighting(scene):
    forest.lighting(scene)
    sky = scene.world.node_tree.nodes["Background"]
    sky.inputs["Color"].default_value = (.55,.64,.76,1)
    sky.inputs["Strength"].default_value = .48
    sun = next(obj.data for obj in scene.objects if obj.type=="LIGHT" and obj.data.type=="SUN")
    sun.color = (1,.92,.79)
    sun.energy = 2.29
    sun.angle = .18


def component_board(assets,samples):
    scene = bpy.data.scenes.new(BOARD)
    group = forest.group(scene,"Separate Celestial Citadel component samples")
    box(group,"Deep blue display tabletop",(0,0,-.32),(130,130,.26),"cel-azure-deep")
    examples = (
        ("Celestial Stronghold",.61,0),("Celestial palace",.48,0),("Ceremonial guardian",1.30,0),
        ("Gilt garden arch",1.17,0),("Waterfall islet",.73,.10),("Azure cascade",.78,0),
        ("Crystal sanctuary",1.45,0),("Crystal beacon",1.43,0),("Slender cypress",1.08,0),
        ("Flowering shrub",1.80,0),("Ivory garden planter",1.68,0),("Garden balustrade",1.57,0),
        ("Gilt azure lantern",1.72,0),("Celestial flagstone 1",1.20,0),
        ("Celestial flagstone 4",1.20,0),("Ivory curb",1.71,0),
    )
    for i,(name,scale,z) in enumerate(examples):
        x,y = (i%4-1.5)*3.2,(1.5-i//4)*2.90
        box(group,"Celestial component platform",(x,y,-.114),(2.85,2.48,.21),"cel-shadow",.054)
        offset = .60 if name=="Azure cascade" else .16
        instance(group,assets[name],name,(x,y+offset,z),scale,.14)
        forest.label(group,name,(x,y-1.06,.014))
    scene.camera = forest.camera(scene,"Celestial components",(0,-11.4,24),(0,0,.17),15.0)
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
    views["Overview"].location = Vector((4.28,-26.2,25.3))
    forest.aim(views["Overview"],(8,1.30,-.29))
    views["Overview"].data.ortho_scale = 34.65
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
    bpy.ops.wm.save_as_mainfile(filepath=str(out/"celestial-citadel.blend"),compress=True)

    shots = (
        (scene,"Overview",OVERVIEW_TIME,(1740,1050),"celestial-overview"),
        (scene,"Match",OVERVIEW_TIME,(1600,900),"celestial-match"),
        (board,None,0,(1536,1320),"celestial-kit"),
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
