"""Shared architectural and moving components for the Stronghold study."""

from math import cos, sin, tau

import bpy

from mesh import box, surface, tube

FORMS = ("Base", "A1", "A2", "B1", "B2")
STONE = ("stone", "stone-light", "stone-cool")


def layer(parent,name):
    group = bpy.data.collections.new(name)
    parent.children.link(group)
    return group


def empty(group,name,at,parent=None):
    obj = bpy.data.objects.new(name,None)
    obj.empty_display_size = .12
    group.objects.link(obj)
    obj.parent = parent
    obj.location = at
    return obj


def mechanism(group,name,at,build,*,pitch=0,muzzle=None,motion):
    parts = layer(group,name)
    pivot = empty(parts,name+" pivot",at)
    pivot["Motion"] = motion
    build(parts)
    for obj in tuple(parts.objects):
        if obj != pivot:
            obj.parent = pivot
    pivot.rotation_euler.y = -pitch
    if muzzle is not None:
        marker = empty(parts,"Muzzle",muzzle,pivot)
        marker.empty_display_type = "CIRCLE"
    return pivot


def profile_x(group,name,rings,colors,*,sides=14,cap=False):
    """Revolve the side profile along +X with a rim and open chamber when needed."""
    pts = [(x,r*cos(tau*i/sides),r*sin(tau*i/sides)) for x,r in rings for i in range(sides)]
    faces = [(row*sides+i,row*sides+(i+1)%sides,(row+1)*sides+(i+1)%sides,(row+1)*sides+i)
             for row in range(len(rings)-1) for i in range(sides)]
    materials = [key for key in colors for _ in range(sides)]
    if cap:
        faces += [tuple(reversed(range(sides))),tuple(range(len(pts)-sides,len(pts)))]
        materials += [colors[0],colors[-1]]
    return surface(group,name,pts,faces,materials,smooth=True)


def bore(group, muzzle, radius, back, key="iron", edge="bronze"):
    profile_x(group,"Forged barrel and open chamber", [
        (back,radius*.27),(back+.045,radius*.77),(back+.12,radius*.94),
        (muzzle-radius*.63,radius*.90),(muzzle-radius*.43,radius*1.14),
        (muzzle,radius*1.13),(muzzle,radius*.74),
        (muzzle-radius*.45,radius*.69),(muzzle-radius*1.25,radius*.55),
        (muzzle-radius*1.58,.004),
    ],[key,key,key,edge,edge,edge,"soot","soot","soot"],cap=True)


def disc(group,name,center,radius,thick,key,*,axis="y",sides=14,edge=None):
    x,y,z = center
    points = {"x": ((x-thick/2,y,z),(x+thick/2,y,z)),
              "y": ((x,y-thick/2,z),(x,y+thick/2,z)),
              "z": ((x,y,z-thick/2),(x,y,z+thick/2))}
    return tube(group,name,points[axis],[radius,radius],key,sides=sides,accent=edge)


def gear(group,radius,teeth,key="bronze"):
    disc(group,"Cast gear wheel",(0,0,0),radius*.86,.053,key,sides=16,edge="gold")
    disc(group,"Axle hub",(0,-.032,0),radius*.30,.048,"iron",sides=10)
    vertices,faces = [],[]
    for i in range(teeth):
        a = i*tau/teeth
        start = len(vertices)
        vertices.extend((r*cos(a+t),y,r*sin(a+t))
                        for y in (-.028,.028)
                        for r,t in ((radius*.80,-.12),(radius,-.093),(radius,.093),(radius*.80,.12)))
        faces.extend(tuple(start+j for j in f) for f in ((0,1,2,3),(7,6,5,4),(0,4,5,1),
                     (1,5,6,2),(2,6,7,3),(3,7,4,0)))
    surface(group,"Cut gear teeth",vertices,faces,[key]*len(faces))


def plaque(group,name,outline,y,depth,key,bevel=.011):
    n = len(outline)
    points = [(x,y+offset,z) for offset in (-depth/2,depth/2) for x,z in outline]
    faces = [tuple(range(n)),tuple(reversed(range(n,2*n)))]
    faces += [(i,i+n,(i+1)%n+n,(i+1)%n) for i in range(n)]
    obj = surface(group,name,points,faces,[key]*len(faces))
    if bevel:
        mod = obj.modifiers.new("Supported edge","BEVEL")
        mod.width,mod.segments = bevel,1
    return obj


