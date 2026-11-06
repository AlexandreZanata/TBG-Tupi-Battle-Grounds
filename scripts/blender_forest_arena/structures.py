"""Model the Stronghold and interchangeable architectural props."""

from math import cos, pi, sin, tau

from mesh import box, pebble, surface, tube


def ring(group,name,x,y,z,r,height,key,sides=10,accent=None):
    tube(group,name,[(x,y,z),(x,y,z+height)],[r,r],key,sides=sides,accent=accent)


def arch(group, y):
    for side in (-1,1):
        for row in range(3):
            box(group,"Arch jamb",(side*.368,y,.17+.24*row),(.18,.13,.235),
                "stone-light" if row%2 else "stone",.018)
    segments = 9
    for i in range(segments):
        a,b = (i+.07)*pi/segments,(i+.93)*pi/segments
        profile = [(r*cos(t),.69+r*sin(t)) for r,t in ((.28,a),(.45,a),(.45,b),(.28,b))]
        vertices = [(x,y+d,z) for d in (-.076,.076) for x,z in profile]
        surface(group,"Cut arch voussoir",vertices,[(0,1,2,3),(7,6,5,4),(0,4,5,1),
                (1,5,6,2),(2,6,7,3),(3,7,4,0)],["stone-light" if i==4 else "stone"]*6)
    for x in (-.20,-.10,0,.10,.20):
        height = .66+.22*(1-(x/.28)**2)**.5
        box(group,"Recessed gate plank",(x,y+.061,height/2),(.096,.08,height),
            "wood-dark" if x==0 else "oak",.009)
    for z in (.24,.57):
        box(group,"Gate iron strap",(0,y+.015,z),(.52,.035,.043),"iron",.005)


def turret(group, x, y, radius, height):
    foot = .17
    tube(group,"Battered tower plinth",[(x,y,foot-.14),(x,y,foot+.10)],
         [radius*1.23,radius],"stone-cool",sides=10,accent="stone")
    rings = 7
    for row in range(rings):
        z = foot+.06+row*(height-foot-.06)/rings
        r = radius*(1-.11*row/rings)
        ring(group,"Dressed turret course",x,y,z,r,(height-foot-.06)/rings-.012,
             "stone" if row%3 else "stone-cool",10,"stone-light")
    ring(group,"Projecting turret cornice",x,y,height-.035,radius*1.12,.13,"stone-light")
    for i in range(7):
        a = tau*i/7
        pos = (x+radius*.91*cos(a),y+radius*.91*sin(a),height+.19)
        box(group,"Crenellation",pos,(radius*.30,radius*.53,.28),"stone",.027,a)
    for z in (height*.48,height*.76):
        box(group,"Recessed arrow slit",(x,y-radius*.90-.010,z),(.082,.036,.24),"mortar",.012)
        box(group,"Arrow slit sill",(x,y-radius*.93,z-.124),(.16,.13,.043),"stone-light",.009)


def stronghold(group):
    box(group,"Chiseled foundation slab",(0,0,.045),(2.54,2.02,.16),"stone-cool",.11)
    box(group,"Inset foundation course",(0,0,.141),(2.39,1.87,.12),"stone",.055)
    box(group,"Recessed keep shell",(0,.11,.86),(1.75,1.34,1.50),"mortar",.043)
    for row in range(6):
        z = .22+row*.235
        for side in (-1,1):
            box(group,"Shouldered front ashlar",(side*(.62+.015*(row%2)),-.65,z),
                (.38,.24,.223),"stone" if (row+side)%3 else "stone-cool",.025)
            for index in range(3):
                y = -.38+.43*index+.021*(row%2)
                box(group,"Bonded side ashlar",(side*.866,y,z),(.18,.417,.223),
                    ("stone","stone-light","stone-cool")[(row+index+side)%3],.018)
        if row>=4:
            box(group,"Lintel masonry",(0,-.65,z),(.86,.24,.223),"stone",.025)
    arch(group,-.818)
    for step in range(3):
        box(group,"Gate stair",(0,-1.21+.108*step,.033+.032*step),
            (.71,.17,.086+.064*step),"stone-light",.016)
    box(group,"Keep wall walk",(0,.085,1.57),(1.95,1.57,.20),"stone-light",.045)
    for x in (-.65,-.23,.23,.65):
        box(group,"Keep parapet block",(x,-.63,1.77),(.26,.19,.30),"stone",.027)
    turret(group,-.88,-.57,.40,1.82)
    turret(group,.88,-.57,.40,1.82)
    turret(group,0,.37,.66,2.83)
    ring(group,"Central tower deck",0,.37,2.965,.45,.035,"wood-dark",10)


def cloth(group,name,key,center,width,height):
    x,y,z = center
    vertices = [(x+width*t,y+.046*sin(t*7+r*1.6),z-r*height-(.1*height if r==1 and abs(t)<.12 else 0))
                for r in (0,.39,.73,1) for t in (-.5,-.25,0,.25,.5)]
    faces = [(r*5+i,r*5+i+1,(r+1)*5+i+1,(r+1)*5+i) for r in range(3) for i in range(4)]
    surface(group,name,vertices,faces,[key]*len(faces),smooth=True)


