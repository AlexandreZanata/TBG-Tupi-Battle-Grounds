"""Model the banded Strongholds and the valley's static lightning conductors."""

from math import cos, sin, tau

from mesh import box, pebble, surface, tube
from structures import cloth


def ring(group, name, at, radius, thickness, key, accent=None):
    x,y,z = at
    segments,sides = 16,6
    vertices = []
    for i in range(segments):
        a = i*tau/segments
        for j in range(sides):
            b = j*tau/sides
            r = radius+thickness*cos(b)
            vertices.append((x+r*cos(a),y+thickness*sin(b),z+r*sin(a)))
    faces = [(i*sides+j,i*sides+(j+1)%sides,(i+1)%segments*sides+(j+1)%sides,
              (i+1)%segments*sides+j) for i in range(segments) for j in range(sides)]
    keys = [accent if accent and i%sides in (1,2) else key for i in range(len(faces))]
    surface(group,name,vertices,faces,keys)


def inlay(group, points, key="storm-glow"):
    tube(group,"Fixed lightning-shaped metal inlay",points,[.027]*len(points),key,sides=5)


def stronghold(group):
    box(group,"Split-slate Stronghold foundation",(0,0,.052),(2.57,2.11,.169),"storm-slate-deep",.073)
    box(group,"Silver coping across the foundation",(0,0,.157),(2.40,1.95,.094),"storm-shale",.033)
    box(group,"Recessed watch hall",(0,.086,.721),(1.88,1.47,1.20),"storm-char",.068)
    for s in (-1,1):
        wall(group,s)
        turret(group,s)
    gate(group)
    box(group,"Upper parapet walk",(0,.03,1.353),(2.14,1.56,.182),"storm-slate-light",.062)
    for x in (-.69,-.35,0,.35,.69):
        box(group,"Separately dressed front crenellation",(x,-.690,1.485),
            (.244,.267,.247),"storm-slate",.021)
    lookout(group)


def wall(group, side):
    for row in range(4):
        z = .289+row*.225
        box(group,"Staggered slate beside the gate",(side*(.63+.032*(row%2)),-.697,z),
            (.49,.203,.219),"storm-slate" if row%3 else "storm-slate-light",.017)
        for i in range(3):
            box(group,"Staggered sidewall block",(side*.953,-.41+i*.435,z),
                (.218,.419,.218),"storm-slate-edge" if (row+i)%3 else "storm-slate",.017)
    tube(group,"Copper lightning drain on the watch hall",[(side*.90,-.807,.29),
         (side*.90,-.807,1.06),(side*.78,-.800,1.23)],[.021,.022,.019],"storm-copper",sides=7)


def turret(group, side):
    x,y = side*.963,-.511
    tube(group,"Faceted corner watch turret",[(x,y,.17),(x,y,.92),(x,y,1.59)],
         [.329,.301,.258],"storm-slate-deep",sides=8,accent="storm-slate")
    for z,r in ((.37,.335),(1.03,.313),(1.57,.287)):
        tube(group,"Separate dressed turret band",[(x,y,z),(x,y,z+.064)],
             [r,r*.97],"storm-slate-light",sides=8)
    tube(group,"Seamed corner turret cap",[(x,y,1.64),(x,y,1.70),(x,y,1.89)],
         [.306,.301,.081],"storm-iron",sides=8,accent="storm-steel")
    box(group,"Amber corner observation slit",(x,y-.286,1.281),(.085,.032,.214),"storm-amber",.011)


def gate(group):
    box(group,"Granite lintel above the oak doors",(0,-.817,.921),(.839,.255,.154),"storm-slate-light",.023)
    for s in (-1,1):
        box(group,"Beveled granite door jamb",(s*.353,-.826,.494),
            (.164,.191,.813),"storm-slate-light",.021)
        for i in range(3):
            x = s*(.052+i*.098)
            box(group,"Storm-season oak gate board",(x,-.817,.503),(.095,.093,.695),"storm-oak",.006)
        box(group,"Bronze face hinge",(s*.158,-.88,.587),(.255,.034,.050),"storm-brass",.010)
    for i in range(3):
        box(group,"Shallow slate entrance step",(0,-1.16+i*.109,.03+i*.028),
            (.82,.157,.080+i*.056),"storm-slate-light",.018)


def lookout(group):
    y = .293
    tube(group,"Tapered main storm watchtower",[(0,y,1.32),(0,y,1.91),(0,y,2.46)],
         [.565,.491,.391],"storm-slate-deep",sides=8,accent="storm-slate-edge")
    for z,r in ((1.58,.546),(1.89,.516),(2.32,.454)):
        tube(group,"Banded tower course",[(0,y,z),(0,y,z+.070)],[r,r*.97],"storm-slate-light",sides=8)
    box(group,"Recessed lookout window",(0,-.165,2.105),(.270,.053,.325),"storm-char",.028)
    box(group,"Amber light deep inside the lookout",(0,-.195,2.106),(.112,.017,.201),"storm-amber",.009)
    tube(group,"Eight folded plates of the tower cap",[(0,y,2.427),(0,y,2.518),(0,y,2.747)],
         [.518,.488,.207],"storm-iron",sides=8,accent="storm-steel")
    tube(group,"Fork stem below the tower finial",[(0,y,2.691),(0,y,3.21)],[.063,.033],"storm-brass",sides=7)
    ring(group,"Copper storm ring finial",(0,y,3.114),.235,.030,"storm-copper","storm-brass")
    inlay(group,[(-.042,y-.036,3.23),(.060,y-.035,3.14),(-.021,y-.035,3.10),
                 (.048,y-.035,3.00)],"storm-glass")


