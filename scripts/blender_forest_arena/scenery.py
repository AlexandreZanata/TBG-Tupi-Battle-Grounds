"""Build the coastal flora, rocks, waterfalls and offshore islands."""

from math import cos, sin, tau
from random import Random

from mesh import pebble, surface, tube


def fir(group, dense):
    tube(group,"Tapered pine trunk",[(0,0,0),(.029,.01,.47),(-.037,.035,1.72),(.022,.021,2.72)],
         [.135,.103,.072,.017],"oak",sides=8,accent="wood")
    for row,(z,r,h) in enumerate(((.43,.70,.96),(.89,.63,.93),(1.35,.52,.91),(1.80,.36,.79),(2.19,.24,.70))):
        if not dense and row==1:
            continue
        x = .038*sin(row*3)
        tube(group,"Layered needle bough",[(x,0,z),(x,0,z+.14),(x*.6,.029,z+h)],
             [r*.72,r,.011],"fir-shadow" if row<2 else "fir",sides=9+row%2,accent="fir-light")


def oak(group):
    tube(group,"Forked oak trunk",[(0,0,0),(.052,.01,.56),(-.025,.009,1.18),(.025,.07,1.75)],
         [.172,.135,.095,.036],"oak",sides=8,accent="wood")
    for i,(x,y,z,r) in enumerate(((-.49,.02,1.41,.54),(.43,-.10,1.64,.56),
                                  (0,-.36,1.83,.53),(.26,.35,1.99,.58),(-.32,.22,2.15,.54),
                                  (.02,.03,2.47,.48))):
        tube(group,"Oak branch",[(0,.03,z*.49),(x*.72,y*.82,z*.87),(x,y,z)],
             [.079,.061,.023],"oak",sides=7)
        pebble(group,"Cloud of oak leaves",(x,y,z),(r,r*.87,r*.76),
               "leaf" if i%2 else "leaf-deep",sides=9,accent="leaf-light",phase=i*.7)


def rock(group):
    for i,(x,y,z,rx,ry,rz) in enumerate(((0,0,.32,.51,.46,.56),(-.42,-.11,.12,.25,.22,.22),
                                         (.35,.26,.11,.20,.23,.19),(.29,-.32,.09,.15,.18,.15))):
        pebble(group,"Weathered shore rock",(x,y,z),(rx,ry,rz),"stone-cool",sides=7,
               accent="stone-light",phase=i*.71)
    pebble(group,"Attached moss on upper shoulder",(-.05,.07,.58),(.32,.27,.15),
           "leaf-deep",sides=8,accent="moss",phase=1.25)


def shrub(group):
    for i,(x,y,z,r) in enumerate(((-.23,0,.17,.32),(.21,.10,.18,.29),(0,-.11,.29,.32),
                                 (.04,.25,.30,.27),(-.29,.23,.12,.22))):
        pebble(group,"Clustered shrub leaves",(x,y,z),(r,r*.86,r*.78),"leaf" if i%2 else "leaf-deep",
               sides=7,accent="leaf-light",phase=i)


def grass(group):
    rng = Random(229)
    verts, faces, colors = [],[],[]
    for i in range(21):
        a = rng.uniform(0,tau)
        x,y = rng.uniform(-.24,.24),rng.uniform(-.19,.19)
        width, height = rng.uniform(.012,.026),rng.uniform(.10,.30)
        ux,uy = cos(a),sin(a)
        offset = len(verts)
        verts.extend(((x-width*uy,y+width*ux,0),(x+width*uy,y-width*ux,0),
                      (x+.075*ux,y+.075*uy,height*.65),(x+.14*ux,y+.14*uy,height)))
        faces.extend(((offset,offset+1,offset+2),(offset,offset+2,offset+3)))
        colors.extend(("grass","grass-light" if i%3 else "grass"))
    surface(group,"Bent meadow grass blades",verts,faces,colors)


