"""Construct the meadow, playable lane and layered coast."""

from math import cos, sin, tau
from random import Random

from mesh import PALETTE, box, color, pebble, surface

SEGMENTS = 64
CENTER = (8,0)
LANE_WIDTH = 5.0
LANE_LENGTH = 16.0
HOLD_CENTERS = (-.9,16.9)


def rim():
    return [(8+12.15*r*cos(a),6.22*r*sin(a)) for i in range(SEGMENTS)
            for a in (tau*i/SEGMENTS,) for r in (1+.028*sin(5*a)+.019*cos(11*a),)]


def rows(group, name, rings, paint, *, floor=False, reverse=False):
    n = len(rings[0])
    vertices = [v for ring in rings for v in ring]
    quads = [(j*n+i,j*n+(i+1)%n,(j+1)*n+(i+1)%n,(j+1)*n+i)
             for j in range(len(rings)-1) for i in range(n)]
    colors = [paint(j,i) for j in range(len(rings)-1) for i in range(n)]
    if floor:
        quads.append(tuple(range((len(rings)-1)*n,len(rings)*n)))
        colors.append(paint(0,0))
    if reverse:
        quads = [face[::-1] for face in quads]
    return surface(group,name,vertices,quads,colors)


def meadow(group):
    edge = rim()
    rings = []
    for factor in (1,.81,.59,.34,.07):
        ring = []
        for i,(x,y) in enumerate(edge):
            py = y*factor
            ring.append((8+(x-8)*factor,py,grass_height(py,i)))
        rings.append(ring)
    shades = ("turf","turf","turf-light","turf","turf-shade","turf")
    rows(group,"Sculpted meadow surface",rings,lambda band,i: shades[(i//3+band*2)%len(shades)],floor=True)


def coast(group):
    edge = rim()
    rings = []
    for row,(scale,z) in enumerate(((1,-.01),(1.008,-.18),(.987,-.52),(1.019,-.79),(1.041,-1.11),(1.014,-1.69))):
        rings.append([(8+(x-8)*scale,y*scale,z+(.04 if row<2 else .14)*sin(i*.66+row))
                      for i,(x,y) in enumerate(edge)])
    rings[0] = [(x,y,grass_height(y,i)) for i,(x,y) in enumerate(edge)]
    shades = ("cliff-light","stratum","cliff","stratum","cliff-shadow")
    rows(group,"Coastal escarpment with exposed rock bands",rings,
         lambda band,i: shades[band] if (i+2*band)%7 else "cliff",reverse=True)
    for i in (4,9,14,23,32,39,45,49,51,57,62):
        x,y = edge[i]
        pebble(group,"Weathered cliff shoulder",(8+(x-8)*1.017,y*1.014,-.38),
               (.51,.47,.95),"cliff",accent="stratum",phase=i)


def grass_height(y,i):
    return -.066 + max(0,abs(y)-2.6)*(.045 + .031*sin(i*.35))


def lane_bed(group):
    box(group,"Compact earth foundation",(0,0,-.061),(18.16,5.23,.104),"path-bed",.041)


def paver(group,variant):
    key = ("sand","sand-light","sand","sand-dark")[variant]
    x,y = .491,.603
    corners = [(-x+.052,-y),(x-.055,-y),(x,-y+.065),(x,y-.06),
               (x-.045,y),(-x+.059,y),(-x,y-.068),(-x,-y+.048)]
    vertices = [(u*(.96 if row==2 else 1),v*(.978 if row==2 else 1),z+.003*sin(i*2+variant))
                for row,z in enumerate((-.053,-.016,.006)) for i,(u,v) in enumerate(corners)]
    faces = [(j*8+i,j*8+(i+1)%8,(j+1)*8+(i+1)%8,(j+1)*8+i)
             for j in range(2) for i in range(8)]
    faces.append(tuple(range(16,24)))
    surface(group,"Hand-dressed sandstone flag",vertices,faces,["path-bed"]*8+["sand-dark"]*8+[key])


def shoulder(group):
    box(group,"Stone road shoulder",(0,0,-.019),(.92,.289,.103),"curb",.025)
    box(group,"Narrow worn lip",(0,0,.023),(.87,.218,.029),"curb-light",.012)


def ocean(group):
    rng = Random(13)
    n = 33
    vertices = [(8+(x-16)*2.47+.16*sin(y+x), (y-12)*2.47+.19*cos(x+y), -1.28)
                for y in range(n) for x in range(n)]
    faces = []
    for y in range(n-1):
        for x in range(n-1):
            i = y*n+x
            faces.extend(((i,i+1,i+n+1),(i,i+n+1,i+n)))
    water = surface(group,"Open sea with broad painted values",vertices,faces,["water"]*len(faces))
    paint_open_sea(water)
    edge = rim()
    strips = [[(8+(x-8)*scale,y*scale,-1.245 + i*.005) for x,y in edge]
              for i,scale in enumerate((1.22,1.13,1.067,1.019))]
    rows(group,"Shallow shelf around the main island",strips,
         lambda band,i: ("water-light","shelf-deep","shelf")[band])
    for j in range(2):
        for i in range(SEGMENTS):
            if (i+3*j)%8 > (5 if j==0 else 2):
                continue
            a,b = edge[i],edge[(i+1)%SEGMENTS]
            at = lambda p,s: (8+(p[0]-8)*s,p[1]*s,-1.220+j*.003)
            scale = 1.05+j*.045 + .006*sin(i*.8)
            points = [at(a,scale+.007),at(b,scale+.007),at(b,scale),at(a,scale)]
            surface(group,"Broken tidal crest",points,[(0,1,2,3)],["foam" if j==0 else "ripple"])
    for _ in range(105):
        x,y = rng.uniform(-9,25),rng.uniform(-13,19)
        if ((x-8)/12.8)**2+(y/6.95)**2 < 1.02:
            continue
        length = rng.uniform(.08,.69)
        points = [(x-length,y,-1.215),(x+length*.76,y-.033,-1.215),
                  (x+length,y+.029,-1.215),(x-length*.43,y+.040,-1.215)]
        surface(group,"Sea ripple stroke",points,[(0,1,2,3)],["ripple"])


def paint_open_sea(water):
    layer = water.data.color_attributes.new(name="Water tones",type="FLOAT_COLOR",domain="POINT")
    shader = water.data.materials[0].node_tree
    attr = shader.nodes.new("ShaderNodeVertexColor")
    attr.layer_name = layer.name
    shader.links.new(attr.outputs["Color"],shader.nodes.get("Principled BSDF").inputs["Base Color"])
    dark,light = (color(PALETTE[key]) for key in ("water-dark","water-light"))
    for v,rgba in zip(water.data.vertices,layer.data,strict=True):
        x,y,_ = v.co
        blend = .42+.21*sin(x*.23+y*.32)+.14*cos(x*.56-y*.45)+.065*sin(y*.81)
        rgba.color = tuple(lo+(hi-lo)*blend for lo,hi in zip(dark,light,strict=True))
