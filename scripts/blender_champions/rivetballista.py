"""Build the rivet automaton from separate armor, feed rails and cartridges."""

from bloomkeeper import leaf
from completion_people import limbs


def rivetballista(m,tier,branch,*,production=False):
    guide = tier > 0 and branch == 0
    storm = tier > 0 and branch == 1
    panel = "star_ceramic" if guide else "steel_dark" if storm else "brass"
    edge = "prism_ice" if guide else "copper"
    limbs(m,"slag",panel,arm=.22,leg=.50,width=.059,skin="iron")
    m.loft("Riveted torso casing","Chest",panel,[
        (-.044,.082,.078,0),(.066,.117,.102,.001),(.200,.157,.110,.002),
        (.288,.152,.085,.007),(.336,.080,.068,0),
    ],n=10)
    m.tube("Piston neck","Chest","slag",[(0,.294,0),(0,.412,0)],[.074,.063],n=10)
    m.box("Counterweight hips","Hips","slag",(0,-.005,0),(.239,.108,.177),bevel=.022)
    m.box("Beveled forge helmet","Head",panel,(0,.022,.037),(.232,.177,.244),bevel=.026)
    m.box("Inset visor slot","Head","ink",(0,.039,.156),(.163,.037,.026),bevel=.007)
    m.box("Hot optical slit","Head","forge",(0,.039,.169),(.130,.012,.013),bevel=.003)
    for s,side in ((-1,"R"),(1,"L")):
        m.ell("Armature shoulder joint","Chest","iron",(s*.157,.286,0),(.073,.067,.070),segments=10)
        m.box("Piston guard",f"Shin{side}",edge,(0,-.112,.044),(.072,.185,.073),bevel=.013)
        m.ell("Armor rivet","Chest",edge,(s*.088,.215,.097),(.025,.024,.023),segments=9)
        if guide and tier==2:
            leaf(m,"Ceramic aiming antenna","Head",panel,(s*.074,.089,.017),(s*.111,.194,-.059),.027)
    rail(m,panel,edge,tier,guide,storm)
    for x in (-.089,0,.089) if storm else (0,):
        top=.435 if storm and tier==2 else .345
        m.box("Back-fed rivet magazine","BackSpring",edge,(x,top*.47,-.079),(.070,top,.093),bevel=.011)
        m.box("Individual magazine feed cap","BackSpring",panel,(x,top,-.079),(.079,.047,.104),bevel=.009)


def rail(m,panel,edge,tier,guide,storm):
    n="WeaponR"
    end=.42 if guide else .35
    m.tube("Stock locked to WeaponR",n,"slag",[(0,0,-.119),(0,0,.245)],[.039,.043],n=9)
    for s in (-1,1):
        m.box("Separate machined guide rail",n,panel,(s*.044,.011,.055+end*.45),
              (.049,.066,end-.08),bevel=.009)
        m.tube("Curved sprung ballista limb",n,edge,[(s*.022,0,.206),(s*.101,.015,.224),
               (s*(.167+.033*guide),.029,.188)],[.029,.022,.012],n=8)
        if guide:
            m.tube("Ice-blue guide rail standoff",n,edge,[(s*.043,.015,.132),
                   (s*.056,.058,.157),(s*.043,.055,.316)],[.016,.017,.013],n=7)
        if storm and tier==2:
            m.box("Copper rail exhaust shield",n,edge,(s*.077,0,.135),(.041,.075,.178),bevel=.010)
    m.rim("Open rivet exit at the native origin",n,edge,(0,0,end),(.039,.035),.010,segments=12)
    m.tube("Forward split housing",n,panel,[(0,0,end-.092),(0,0,end-.010)],[.054,.048],n=10)