def fern(group):
    for i in range(7):
        angle = i*tau/7
        ux,uy = cos(angle),sin(angle)
        tube(group,"Arching fern rachis",[(0,0,0),(.22*ux,.22*uy,.24),(.51*ux,.51*uy,.25)],
             [.019,.015,.006],"grass-light",sides=5)
        vertices,faces,colors = [],[],[]
        for j in range(5):
            r = .14+j*.069
            z = .16+.11*sin(j*.48)
            x,y = r*ux,r*uy
            width = .109-j*.015
            for side in (-1,1):
                n = len(vertices)
                vertices.extend(((x,y,z),(x+ux*.074-side*uy*width,y+uy*.074+side*ux*width,z-.016),
                                 (x+ux*.12,y+uy*.12,z+.017)))
                faces.append((n,n+1,n+2) if side==1 else (n,n+2,n+1))
                colors.append("leaf-light" if j%2==0 else "leaf")
        surface(group,"Paired fern leaflets",vertices,faces,colors)


def flowers(group):
    vertices,faces,colors = [],[],[]
    for i,(x,y,z) in enumerate(((-.16,.08,.18),(-.045,-.11,.15),(.10,-.07,.23),
                                 (.20,.055,.17),(.04,.16,.20))):
        tube(group,"Wildflower stem",[(x*.81,y*.79,0),(x,y,z)],[.012,.007],"grass",sides=5)
        for j in range(5):
            a = j*tau/5+i
            n = len(vertices)
            vertices.extend((x+r*cos(a+turn),y+r*sin(a+turn),z+height)
                            for r,turn,height in ((.012,0,0),(.052,-.51,.002),(.084,0,-.012),
                                                  (.052,.51,.004),(.045,0,.011)))
            faces.extend((n+k,n+(k+1)%4,n+4) for k in range(4))
            colors.extend(["petal" if i!=2 else "poppy"]*4)
        n = len(vertices)
        vertices.extend((x+.024*cos(j*tau/6),y+.024*sin(j*tau/6),z+.013) for j in range(6))
        faces.append(tuple(range(n,n+6)))
        colors.append("trim")
    surface(group,"Folded petals and flower centers",vertices,faces,colors)


def islet(group, tall):
    n = 15
    height = 2.33 if tall else 1.41
    radii = (2.43,1.76) if tall else (1.78,1.43)
    rings = ((1.05,-1.60), (1,-1.18),(.92,-.65),(.86,height*.50-1.23),(.79,height-1.25),(.67,height-.99))
    vertices = [(radii[0]*f*(1+.13*sin(a*5))*cos(a),radii[1]*f*sin(a),
                 z+.12*cos(a*3)) for f,z in rings for i in range(n) for a in (i*tau/n,)]
    faces = [(r*n+i,r*n+(i+1)%n,(r+1)*n+(i+1)%n,(r+1)*n+i)
             for r in range(len(rings)-1) for i in range(n)]
    colors = [("cliff","stratum","cliff-shadow","cliff","turf")[r] if i%4 else "cliff-light"
              for r in range(len(rings)-1) for i in range(n)]
    faces.append(tuple(range((len(rings)-1)*n,len(rings)*n)))
    surface(group,"Layered offshore headland",vertices,faces,colors+["turf"])


def cascade(group):
    for i in range(6):
        x = (i-2.5)*.086
        vertices = [(x-.044,.47,1.22),(x+.044,.47,1.22),
                    (x-.044,.03,1.16),(x+.044,.03,1.16),
                    (x-.050,-.060,.95),(x+.050,-.060,.95),
                    (x-.073,-.39,-1.19),(x+.073,-.39,-1.19)]
        surface(group,"Vertical water ribbon",vertices,[(0,1,3,2),(2,3,5,4),(4,5,7,6)],
                ["ripple","foam","shelf" if i%2 else "ripple"])
    for i in range(14):
        a = i*tau/14
        x,y = .68*cos(a),-.48+.32*sin(a)
        points = [(x,y,-1.214),(x+.19*sin(a+.2),y-.11*cos(a),-1.214),
                  (x+.10*sin(a),y-.05*cos(a),-1.214)]
        surface(group,"Waterfall foam fan",points,[(0,1,2)],["foam"])


def reef(group):
    for i,(x,y,z,r) in enumerate(((0,0,-1.10,.43),(.25,.19,-1.08,.39),(-.39,.02,-1.19,.20))):
        pebble(group,"Tide-worn boulder",(x,y,z),(r,r*.92,r*.78),"cliff",accent="stratum",phase=i)
    for i in range(11):
        a,b = i*tau/11,(i+.72)*tau/11
        points = [(r*cos(t),.82*r*sin(t),-1.217) for r,t in ((.73,a),(.73,b),(.67,b),(.67,a))]
        surface(group,"Tidal foam around rock",points,[(0,1,2,3)],["foam"])
