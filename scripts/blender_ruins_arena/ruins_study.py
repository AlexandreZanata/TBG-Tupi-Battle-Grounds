"""Build the modular Ancient Ruins arena and stage the two offshore seats."""

import argparse
import json
from math import sin, tau
from pathlib import Path
from random import Random
import sys

import bpy
from mathutils import Vector

HERE=Path(__file__).resolve().parent
sys.path[:0]=[str(HERE),str(HERE.parent/"blender_forest_arena"),str(HERE.parent/"blender_tundra_arena")]
import build as forest
from auxiliary_study import timeline
from aux_parts import FPS
from mesh import box, instance, prototype
from terrain import HOLD_CENTERS, LANE_LENGTH, LANE_WIDTH
from tundra_study import cameras as reference_cameras, render
from ruins_auxiliaries import place
import ruins_props as props
import ruins_terrain as terrain

ARENA="01  Ancient Ruins"
BOARD="02  Ruins component board"
ROAD="02  BattleLine and limestone curbs"
HOLDS="03  Ruins Strongholds"
FIXTURES="04  Ritual lamps and balustrades"
BACKDROP="05  Cedar and broken colonnade"
UNDERSTORY="06  Dry moss and fallen masonry"
COAST="07  Lagoon sanctuary and flooded remains"
SITES_LAYER="08  Mineiros and Mercadores"
SCENIC_LAYERS=(FIXTURES,BACKDROP,UNDERSTORY,COAST)
OVERVIEW_TIME=24.37


def kit():
    makers={
        "Dry moss plateau":terrain.plateau, "Limestone cliffs":terrain.cliffs,
        "Turquoise lagoon":terrain.lagoon, "Shallow silt shelf":terrain.shelf,
        "Packed limestone road bed":terrain.lane_bed, "Limestone curb":terrain.curb,
        "Ruins Stronghold":props.stronghold, "Broken colonnade":props.colonnade,
        "Fragmented arch":props.gate_fragment, "Fallen limestone drums":props.collapsed_column,
        "Stone balustrade":props.balustrade, "Turquoise ritual lamp":props.ritual_lamp,
        "Sanctuary islet":props.shrine_islet, "Crest sanctuary":props.sanctuary,
        "Submerged column drum":props.submerged_drum, "Flooded limestone steps":props.flooded_steps,
        "Lagoon lilies":props.lily, "Moss fern":props.fern, "Old cedar":props.fir,
        "Lagoon reed":props.reeds,
        "Blue Stronghold banners":lambda group: props.standard(group,"blue"),
        "Red Stronghold banners":lambda group: props.standard(group,"red"),
    }
    for i in range(4):
        makers[f"Limestone flagstone {i+1}"]=lambda group,i=i: terrain.flagstone(group,i)
    assets={name:prototype(name) for name in makers}
    for name,make in makers.items():
        make(assets[name])
    return assets


def assemble(scene,assets):
    rng=Random(704)

    def put(group,name,at,scale=1,angle=0):
        return instance(group,assets[name],name,at,scale,angle)

    place_lane(scene,put,rng)
    place_fixtures(scene,put)
    place_backdrop(scene,put,rng)
    place_understory(scene,put,rng)
    place_coast(scene,put,rng)
    place(forest.group(scene,SITES_LAYER))


def place_lane(scene,put,rng):
    land=forest.group(scene,"01  Dry moss, limestone and turquoise lagoon")
    road=forest.group(scene,ROAD)
    castles=forest.group(scene,HOLDS)
    for name in ("Dry moss plateau","Limestone cliffs","Turquoise lagoon","Shallow silt shelf"):
        put(land,name,(0,0,0))
    put(road,"Packed limestone road bed",(LANE_LENGTH/2,0,0))
    for i in range(18):
        x=i-.50
        for j in range(4):
            variant=1+rng.choices(range(4),(7,3,6,2))[0]
            put(road,f"Limestone flagstone {variant}",(x+(j%2)*.009,(j-1.5)*1.238,0),
                angle=rng.uniform(-.007,.007))
        for sign in (-1,1):
            put(road,"Limestone curb",(x,sign*(LANE_WIDTH/2+.078),0),angle=sign*.009)
    for x,seat in zip(HOLD_CENTERS,("Blue","Red"),strict=True):
        put(castles,"Ruins Stronghold",(x,0,0))
        put(castles,f"{seat} Stronghold banners",(x,0,0))


