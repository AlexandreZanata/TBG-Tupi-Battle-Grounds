"""Assemble the iron Strongholds and the rear foundry building."""

from math import cos, sin, tau

from clockwork_mechanics import grille, rivets
from mesh import box, surface, tube
from structures import cloth


def chimney(group,x,y,bottom,height,radius):
    top = bottom+height
    tube(group,"Faceted iron foundry smokestack",[(x,y,bottom),(x,y,top-.15)],
         [radius*1.13,radius*.85],"cw-iron",sides=8,accent="cw-steel")
    for z in (bottom+.16,top-.43):
        tube(group,"Brass stack retaining ring",[(x,y,z),(x,y,z+.068)],
             [radius*1.01,radius*.99],"cw-brass-dark",sides=8,accent="cw-brass")
    tube(group,"Blackened recess in the stack mouth",[(x,y,top-.149),(x,y,top-.134)],
         [radius*.76,radius*.76],"cw-char",sides=8)
    chimney_lip(group,x,y,top,radius)


def chimney_lip(group,x,y,top,radius):
    vertices = [(x+r*cos(i*tau/8),y+r*sin(i*tau/8),z)
                for r,z in ((radius*1.04,top-.17),(radius*1.04,top-.055),
                            (radius*.74,top-.055),(radius*.74,top-.16)) for i in range(8)]
    faces = [(band*8+i,band*8+(i+1)%8,(band+1)%4*8+(i+1)%8,(band+1)%4*8+i)
             for band in range(4) for i in range(8)]
    surface(group,"Open annular brass chimney rim",vertices,faces,
            ["cw-brass-dark"]*8+["cw-brass"]*8+["cw-char"]*8+["cw-brass-dark"]*8)


def stronghold(group):
    box(group,"Slate Stronghold foundation",(0,0,.062),(2.55,2.08,.166),"cw-rock-dark",.073)
    box(group,"Riveted steel Stronghold floor",(0,0,.171),(2.35,1.88,.086),"cw-steel",.033)
    box(group,"Recessed iron gatehouse",(0,.084,.769),(1.77,1.49,1.17),"cw-iron-dark",.065)
    for side in (-1,1):
        gatehouse_wing(group,side)
    gate(group)
    box(group,"Thick gatehouse cornice",(0,.077,1.381),(2.13,1.64,.163),"cw-steel",.041)
    for x in (-.68,-.34,0,.34,.68):
        box(group,"Brass-capped front battlement",(x,-.663,1.527),(.258,.255,.219),"cw-iron",.021)
        box(group,"Separate brass battlement cap",(x,-.663,1.645),(.269,.265,.047),"cw-brass",.010)
    tower(group)
    tube(group,"Copper return pipe on the gatehouse",[(.62,.53,1.56),(.85,.53,1.63),
         (1.05,.39,1.50),(1.05,.39,.57)],[.050,.055,.055,.050],"cw-copper",sides=8)


def gatehouse_wing(group,side):
    x = side*.964
    box(group,"Faceted corner gatehouse column",(x,-.437,.845),(.44,.59,1.33),"cw-iron",.055)
    for z in (.307,.918,1.45):
        box(group,"Bolted horizontal gatehouse band",(x,-.452,z),(.49,.63,.094),"cw-brass-dark",.016)
    grille(group,(x,-.761,1.16),.233,.329)
    for z in (.39,.72,1.45):
        rivets(group,(x,-.781,z),.153,.153)


def gate(group):
    for i in range(7):
        x = (i-3)*.091
        box(group,"Separate dark Stronghold gate plank",(x,-.780,.588),
            (.086,.091,.769),"cw-oak",.010)
    for z in (.337,.707):
        box(group,"Brass horizontal gate hinge",(0,-.842,z),(.661,.045,.062),"cw-brass-dark",.009)
    for s in (-1,1):
        box(group,"Cast-iron door jamb",(s*.389,-.820,.596),(.161,.189,.923),"cw-steel",.021)
    box(group,"Segmented brass gate lintel",(0,-.820,1.083),(.937,.211,.160),"cw-brass",.030)
    for i in range(3):
        box(group,"Separate gate threshold step",(0,-1.179+i*.113,.036+i*.033),
            (.88,.165,.084+i*.065),"cw-rock-light",.018)