def heraldry(group,key):
    tube(group,"Keep flagstaff",[(0,.37,2.94),(0,.37,3.89)],[.025,.019],"iron",sides=7)
    tube(group,"Flagstaff finial",[(0,.37,3.85),(0,.37,3.94)],[.043,.016],"trim",sides=8)
    cloth(group,"Keep pennant",key,(.265,.37,3.83),.53,.32)
    for x in (-.89,.89):
        tube(group,"Banner crossbar",[(x-.16,-.95,1.40),(x+.16,-.95,1.40)],
             [.023,.023],"wood-dark",sides=7)
        cloth(group,"Heraldic wall cloth",key,(x,-.964,1.397),.28,.54)


def torch(group):
    box(group,"Brazier plinth",(0,0,.07),(.45,.45,.14),"stone-cool",.05)
    box(group,"Brazier pedestal",(0,0,.225),(.30,.30,.26),"stone",.043)
    tube(group,"Forged fire bowl",[(0,0,.31),(0,0,.39),(0,0,.45)],[.12,.23,.205],"iron",8)
    pebble(group,"Glowing coals",(0,0,.45),(.17,.17,.037),"ember",accent="fire")
    for x,y,height in ((-.08,.02,.23),(.06,.04,.31),(.03,-.068,.19)):
        tube(group,"Flame tongue",[(x,y,.43),(x+.035,y,.54),(x+.022,y+.03,.46+height)],
             [.066,.047,.004],"fire",sides=6,accent="glow")


def fence(group):
    for x in (-.60,.60):
        box(group,"Split fence post",(x,0,.27),(.15,.17,.54),"oak",.024,x*.04)
        box(group,"Post end grain",(x,0,.535),(.135,.155,.027),"wood-end",.012,x*.04)
    for z in (.22,.38):
        box(group,"Shaped fence rail",(0,0,z),(1.20,.098,.096),"wood",.014,.015)


def cargo(group):
    box(group,"Crate body",(-.20,0,.203),(.44,.40,.407),"wood-dark",.025)
    for x in (-.35,-.20,-.05):
        box(group,"Crate face plank",(x,-.205,.20),(.141,.032,.35),"wood",.009)
    for z in (.062,.339):
        box(group,"Crate face strap",(-.20,-.23,z),(.445,.041,.058),"wood-end",.008)
    tube(group,"Staved supply barrel",[(.229,.060,.016),(.229,.06,.12),(.229,.06,.33),(.229,.06,.415)],
         [.151,.185,.181,.154],"wood",sides=10,accent="oak")
    for z in (.083,.347):
        ring(group,"Barrel iron band",.229,.06,z,.180,.049,"iron")
    ring(group,"Barrel end grain",.229,.06,.408,.135,.010,"wood-end")


def lighthouse(group):
    tube(group,"Beacon stone foot",[(0,0,.03),(0,0,.23)],[.50,.43],"stone-cool",10,"stone-light")
    for row in range(8):
        z = .22+row*.225
        radius = .407-row*.020
        tube(group,"Tapered beacon course",[(0,0,z),(0,0,z+.225)],
             [radius,radius-.020],"red" if row in (2,5) else "linen",12)
    ring(group,"Lantern gallery",0,0,1.99,.445,.12,"stone-light",12)
    ring(group,"Warm beacon lens",0,0,2.11,.219,.30,"glow",10)
    for i in range(8):
        a = tau*i/8
        x,y = .263*cos(a),.263*sin(a)
        tube(group,"Lantern mullion",[(x,y,2.10),(x,y,2.43)],[.026,.025],"iron",6)
    tube(group,"Copper beacon roof",[(0,0,2.40),(0,0,2.49),(0,0,2.80)],[.365,.329,.02],
         "red",10,accent="ember")


def jetty(group):
    for x in (-.38,.38):
        for y in (-.71,-.05,.70):
            tube(group,"Pier piling",[(x,y,-.92),(x,y,.10)],[.088,.082],"oak",sides=7)
    for i in range(10):
        box(group,"Pier tread",(0,-.80+i*.18,.027),(.95,.164,.089),"wood" if i%3 else "wood-end",.015)


def sailboat(group):
    points = [(-.42,.70),(-.42,-.37),(-.23,-.84),(0,-1.12),(.23,-.84),(.42,-.37),(.42,.70)]
    vertices = [(x*r,y*r,z) for r,z in ((.67,-.16),(1,.12),(.87,.12),(.78,.015)) for x,y in points]
    n = len(points)
    faces = [(r*n+i,r*n+(i+1)%n,(r+1)*n+(i+1)%n,(r+1)*n+i)
             for r in range(3) for i in range(n)]
    faces.append(tuple(range(3*n,4*n)))
    surface(group,"Carvel hull and interior",vertices,faces,["oak"]*n+["wood-end"]*n+["wood"]*(n+1))
    box(group,"Boat bench",(0,.37,.08),(.70,.14,.075),"wood-end",.013)
    tube(group,"Sail mast",[(0,-.08,.03),(0,-.08,1.71)],[.031,.020],"wood-dark",7)
    vertices = [(0,-.08,1.65),(.19,-.39,1.12),(.21,-.68,.46),
                (-.07,-.11,.35),(.14,.47,.39),(.16,.21,.94)]
    surface(group,"Wind-filled linen sail",vertices,[(0,1,5),(1,2,3,5),(3,4,5)],
            ["linen","linen","sand-light"],smooth=True)
    tube(group,"Lower sail boom",[(-.07,-.73,.34),(-.07,.48,.34)],[.019,.018],"oak",7)