def place_fixtures(scene,put):
    group=forest.group(scene,FIXTURES)
    for sign in (-1,1):
        for x in (1.09,5.10,10.90,14.91):
            put(group,"Turquoise ritual lamp",(x,sign*2.96,-.044),.94)
        for x in (2.39,3.56,12.44,13.61):
            put(group,"Stone balustrade",(x,sign*3.11,-.045),angle=sign*.031*sin(x))
        for x in (6.35,9.65):
            put(group,"Fallen limestone drums",(x,sign*3.24,-.041),.55,sign*.22)


def place_backdrop(scene,put,rng):
    group=forest.group(scene,BACKDROP)
    for x,y,scale,angle in ((2.35,4.28,.91,.17),(4.54,4.75,.92,-.16),(11.31,4.76,1.10,.12),
                            (13.49,4.39,.90,-.13)):
        put(group,"Broken colonnade",(x,y,-.018),scale,angle)
    for x,y,scale,angle in ((-.31,3.57,.92,-.31),(6.36,4.47,1.06,.21),(15.57,3.62,1.08,.32)):
        put(group,"Fragmented arch",(x,y,-.035),scale,angle)
    for x,y in ((.37,4.36),(1.29,4.46),(3.20,4.99),(5.66,5.37),(7.11,5.60),
                (9.28,5.53),(10.10,5.18),(12.35,5.22),(14.45,4.70),(15.39,4.55),(17.20,3.74)):
        put(group,"Old cedar",(x,y,-.026),rng.uniform(.70,1.32),rng.random()*tau)


def place_understory(scene,put,rng):
    group=forest.group(scene,UNDERSTORY)
    for i in range(57):
        x,y=rng.uniform(-1.4,17.4),rng.uniform(3.53,5.67)
        side=1 if i%4==0 else -1
        shore=((x-8)/11.8)**2+(y/6.15)**2
        access=side<0 and min(abs(x-1.42),abs(x-14.58))<1.08 and y>3.95
        if shore>.92 or access:
            continue
        name=("Fallen limestone drums","Lagoon reed","Moss fern","Moss fern")[i%4]
        put(group,name,(x,side*y,-.052+(y-3.4)*.028),rng.uniform(.66,1.36),rng.random()*tau)
    for x,y,scale in ((3.95,-4.73,.84),(6.45,-5.31,.70),(9.65,-5.34,.76),(12.25,-4.78,.76)):
        put(group,"Fragmented arch",(x,y,-.035),scale,rng.uniform(-.47,.38))


def place_coast(scene,put,rng):
    group=forest.group(scene,COAST)
    level=terrain.WATERLINE
    put(group,"Sanctuary islet",(8.18,9.67,level+.286),angle=.034)
    put(group,"Crest sanctuary",(8.18,9.67,level+.627),angle=.034)
    for x,y,angle in ((8.03,-6.75,.12),(4.16,7.24,2.38),(11.92,7.02,3.51)):
        put(group,"Flooded limestone steps",(x,y,level+.095),angle=angle)
    for x,y,scale in ((-4.46,-3.04,.80),(-2.26,-5.76,.93),(4.28,-7.58,.92),(7.49,-8.11,1.22),
                      (9.26,-7.33,.94),(11.81,-7.19,.77),(18.81,-4.67,1.03),(20.13,-3.26,.73),
                      (3.31,7.76,.89),(12.78,7.50,.78),(13.51,9.24,1.08)):
        put(group,"Submerged column drum",(x,y,level+.045),scale,rng.random()*tau)
    for x,y in ((-3.67,-4.77),(4.27,-6.55),(5.63,-7.43),(6.42,-8.46),(8.95,-8.57),(10.20,-8.02),
                (11.89,-6.73),(18.79,-5.25),(2.17,7.01),(4.77,7.52),(6.17,7.67),
                (8.82,7.55),(10.62,7.54),(13.74,7.71),(15.79,6.66)):
        put(group,"Lagoon lilies",(x,y,level+.009),rng.uniform(.88,1.43),rng.random()*tau)
    for x,y in ((5.81,8.87),(6.20,9.32),(10.36,8.88),(10.71,9.13),(5.18,9.96),(11.19,9.90)):
        put(group,"Lagoon reed",(x,y,level+.567),rng.uniform(.88,1.27),rng.random()*tau)