def standard(group, key):
    tube(group,"Crosswind pennant mast",[(0,.29,3.28),(.06,.29,3.79)],
         [.022,.016],"storm-iron",sides=7)
    cloth(group,"Long pennant pulled by the valley wind",key,(.348,.29,3.753),.575,.268)
    for x in (-.93,.93):
        tube(group,"Stone wall banner crossbar",[(x-.142,-.828,1.382),(x+.142,-.828,1.382)],
             [.014,.014],"storm-brass",sides=6)
        cloth(group,"Architectural wind banner",key,(x,-.848,1.384),.241,.420)


def conductor(group):
    for z,width,height in ((.073,.83,.157),(.204,.68,.136),(.337,.54,.156)):
        box(group,"Terraced lightning conductor pedestal",(0,0,z),(width,width*.84,height),
            "storm-slate-deep" if z<.20 else "storm-slate-light",.033)
    tube(group,"Chamfered conductor obelisk",[(0,0,.383),(0,0,1.37),(0,0,1.72)],
         [.239,.178,.115],"storm-slate",sides=8,accent="storm-slate-light")
    for z,r in ((.55,.243),(1.18,.215),(1.58,.177)):
        tube(group,"Copper insulating collar",[(0,0,z),(0,0,z+.069)],
             [r,r*.94],"storm-copper",sides=8,accent="storm-brass")
    inlay(group,[(-.061,-.217,.63),(.101,-.219,.831),(-.048,-.204,.976),(.055,-.189,1.115)])
    for s in (-1,1):
        tube(group,"Copper fork leg",[(s*.077,0,1.61),(s*.306,0,1.81),(s*.334,0,2.06)],
             [.043,.038,.029],"storm-copper",sides=7)
    ring(group,"Separate outer copper storm ring",(0,0,2.061),.334,.039,"storm-copper","storm-brass")
    ring(group,"Glazed inner insulator ring",(0,-.035,2.061),.227,.021,"storm-cyan","storm-glass")
    inlay(group,[(.071,-.056,2.21),(-.050,-.056,2.09),(.041,-.056,2.06),(-.071,-.056,1.93)])


def relay_plinth(group):
    box(group,"Relay lower split-stone podium",(0,0,.125),(5.00,1.88,.267),"storm-slate-deep",.088)
    box(group,"Raised relay service deck",(0,0,.279),(4.68,1.61,.162),"storm-slate-edge",.058)
    for i in range(4):
        box(group,"Recessed service stair",(0,-1.37+i*.168,.046+i*.043),
            (1.28,.246,.095+i*.083),"storm-slate-light",.021)
    for x in (-1.92,1.92):
        box(group,"Copper gutter at edge of service deck",(x,-.47,.375),(.079,.58,.052),"storm-copper",.012)
    for s in (-1,1):
        tube(group,"Service feed into the central conductor",[(s*2.12,.25,.391),(s*1.66,.25,.407),
             (s*.79,-.18,.408),(0,-.18,.409)],[.029,.032,.031,.028],"storm-copper",sides=7)
        for j in range(3):
            box(group,"Silver warning edge on the relay floor",(s*(.66+j*.32),-.746,.375),
                (.177,.053,.030),"storm-steel",.007,angle=s*.26)


def electric_lamp(group):
    box(group,"Low slate socket",(0,0,.060),(.355,.352,.124),"storm-slate-deep",.025)
    tube(group,"Insulated lamp stem",[(0,0,.121),(0,0,.317),(0,0,.448)],
         [.086,.060,.111],"storm-iron",sides=7,accent="storm-steel")
    tube(group,"Amber glow tube",[(0,0,.438),(0,0,.605)],[.081,.076],"storm-amber",sides=7)
    for x,y in ((-.089,0),(.089,0),(0,-.089),(0,.089)):
        tube(group,"Protective copper lamp cage",[(x,y,.432),(x,y,.607)],
             [.015,.014],"storm-copper",sides=6)
    tube(group,"Separate swept rain hood",[(0,0,.596),(0,0,.625),(.039,0,.718)],
         [.147,.145,.024],"storm-iron",sides=7,accent="storm-steel")


def railing(group):
    for x in (-.52,0,.52):
        box(group,"Independent slate rail base",(x,0,.041),(.207,.207,.089),"storm-slate-deep",.018)
        tube(group,"Riveted rail post",[(x,0,.072),(x,0,.423)],[.045,.034],
             "storm-iron",sides=6,accent="storm-steel")
        pebble(group,"Copper rail fastener",(x,-.04,.335),(.029,.014,.026),"storm-copper",sides=6)
    for z in (.233,.373):
        box(group,"Thin iron rail bar",(0,0,z),(1.08,.046,.043),"storm-iron",.007)


def ground_coil(group):
    box(group,"Slate base of a low grounding coil",(0,0,.069),(.48,.43,.148),"storm-slate-deep",.027)
    tube(group,"Glazed ceramic ground insulator",[(0,0,.132),(0,0,.392)],
         [.101,.083],"storm-glass",sides=8)
    for i in range(4):
        z = .171+i*.056
        tube(group,"Separate grounding coil collar",[(0,0,z),(0,0,z+.025)],
             [.120,.116],"storm-copper",sides=9)
    tube(group,"Rounded ground coil terminal",[(0,0,.391),(0,0,.458)],
         [.097,.046],"storm-steel",sides=8)