def tower(group):
    box(group,"Upper iron clock tower",(0,.215,1.999),(1.02,.94,1.10),"cw-iron",.062)
    for x in (-.408,.408):
        box(group,"Brass corner tower upright",(x,-.270,2.006),(.079,.071,.966),"cw-brass-dark",.014)
    grille(group,(0,-.296,2.047),.461,.504)
    box(group,"Machined roof coping",(0,.215,2.571),(1.15,1.06,.142),"cw-brass",.032)
    for x in (-.40,0,.40):
        for y in (-.215,.645):
            box(group,"Separate upper iron merlon",(x,y,2.728),(.239,.196,.242),"cw-iron",.021)
    chimney(group,-.23,.337,2.70,.609,.143)
    chimney(group,.26,.343,2.70,.403,.132)


def standard(group,key):
    tube(group,"Brass keep pennant staff",[(0,.24,2.72),(0,.24,3.76)],
         [.025,.017],"cw-brass",sides=7)
    cloth(group,"Keep pennant",key,(.270,.24,3.704),.54,.267)
    for x in (-.70,.70):
        tube(group,"Short wall pennant bar",[(x-.126,-.802,1.340),(x+.126,-.802,1.340)],
             [.015,.015],"cw-brass",sides=6)
        cloth(group,"Architectural wall pennant",key,(x,-.823,1.339),.230,.437)


def foundry(group):
    box(group,"Lower foundry stylobate",(0,.15,.114),(5.52,2.64,.247),"cw-rock-dark",.070)
    box(group,"Riveted service floor",(0,.15,.284),(5.21,2.39,.143),"cw-steel",.048)
    box(group,"Recessed iron foundry hall",(0,.24,1.527),(4.92,1.91,2.40),"cw-iron-dark",.073)
    for x in (-2.25,-.80,.80,2.25):
        box(group,"Brass vertical foundry pier",(x,-.744,1.633),(.153,.153,2.383),"cw-brass-dark",.023)
        rivets(group,(x,-.835,2.80),.046,.08)
    box(group,"Dark plate behind the three drive wheels",(0,-.822,1.662),
        (4.64,.146,2.221),"cw-char",.042)
    for x in (-1.61,0,1.61):
        tube(group,"Stationary drive axle mount",[(x,-.89,1.77),(x,-1.151,1.77)],
             [.188,.188],"cw-steel",sides=10)
    for x in (-1.89,0,1.89):
        grille(group,(x,-.863,.463),.618,.281)
    roof(group)
    for i in range(4):
        box(group,"Broad foundry service stair",(0,-1.53+i*.136,.031+i*.034),
            (1.38,.240,.071+i*.068),"cw-rock-light",.022)
    for x,height,r in ((-1.85,1.19,.239),(-1.23,.82,.211),(1.69,1.45,.267)):
        chimney(group,x,.82,2.76,height,r)


def roof(group):
    box(group,"Brass foundry roof cornice",(0,.17,2.826),(5.13,2.247,.153),"cw-brass",.036)
    for i in range(4):
        x = -2.38+i*1.19
        section = ((x,2.912),(x+.934,3.226),(x+1.133,2.912))
        verts = [(u,y,z) for y in (-.819,1.232) for u,z in section]
        surface(group,"Individual sawtooth roof bay",verts,
                [(2,1,0),(3,4,5),(0,1,4,3),(1,2,5,4),(2,0,3,5)],
                ["cw-steel","cw-iron","cw-iron","cw-steel","cw-iron-dark"])
        box(group,"Amber clerestory below a sawtooth roof",(x+.72,-.858,2.981),
            (.517,.045,.100),"cw-amber",.008)
