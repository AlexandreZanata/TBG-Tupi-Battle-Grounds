"""Build the editable Volcanic Forge and stage both auxiliary seats."""

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
from forge_auxiliaries import place
import forge_props as props
import forge_terrain as terrain

ARENA="01  Volcanic Forge"
BOARD="02  Forge component board"
ROAD="02  BattleLine and basalt curbs"
HOLDS="03  Forge Strongholds"
FIXTURES="04  Forged lanterns and fittings"
RIDGE="05  Basalt organ pipes"
UNDERSTORY="06  Ash, dry branches and seams"
COAST="07  Caldera and lava channels"
SITES_LAYER="08  Mineiros and Mercadores"
SCENIC_LAYERS=(FIXTURES,RIDGE,UNDERSTORY,COAST)
OVERVIEW_TIME=24.37


def kit():
    makers={
        "Ash plateau":terrain.plateau, "Basalt escarpment":terrain.cliff, "Lava basin":terrain.sea,
        "Cooling shoreline":terrain.coast, "Charcoal road bed":terrain.lane_bed,
        "Basalt curb":terrain.shoulder, "Forge Stronghold":props.stronghold,
        "Basalt organ pipes":props.basalt_pipes, "Cooled slag":props.slag,
        "Ember fissure":props.fissure, "Scorched branch":props.scorched_branch,
        "Ash grass":props.grass, "Iron railing":props.rail, "Forge brazier":props.brazier,
        "Forged anvil":props.anvil, "Drifting slag crust":props.crust, "Mineral vent":props.vent,
        "Open caldera":props.crater,
        "Blue Stronghold banners":lambda group: props.standard(group,"blue"),
        "Red Stronghold banners":lambda group: props.standard(group,"red"),
    }
    for i in range(4):
        makers[f"Charcoal flagstone {i+1}"]=lambda group,i=i: terrain.flagstone(group,i)
    assets={name:prototype(name) for name in makers}
    for name,make in makers.items():
        make(assets[name])
    return assets


def assemble(scene,assets):
    rng=Random(401)

    def put(group,name,at,scale=1,angle=0):
        return instance(group,assets[name],name,at,scale,angle)

    place_lane(scene,put,rng)
    place_fittings(scene,put)
    place_ridge(scene,put,rng)
    place_understory(scene,put,rng)
    place_offshore(scene,put,rng)
    place(forest.group(scene,SITES_LAYER))


def place_lane(scene,put,rng):
    land=forest.group(scene,"01  Ash, basalt and cooling crust")
    road=forest.group(scene,ROAD)
    castles=forest.group(scene,HOLDS)
    for name in ("Ash plateau","Basalt escarpment","Lava basin","Cooling shoreline"):
        put(land,name,(0,0,0))
    put(road,"Charcoal road bed",(LANE_LENGTH/2,0,0))
    for i in range(18):
        x=i-.50
        for j in range(4):
            variant=1+rng.choices(range(4),(8,2,4,1))[0]
            put(road,f"Charcoal flagstone {variant}",(x+(j%2)*.009,(j-1.5)*1.238,0),
                angle=rng.uniform(-.007,.007))
        for sign in (-1,1):
            put(road,"Basalt curb",(x,sign*(LANE_WIDTH/2+.085),0),angle=sign*.010)
    for x,seat in zip(HOLD_CENTERS,("Blue","Red"),strict=True):
        put(castles,"Forge Stronghold",(x,0,0))
        put(castles,f"{seat} Stronghold banners",(x,0,0))


def place_fittings(scene,put):
    group=forest.group(scene,FIXTURES)
    for sign in (-1,1):
        for x in (1.01,5.16,10.84,14.99):
            put(group,"Forge brazier",(x,sign*2.98,-.012),.92)
        for x in (2.42,3.58,12.42,13.58):
            put(group,"Iron railing",(x,sign*3.12,-.029),angle=sign*.027*sin(x))
        for x in (6.39,9.61):
            put(group,"Forged anvil",(x,sign*3.07,-.021),.93,sign*.20)


def place_ridge(scene,put,rng):
    group=forest.group(scene,RIDGE)
    for i in range(12):
        x=-1.48+i*1.73
        y=4.52+1.03*sin((x+2.4)*.139)+rng.uniform(-.17,.18)
        put(group,"Basalt organ pipes",(x,y,.018),rng.uniform(.79,1.33),rng.random()*tau)
    for i in range(8):
        x=.41+i*2.14
        name="Scorched branch" if i%2 else "Cooled slag"
        put(group,name,(x,3.71+.22*sin(i*.8),-.018),rng.uniform(.62,.94),rng.random()*tau)


