"""Shape the Ancient Ruins plateau, limestone BattleLine and turquoise lagoon."""

from math import cos, sin
from random import Random

from mesh import PALETTE, box, color, material, pebble, surface
from terrain import CENTER, LANE_LENGTH, LANE_WIDTH, rim, rows

WATERLINE=-1.28
RUINS={
    "ruin-earth":"77745b", "ruin-earth-light":"8b8268", "ruin-earth-dark":"666852",
    "limestone":"bdb092", "limestone-light":"d4c7a6", "limestone-dark":"93866d",
    "limestone-seam":"aa9a7d", "ruin-sand":"9d8a6b", "ruin-shadow":"555849",
    "ruin-paver":"a39883", "ruin-paver-light":"b0a28b", "ruin-paver-dark":"8c8375",
    "ruin-road-bed":"726858", "ruin-curb":"a99c7e", "ruin-jade":"659c91",
    "ruin-jade-dark":"3d6d65", "ruin-jade-light":"9acbb2",
    "ruin-moss":"718052", "ruin-fern":"697951", "ruin-fir-dark":"3d5952",
    "ruin-fir":"52736a", "ruin-leaf":"879157", "ruin-leaf-dark":"62764d",
    "ruin-reed":"938350", "ruin-flower":"d8b47b", "ruin-petal":"c09087",
    "ruin-oak":"6d5848", "ruin-wood":"8f7459", "ruin-grain":"a78b69",
    "ruin-bronze":"9d7950", "ruin-copper":"577d70", "ruin-linen":"dbca9f",
    "ruin-market":"657a6b", "ruin-ore":"599287", "ruin-ore-light":"97bd9e",
    "water-ruin":"306e71", "lagoon-deep":"225767", "lagoon-light":"51948f",
    "lagoon-shelf":"609b92", "lagoon-sand":"82a69a", "lagoon-ripple":"9db6a5",
    "ritual-glow":"9ce8c4", "ritual-ember":"518c82",
}


def register_palette():
    PALETTE.update(RUINS)
    for key,metallic,roughness in (("ruin-copper",.42,.52),("ruin-bronze",.59,.43),("ruin-jade",.06,.43)):
        shader=material(key).node_tree.nodes["Principled BSDF"]
        shader.inputs["Metallic"].default_value=metallic
        shader.inputs["Roughness"].default_value=roughness
    mat=material("ritual-glow")
    shader=mat.node_tree.nodes["Principled BSDF"]
    shader.inputs["Emission Color"].default_value=mat.diffuse_color
    shader.inputs["Emission Strength"].default_value=.68


def earth_height(y,i):
    return -.074+max(0,abs(y)-2.62)*(.029+.024*sin(i*.39))


