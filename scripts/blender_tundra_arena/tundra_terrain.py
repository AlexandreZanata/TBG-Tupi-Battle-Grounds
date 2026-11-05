"""Shape the snowbound plateau, compact stone BattleLine and broken ice coast."""

from math import cos, sin, tau
from random import Random

from mesh import PALETTE, box, color, pebble, surface
from terrain import CENTER, LANE_LENGTH, LANE_WIDTH, SEGMENTS, rim, rows

WATERLINE = -1.28
WINTER = {
    "snow": "dbe5e5", "snow-light": "f0f2ec", "snow-shadow": "b9cdd6",
    "snow-shelf": "aec9d1", "frost-bank": "b9d5dd",
    "road-slate": "6e818d", "road-light": "82949d", "road-dark": "596f7e",
    "road-bed": "4b606e", "road-edge": "a1bac3",
    "glacier": "70a8c1", "glacier-dark": "49768d", "glacier-light": "a3d5e1",
    "ice": "87bdcd", "ice-deep": "4b91a6", "ice-white": "bedce2",
    "fracture": "dbf3ec", "ice-rim": "568a9d", "ice-vein": "3d8298",
    "frost-granite": "5d707d", "frost-basalt": "3e5566", "frost-ashlar": "879ba4",
    "frost-fir": "31595c", "frost-fir-light": "52777a", "frost-fir-dark": "244851",
    "winter-willow": "746454", "winter-reed": "ac8053", "winter-moss": "7f8170",
    "water-tundra": "214355", "tundra-water-low": "19384c", "tundra-water-high": "2e6675",
    "winter-wood": "735b49", "winter-timber": "4c4941", "winter-grain": "a78865",
    "winter-cloth": "456677", "winter-linen": "e0d9bd",
    "winter-copper": "b36a47", "winter-crystal": "8cdbdc", "winter-crystal-dark": "427e9c",
    "ridge-dark": "5e7d96", "ridge-light": "9ab0bf", "ridge-snow": "d0dbe0",
}


def register_palette():
    PALETTE.update(WINTER)


def snow_height(y,i):
    return -.074 + max(0,abs(y)-2.55)*(.031+.021*sin(i*.39))