def lighting(scene):
    forest.lighting(scene)
    background=scene.world.node_tree.nodes["Background"]
    background.inputs["Color"].default_value=(.43,.57,.55,1)
    background.inputs["Strength"].default_value=.52
    sun=next(obj.data for obj in scene.objects if obj.type=="LIGHT" and obj.data.type=="SUN")
    sun.color=(1,.89,.72)
    sun.energy=2.46


def component_board(assets,samples):
    scene=bpy.data.scenes.new(BOARD)
    group=forest.group(scene,"Individual Ancient Ruins asset samples")
    box(group,"Dark jade studio tabletop",(0,0,-.32),(130,130,.26),"ruin-jade-dark")
    examples=(("Ruins Stronghold",.62),("Crest sanctuary",.51),("Broken colonnade",1.23),
              ("Fragmented arch",1.28),("Fallen limestone drums",1.56),("Stone balustrade",1.60),
              ("Turquoise ritual lamp",1.65),("Old cedar",1.12),("Flooded limestone steps",1.05),
              ("Submerged column drum",1.72),("Lagoon lilies",1.78),("Lagoon reed",1.85),
              ("Moss fern",1.92),("Limestone flagstone 1",1.24),
              ("Limestone flagstone 4",1.24),("Limestone curb",1.81))
    for i,(name,scale) in enumerate(examples):
        x,y=(i%4-1.5)*3.2,(1.5-i//4)*2.90
        box(group,"Component review platform",(x,y,-.114),(2.85,2.48,.21),"ruin-shadow",.058)
        z=.20 if name=="Submerged column drum" else .155 if name=="Flooded limestone steps" else 0
        instance(group,assets[name],name,(x,y+.16,z),scale,.14)
        forest.label(group,name,(x,y-1.06,.014))
    scene.camera=forest.camera(scene,"Ruins component board",(0,-11.4,24),(0,0,.17),15.0)
    lighting(scene)
    forest.render_setup(scene,1536,1320,samples)
    return scene


def main():
    args=argparse.ArgumentParser()
    args.add_argument("output",type=Path)
    args.add_argument("--samples",type=int,default=20)
    args.add_argument("--draft",action="store_true")
    options=args.parse_args(sys.argv[sys.argv.index("--")+1:])
    out=options.output.resolve()
    out.mkdir(parents=True,exist_ok=True)
    bpy.ops.wm.read_factory_settings(use_empty=True)
    terrain.register_palette()
    scene=bpy.context.scene
    scene.name=ARENA
    assets=kit()
    assemble(scene,assets)
    views=reference_cameras(scene)
    views["Overview"].location=Vector((4.50,-26.5,25.2))
    forest.aim(views["Overview"],(8,1.24,-.26))
    views["Overview"].data.ortho_scale=34.0
    lighting(scene)
    timeline(scene)
    scene.frame_set(round(OVERVIEW_TIME*FPS)+1)
    scene.camera=views["Overview"]
    forest.render_setup(scene,1740,1050,options.samples)
    board=component_board(assets,options.samples)
    metrics=forest.measure(scene,assets)
    (out/"metrics.json").write_text(json.dumps(metrics,indent=2)+"\n")
    for area in bpy.context.screen.areas:
        if area.type=="VIEW_3D":
            area.spaces.active.region_3d.view_perspective="CAMERA"
    bpy.ops.wm.save_as_mainfile(filepath=str(out/"ancient-ruins.blend"),compress=True)

    shots=(
        (scene,"Overview",OVERVIEW_TIME,(1740,1050),"ruins-overview"),
        (scene,"Match",OVERVIEW_TIME,(1600,900),"ruins-match"),
        (board,None,0,(1536,1320),"ruins-kit"),
        (scene,"Mineiro A",24.24,(900,730),"mineiro-a"),
        (scene,"Mineiro B",24.24,(900,730),"mineiro-b"),
        (scene,"Mercador A",OVERVIEW_TIME,(900,730),"mercador-a"),
        (scene,"Mercador B",OVERVIEW_TIME,(900,730),"mercador-b"),
    )
    for shot,view,time,size,filename in shots:
        if options.draft:
            size=tuple(round(d*.53) for d in size)
        render(shot,views[view] if view else board.camera,time,size,
               8 if options.draft else options.samples,out/(filename+".png"))


if __name__=="__main__":
    main()