def place_understory(scene,put,rng):
    group=forest.group(scene,UNDERSTORY)
    for i in range(44):
        x,y=rng.uniform(-1.42,17.42),rng.uniform(3.42,5.62)
        side=1 if i%4==0 else -1
        shore=((x-8)/11.78)**2+(y/6.15)**2
        access=side<0 and min(abs(x-1.42),abs(x-14.58))<1.05 and y>3.96
        if shore>.92 or access:
            continue
        name="Cooled slag" if i%5==0 else "Ash grass"
        put(group,name,(x,side*y,-.020+(y-3.41)*.034),rng.uniform(.63,1.06),rng.random()*tau)
    for x,y,s in ((3.77,-4.92,.83),(5.91,-5.39,1.02),(7.52,-4.75,1.30),
                  (10.62,-5.46,1.08),(12.30,-4.73,.97),(4.14,3.85,.81),(11.99,3.91,.83)):
        put(group,"Ember fissure",(x,y,.033),s,rng.uniform(-.31,.28))
    for x,y,s in ((3.38,-4.22,.76),(6.74,-4.96,.58),(9.26,-4.96,.59),(12.62,-4.23,.76)):
        put(group,"Basalt organ pipes",(x,y,.019),s,rng.random()*tau)


def place_offshore(scene,put,rng):
    group=forest.group(scene,COAST)
    put(group,"Open caldera",(8.1,11.36,terrain.LAVA_LEVEL+.075),(1.56,1.61,1.40),.10)
    for x,y,s in ((-.64,8.48,.91),(17.65,8.73,.93),(2.01,10.55,.80),(13.97,10.65,.79)):
        put(group,"Mineral vent",(x,y,terrain.LAVA_LEVEL+.04),s,rng.random()*tau)
        put(group,"Basalt organ pipes",(x+1.23,y+.11,terrain.LAVA_LEVEL-.02),s,rng.random()*tau)
    for x,y,s in ((-5.62,-2.02,.94),(-3.81,-5.36,1.14),(4.23,-7.54,1.08),(6.73,-8.39,.76),
                  (8.37,-8.03,.91),(10.85,-7.54,.93),(19.83,-5.24,1.12),(21.66,-1.89,.90),
                  (-4.37,5.63,.80),(21.37,5.45,.86),(5.09,8.23,.79),(10.97,8.22,.86)):
        put(group,"Drifting slag crust",(x,y,terrain.LAVA_LEVEL+.061),s,rng.random()*tau)


def lighting(scene):
    forest.lighting(scene)
    background=scene.world.node_tree.nodes["Background"]
    background.inputs["Color"].default_value=(.54,.40,.51,1)
    background.inputs["Strength"].default_value=.47
    sun=next(obj.data for obj in scene.objects if obj.type=="LIGHT" and obj.data.type=="SUN")
    sun.color=(1,.83,.72)
    sun.energy=2.32


def component_board(assets,samples):
    scene=bpy.data.scenes.new(BOARD)
    group=forest.group(scene,"Individual forge asset samples")
    box(group,"Dark basalt studio tabletop",(0,0,-.32),(130,130,.26),"basalt-deep")
    examples=(("Forge Stronghold",.59),("Open caldera",.39),("Basalt organ pipes",1.26),
              ("Mineral vent",1.20),("Scorched branch",1.52),("Forge brazier",1.50),
              ("Forged anvil",1.75),("Iron railing",1.54),("Cooled slag",1.49),
              ("Ember fissure",1.50),("Ash grass",1.82),("Drifting slag crust",1.40),
              ("Charcoal flagstone 1",1.23),("Charcoal flagstone 2",1.23),
              ("Basalt curb",1.67),("Charcoal flagstone 4",1.23))
    for i,(name,scale) in enumerate(examples):
        x,y=(i%4-1.5)*3.2,(1.5-i//4)*2.90
        box(group,"Component review platform",(x,y,-.113),(2.85,2.48,.21),"basalt-cut",.055)
        instance(group,assets[name],name,(x,y+.16,0),scale,.14)
        forest.label(group,name,(x,y-1.06,.014))
    scene.camera=forest.camera(scene,"Forge component board",(0,-11.4,24),(0,0,.17),15.0)
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
    views["Overview"].location=Vector((4.42,-25.6,24.7))
    forest.aim(views["Overview"],(8,1.48,-.25))
    views["Overview"].data.ortho_scale=34.7
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
    bpy.ops.wm.save_as_mainfile(filepath=str(out/"volcanic-forge.blend"),compress=True)

    shots=(
        (scene,"Overview",OVERVIEW_TIME,(1740,1050),"forge-overview"),
        (scene,"Match",OVERVIEW_TIME,(1600,900),"forge-match"),
        (board,None,0,(1536,1320),"forge-kit"),
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