def banner(group,key,x,y,z,width,drop):
    tube(group,"Banner crossbar",[(x-width*.62,y,z),(x+width*.62,y,z)],[.019,.019],"oak",7)
    positions = (-.5,-.25,0,.25,.5)
    vertices = [(x+width*u,y+.045*sin(u*8+v*2.7)*v,z-drop*v+drop*.21*abs(u)*v**3)
                for v in (0,.34,.68,1) for u in positions]
    faces = [(row*5+i,row*5+i+1,(row+1)*5+i+1,(row+1)*5+i) for row in range(3) for i in range(4)]
    colors = ["trim" if i%4 in (0,3) else key for i in range(len(faces))]
    surface(group,"Sewn heraldic cloth",vertices,faces,colors,smooth=True)


def blocks(group,x,y,z,width,depth,height,*,key="stone",rows=4,columns=4):
    """A visible front ashlar wall around a recessed inner mass."""
    box(group,"Wall core",(x,y+.025,z+height/2),(width-.03,depth,height),"mortar",.027)
    for row in range(rows):
        for col in range(columns):
            w,h = width/columns,height/rows
            box(group,"Bonded facing stone",(x+(col+.5)*w-width/2,y-depth/2,z+(row+.5)*h),
                (w-.016,.105,h-.016),key if (row+col*2)%4 else "stone-cool",.013)


def pier(group,x,y,z,width,height,*,rows=4):
    for i in range(rows):
        inset = .035*i/rows
        box(group,"Pier masonry",(x,y,z+(i+.5)*height/rows),
            (width-inset,width-inset,height/rows-.010),STONE[i%3],.020)
    box(group,"Pier cap",(x,y,z+height), (width+.068,width+.068,.087),"stone-light",.023)


def footing(group,width,depth):
    box(group,"Weathered foundations",(0,0,.055),(width,depth,.15),"stone-cool",.070)
    box(group,"Dressed upper course",(0,0,.141),(width-.09,depth-.09,.10),"stone",.036)


def ladder(group,x,y,z,height):
    for sign in (-1,1):
        tube(group,"Ladder stringer",[(x+sign*.108,y-.13,z),(x+sign*.108,y,z+height)],
             [.023,.023],"oak",7)
    for i in range(1,7):
        h = i/7
        tube(group,"Ladder rung",[(x-.108,y-.13*(1-h),z+h*height),(x+.108,y-.13*(1-h),z+h*height)],
             [.016,.016],"wood-end",6)


def ballista(group):
    box(group,"Pivot socket",(0,0,.079),(.19,.16,.158),"iron",.023)
    box(group,"Draw rail",(.075,0,.184),(.63,.115,.099),"oak",.016)
    for sign in (-1,1):
        tube(group,"Laminated prod",[(.21,0,.20),(.227,sign*.25,.207),(.159,sign*.46,.232),
                                    (.077,sign*.52,.238)],[.053,.042,.030,.012],"wood",8,"wood-end")
        tube(group,"Tensioned string",[(.077,sign*.52,.238),(-.154,0,.231)],
             [.010,.009],"linen",6)
        disc(group,"Torsion crank",(-.12,sign*.106,.18),.086,.052,"bronze",sides=10)
    tube(group,"Nocked shaft",[(-.26,0,.24),(.35,0,.24)],[.018,.019],"oak",7)
    tube(group,"Steel bolt head",[(.336,0,.24),(.382,0,.24),(.47,0,.24)],
         [.019,.036,.002],"stone-light",sides=8)


def braces(group,a,b,key="oak",width=.048):
    tube(group,"Chamfered brace",[a,b],[width,width],"wood-end" if key=="wood" else key,sides=4,accent="wood")


def deck_rail(group,x,y,z,r,*,skip=()):
    for i in range(10):
        if i in skip:
            continue
        a,b = tau*(i-.37)/10,tau*(i+.37)/10
        at = (x+r*cos(a),y+r*sin(a),z)
        to = (x+r*cos(b),y+r*sin(b),z)
        braces(group,at,to,width=.034)
        braces(group,(at[0],at[1],z-.25),at,width=.034)
