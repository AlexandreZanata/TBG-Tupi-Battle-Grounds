"""Build the basalt platform and lava shoreline around the reference BattleLine."""

from math import cos, sin
from random import Random

from mesh import PALETTE, box, color, material, pebble, surface
from terrain import CENTER, LANE_LENGTH, LANE_WIDTH, rim, rows

LAVA_LEVEL=-1.28
FORGE={
    "ash": "514750", "ash-light": "68585a", "ash-dark": "39333e",
    "basalt": "3f3d47", "basalt-deep": "2d2934", "basalt-cut": "70616a",
    "cooled-slag": "4d323a", "sintered-crust": "6c443d",
    "forge-slate": "64606a", "forge-slate-light": "77717b", "forge-slate-dark": "4d4a55",
    "forge-bed": "342e37", "forge-curb": "8a716b",
    "foundry-copper": "ae674c", "foundry-brass": "c69a66", "foundry-iron": "484451",
    "tempered-wood": "74514a", "charred-timber": "51413e", "hot-grain": "93654f",
    "kiln-canvas": "6c4848", "ash-linen": "cbb59f",
    "volcanic-ore": "a2513e", "volcanic-ore-light": "d18158",
    "magma": "9d362f", "magma-low": "5b242b", "magma-high": "c5603b",
    "magma-edge": "d77042", "magma-bright": "ff9d50", "ember-seam": "eb7442",
    "dry-grass": "967267", "smoke-ridge": "5f4654", "smoke-ridge-light": "886170",
}


def register_palette():
    PALETTE.update(FORGE)
    for key,strength in (("magma-edge",.21),("magma-bright",.76),("ember-seam",.41)):
        mat=material(key)
        shader=mat.node_tree.nodes["Principled BSDF"]
        shader.inputs["Emission Color"].default_value=mat.diffuse_color
        shader.inputs["Emission Strength"].default_value=strength


def ash_height(y,i):
    return -.074+max(0,abs(y)-2.65)*(.036+.019*sin(i*.32))