def plateau(group):
    bands=[[(CENTER[0]+(x-CENTER[0])*scale,y*scale,earth_height(y*scale,i))
            for i,(x,y) in enumerate(rim())] for scale in (1,.82,.58,.33,.061)]
    earth=("ruin-earth","ruin-earth-light","ruin-earth","ruin-earth-dark","ruin-earth","ruin-earth")
    rows(group,"Dry moss and packed earth",bands,
         lambda band,i:earth[(i//3+2*band)%len(earth)],floor=True)


def cliffs(group):
    edge=rim()
    profile=((1,0),(1.008,-.19),(.988,-.44),(1.031,-.74),(1.040,-1.05),(1.015,-1.60))
    bands=[[(CENTER[0]+(x-CENTER[0])*scale,y*scale,z+.055*sin(i*.61+band))
            for i,(x,y) in enumerate(edge)] for band,(scale,z) in enumerate(profile)]
    bands[0]=[(x,y,earth_height(y,i)) for i,(x,y) in enumerate(edge)]
    keys=("limestone-light","limestone-seam","limestone-dark","ruin-sand","ruin-shadow")
    rows(group,"Exposed pale sedimentary cliff",bands,
         lambda band,i:"limestone" if band<4 and (i+band)%9==0 else keys[band],reverse=True)
    for i in (5,10,18,25,32,41,46,54,61):
        x,y=edge[i]
        pebble(group,"Partly eroded limestone outcrop",(x,y,-.51),(.40,.33,.89),
               "limestone-dark",sides=7,accent="limestone",phase=i*.71)


def lagoon(group):
    n=24
    vertices=[(8+(x-12)*4.3+.14*sin(x+y),(y-10)*4.3+.19*cos(x-y),WATERLINE)
              for y in range(n) for x in range(n)]
    faces=[(i,i+1,i+n+1,i+n) for y in range(n-1) for x in range(n-1) for i in (y*n+x,)]
    water=surface(group,"Open turquoise lagoon",vertices,faces,["water-ruin"]*len(faces))
    tones=water.data.color_attributes.new(name="Lagoon depth tones",type="FLOAT_COLOR",domain="POINT")
    nodes=water.data.materials[0].node_tree
    attribute=nodes.nodes.new("ShaderNodeVertexColor")
    attribute.layer_name=tones.name
    nodes.links.new(attribute.outputs["Color"],nodes.nodes["Principled BSDF"].inputs["Base Color"])
    low,high=(color(PALETTE[k]) for k in ("lagoon-deep","lagoon-light"))
    for vertex,rgba in zip(water.data.vertices,tones.data,strict=True):
        x,y,_=vertex.co
        t=.48+.21*sin(x*.22+y*.41)+.13*cos(x*.49-y*.46)
        rgba.color=tuple(a+(b-a)*t for a,b in zip(low,high,strict=True))


def shelf(group):
    edge=rim()
    rings=[[(CENTER[0]+(x-CENTER[0])*(scale+.007*cos(i*1.31+band)),
             y*(scale+.008*sin(i*.79+band)),WATERLINE+.018+.009*band)
            for i,(x,y) in enumerate(edge)] for band,scale in enumerate((1.157,1.105,1.060,1.018))]
    keys=("lagoon-shelf","lagoon-sand","limestone-seam")
    rows(group,"Silt and limestone beneath the shallow shoreline",rings,
         lambda band,i:"lagoon-shelf" if band==2 and i%9==0 else keys[band])
    for i in range(1,len(edge),3):
        if i%7 in (0,1):
            continue
        a,b=rings[1][i],rings[1][(i+1)%len(edge)]
        ridge=[(a[0],a[1],a[2]+.008),(b[0],b[1],b[2]+.008),
               (b[0]+.022,b[1]+.044,b[2]+.008),(a[0]+.018,a[1]+.031,a[2]+.008)]
        surface(group,"Small broken lagoon ripple",ridge,[(0,1,2,3)],["lagoon-ripple"])


def lane_bed(group):
    box(group,"Packed foundation under limestone paving",(0,0,-.066),
        (LANE_LENGTH+2.16,LANE_WIDTH+.23,.104),"ruin-road-bed",.04)


def flagstone(group,variant):
    rng=Random(248+variant)
    x,y=.485,.606
    outline=[(-x+.072,-y),(x-.076,-y),(x,-y+.062),(x,y-.080),
             (x-.072,y),(-x+.082,y),(-x,y-.079),(-x,-y+.067)]
    offsets=[(rng.uniform(-.013,.013),rng.uniform(-.012,.012)) for _ in outline]
    vertices=[(u*scale+du,v*scale+dv,z) for z,scale in ((-.055,1),(-.019,1),(.004,.971))
              for (u,v),(du,dv) in zip(outline,offsets,strict=True)]
    faces=[(j*8+i,j*8+(i+1)%8,(j+1)*8+(i+1)%8,(j+1)*8+i) for j in (0,1) for i in range(8)]
    faces.append(tuple(range(16,24)))
    surface(group,"Chipped chamfered limestone flagstone",vertices,faces,
            ["ruin-road-bed"]*8+["limestone-dark"]*8+
            [("ruin-paver","ruin-paver-light","ruin-paver","ruin-paver-dark")[variant]])
    if variant in (1,2):
        slab_weathering(group,variant)


def slab_weathering(group,variant):
    sign=1 if variant==1 else -1
    cracks=(((-.405,.199),(-.316,.153),(-.273,.055),(-.156,.014)),
            ((.426,-.277),(.361,-.249),(.329,-.161),(.228,-.114)))
    for i,points in enumerate(cracks):
        vertices=[(x,sign*(y+dy),.0043) for x,y in points for dy in (0,.005 if i else .007)]
        surface(group,"Hairline wear traced on limestone",vertices,[(0,1,3,2),(2,3,5,4),(4,5,7,6)],
                ["ruin-paver-dark"]*3)


def curb(group):
    box(group,"Seated limestone curb bed",(0,0,-.020),(.92,.259,.102),"limestone-dark",.020)
    box(group,"Individually chamfered curb cap",(0,0,.029),(.886,.209,.044),"ruin-curb",.014)