def plateau(group):
    edge=rim()
    rings=[[(CENTER[0]+(x-CENTER[0])*scale,y*scale,snow_height(y*scale,i))
            for i,(x,y) in enumerate(edge)] for scale in (1,.82,.57,.32,.065)]
    tones=("snow","snow","snow-light","snow","snow-shadow","snow")
    rows(group,"Packed plateau snow with shaded banks",rings,
         lambda band,i: tones[(i//3+2*band)%len(tones)],floor=True)


def escarpment(group):
    edge=rim()
    profile=((1,0),(1.008,-.23),(.981,-.48),(1.028,-.75),(1.043,-1.09),(1.012,-1.63))
    bands=[[(CENTER[0]+(x-CENTER[0])*scale,y*scale,z+.048*sin(i*.69+band))
            for i,(x,y) in enumerate(edge)] for band,(scale,z) in enumerate(profile)]
    bands[0]=[(x,y,snow_height(y,i)) for i,(x,y) in enumerate(edge)]
    keys=("snow-shelf","frost-ashlar","frost-granite","glacier-dark","frost-basalt")
    rows(group,"Stratified frost-cut rock",bands,
         lambda band,i: "glacier" if band>1 and (i+band)%9==0 else keys[band],reverse=True)
    for i in (5,10,17,25,34,39,45,50,56,61):
        x,y=edge[i]
        pebble(group,"Ice-sealed projecting rock",(x,y,-.49),(.49,.37,.81),
               "frost-granite",accent="glacier",phase=i)


def sea(group):
    n=23
    vertices=[(8+(x-11)*4.3,(y-9)*4.3,WATERLINE) for y in range(n) for x in range(n)]
    faces=[(i,i+1,i+n+1,i+n) for y in range(n-1) for x in range(n-1) for i in (y*n+x,)]
    water=surface(group,"Cold open channel",vertices,faces,["water-tundra"]*len(faces))
    colors=water.data.color_attributes.new(name="Deep water bands",type="FLOAT_COLOR",domain="POINT")
    shader=water.data.materials[0].node_tree
    attr=shader.nodes.new("ShaderNodeVertexColor")
    attr.layer_name=colors.name
    bsdf=shader.nodes["Principled BSDF"]
    bsdf.inputs["Roughness"].default_value=.36
    shader.links.new(attr.outputs["Color"],bsdf.inputs["Base Color"])
    dark,light=(color(PALETTE[key]) for key in ("tundra-water-low","tundra-water-high"))
    for v,rgba in zip(water.data.vertices,colors.data,strict=True):
        x,y,_=v.co
        blend=.46+.18*sin(.28*x+.48*y)+.11*cos(.89*x-.42*y)
        rgba.color=tuple(lo+(hi-lo)*blend for lo,hi in zip(dark,light,strict=True))


def shelf(group):
    rings=[]
    for band,scale in enumerate((1.16,1.105,1.061,1.021)):
        rings.append([(
            CENTER[0]+(x-CENTER[0])*(scale+.010*sin(i*1.49+band)),
            y*(scale+.010*cos(i*1.25+band)),WATERLINE+.026+.010*band,
        ) for i,(x,y) in enumerate(rim())])
    keys=("ice-deep","ice","ice-white")
    rows(group,"Jagged glazed ice shelf",rings,
         lambda band,i: "ice-rim" if (i+3*band)%11==0 else keys[band])
    for i in range(2,SEGMENTS,3):
        a,b,c=(rings[band][i] for band in (0,1,3))
        a,b,c=((x,y,z+.007) for x,y,z in (a,b,c))
        points=[a,(b[0]+.071,b[1]-.052,b[2]),c,(b[0]+.018,b[1]-.026,b[2])]
        surface(group,"Hairline shelf fracture",points,[(0,1,2,3)],["fracture"])


def lane_bed(group):
    box(group,"Compacted slate road foundation",(0,0,-.068),(LANE_LENGTH+2.16,LANE_WIDTH+.23,.104),"road-bed",.041)


def flagstone(group,variant):
    rng=Random(521+variant)
    half_x,half_y=.486,.607
    x,y=half_x,half_y
    corners=[(-x+.069,-y),(x-.091,-y),(x,-y+.072),(x,y-.076),
             (x-.058,y),(-x+.071,y),(-x,y-.075),(-x,-y+.059)]
    offsets=[(rng.uniform(-.016,.016),rng.uniform(-.012,.012)) for _ in corners]
    vertices=[(u*(.968 if i==2 else 1)+dx,v*(.981 if i==2 else 1)+dy,z)
              for i,z in enumerate((-.055,-.020,.004))
              for (u,v),(dx,dy) in zip(corners,offsets,strict=True)]
    faces=[(j*8+i,j*8+(i+1)%8,(j+1)*8+(i+1)%8,(j+1)*8+i)
           for j in (0,1) for i in range(8)]
    faces.append(tuple(range(16,24)))
    key=("road-slate","road-light","road-slate","road-dark")[variant]
    surface(group,"Chamfered blue-gray paving slab",vertices,faces,
            ["road-bed"]*8+["road-dark"]*8+[key])


def shoulder(group):
    box(group,"Lane shoulder recessed in snow",(0,0,-.022),(.92,.267,.103),"road-dark",.025)
    box(group,"Honed frost curb",(0,0,.028),(.89,.197,.043),"road-edge",.018)
    for x in (-.30,.31):
        pebble(group,"Small cap of drifted frost",(x,.086,.044),(.121,.081,.029),
               "snow",sides=6,accent="snow-light",phase=x)