def plateau(group):
    rings=[[(CENTER[0]+(x-CENTER[0])*scale,y*scale,ash_height(y*scale,i))
            for i,(x,y) in enumerate(rim())] for scale in (1,.83,.60,.34,.062)]
    tones=("ash","ash-light","ash","ash-dark","ash","ash")
    rows(group,"Packed ash on the basalt island",rings,
         lambda band,i: tones[(i//4+2*band)%len(tones)],floor=True)


def cliff(group):
    edge=rim()
    profile=((1,0),(1.011,-.23),(.985,-.52),(1.026,-.79),(1.033,-1.05),(1.019,-1.59))
    rings=[[(CENTER[0]+(x-CENTER[0])*scale,y*scale,z+.052*sin(i*.66+band))
            for i,(x,y) in enumerate(edge)] for band,(scale,z) in enumerate(profile)]
    rings[0]=[(x,y,ash_height(y,i)) for i,(x,y) in enumerate(edge)]
    keys=("basalt-cut","basalt","sintered-crust","basalt-deep","cooled-slag")
    rows(group,"Exposed compact basalt strata",rings,
         lambda band,i: "volcanic-ore" if band==2 and i%9==0 else keys[band],reverse=True)
    for i in (3,9,14,25,33,40,46,54,59):
        x,y=edge[i]
        pebble(group,"Cooling buttress at the rock face",(x,y,-.52),(.37,.34,.90),
               "basalt-deep",accent="basalt-cut",phase=i*.7)


def sea(group):
    n=25
    vertices=[(8+(x-12)*4.2,(y-10)*4.2,LAVA_LEVEL) for y in range(n) for x in range(n)]
    quads=[(i,i+1,i+n+1,i+n) for y in range(n-1) for x in range(n-1) for i in (y*n+x,)]
    melt=surface(group,"Lava basin with flowing value bands",vertices,quads,["magma"]*len(quads))
    attribute=melt.data.color_attributes.new(name="Lava flow values",type="FLOAT_COLOR",domain="POINT")
    nodes=melt.data.materials[0].node_tree
    ramp=nodes.nodes.new("ShaderNodeVertexColor")
    ramp.layer_name=attribute.name
    shader=nodes.nodes["Principled BSDF"]
    shader.inputs["Roughness"].default_value=.57
    shader.inputs["Emission Strength"].default_value=.23
    for input_name in ("Base Color","Emission Color"):
        nodes.links.new(ramp.outputs["Color"],shader.inputs[input_name])
    low,high=(color(PALETTE[k]) for k in ("magma-low","magma-high"))
    for v,rgba in zip(melt.data.vertices,attribute.data,strict=True):
        x,y,_=v.co
        mix=.48+.18*sin(x*.33+y*.41)+.16*cos(.44*x-.37*y)+.07*sin(.84*y)
        rgba.color=tuple(a+(b-a)*mix for a,b in zip(low,high,strict=True))
    for x,y,length in ((-5.82,-5.82,1.77),(3.55,-9.15,1.62),(8.1,-10.15,2.25),
                       (11.78,-9.25,1.68),(20.39,-5.52,1.91),(-3.64,6.91,2.10),
                       (18.44,7.89,2.19)):
        current(group,x,y,length)


def current(group,x,y,length):
    points=[]
    for i in range(10):
        u=i/9
        y0=y+.20*sin(u*4.2)+.12*sin(u*7+x)
        width=.058*sin(u*3.14159)+.010
        for side in (-1,1):
            points.append((x+length*u,y0+side*width,LAVA_LEVEL+.010))
    faces=[(i,i+2,i+3,i+1) for i in range(0,18,2)]
    surface(group,"Elongated incandescent current on the lava basin",points,faces,
            ["magma-edge" if i%3 else "ember-seam" for i in range(len(faces))])


def coast(group):
    edge=rim()
    rings=[[(CENTER[0]+(x-CENTER[0])*(scale+.012*sin(i*1.1+j)),
             y*(scale+.010*cos(i*.73+j)),LAVA_LEVEL+.017+j*.012)
            for i,(x,y) in enumerate(edge)] for j,scale in enumerate((1.153,1.110,1.072,1.019))]
    keys=("magma-edge","cooled-slag","sintered-crust")
    rows(group,"Cracked cooling crust along the shoreline",rings,
         lambda band,i: "magma-edge" if (i+band*2)%13==0 else keys[band])
    for i in range(1,len(edge),4):
        a,b,c=(rings[j][i] for j in (0,1,3))
        a,b,c=((x,y,z+.006) for x,y,z in (a,b,c))
        surface(group,"Fine shoreline seam",(a,(b[0]+.054,b[1]-.028,b[2]),c,b),[(0,1,2,3)],
                ["ember-seam"])


def lane_bed(group):
    box(group,"Compacted BattleLine bed",(0,0,-.066),(LANE_LENGTH+2.16,LANE_WIDTH+.23,.105),
        "forge-bed",.042)


def flagstone(group,variant):
    rng=Random(481+variant)
    x,y=.485,.606
    outline=[(-x+.078,-y),(x-.088,-y),(x,-y+.090),(x,y-.066),
             (x-.071,y),(-x+.093,y),(-x,y-.081),(-x,-y+.075)]
    offsets=[(rng.uniform(-.013,.013),rng.uniform(-.012,.012)) for _ in outline]
    vertices=[(u*scale+du,v*scale+dv,z) for z,scale in ((-.056,1),(-.023,1),(.004,.968))
              for (u,v),(du,dv) in zip(outline,offsets,strict=True)]
    faces=[(j*8+i,j*8+(i+1)%8,(j+1)*8+(i+1)%8,(j+1)*8+i) for j in (0,1) for i in range(8)]
    faces.append(tuple(range(16,24)))
    surface(group,"Hammered charcoal road slab",vertices,faces,
            ["forge-bed"]*8+["forge-slate-dark"]*8+
            [("forge-slate","forge-slate-light","forge-slate","forge-slate-dark")[variant]])


def shoulder(group):
    box(group,"Low dark basalt curb",(0,0,-.023),(.92,.265,.106),"basalt-deep",.025)
    box(group,"Chamfered cooled curb lip",(0,0,.030),(.889,.207,.042),"forge-curb",.016)
