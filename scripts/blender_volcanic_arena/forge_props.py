"""Model the forge architecture, industrial props and volcanic landmarks."""

from math import cos, sin, tau
from random import Random

from mesh import box, material, pebble, surface, tube
from structures import arch, heraldry
from terrain import rows


def stronghold(group):
    box(group,"Black basalt foundation",(0,0,.051),(2.55,2.02,.181),"basalt-deep",.084)
    box(group,"Foundry raised plinth",(0,0,.143),(2.36,1.82,.117),"basalt-cut",.052)
    box(group,"Sooted keep core",(0,.103,.829),(1.80,1.36,1.45),"basalt-deep",.061)
    for row in range(5):
        z=.259+row*.246
        for side in (-1,1):
            box(group,"Basalt gatehouse course",(side*(.617+.014*(row%2)),-.636,z),
                (.408,.261,.231),"basalt" if row%2 else "basalt-cut",.026)
            for i in range(3):
                box(group,"Interlocked side block",(side*.866,-.36+i*.43,z),
                    (.176,.408,.229),"basalt" if (row+i)%3 else "basalt-cut",.019)
        if row>=3:
            box(group,"Gate lintel bed",(0,-.645,z),(.885,.252,.235),"basalt",.028)
    gate_masonry(group)
    for i in range(3):
        box(group,"Blackened stone entrance step",(0,-1.19+.105*i,.028+.028*i),
            (.767,.171,.082+.059*i),"forge-curb",.015)
    box(group,"Reinforced parapet beam",(0,.072,1.555),(1.98,1.57,.154),"foundry-iron",.028)
    for x in (-.59,0,.59):
        box(group,"Copper-edged battlement",(x,-.663,1.685),(.34,.238,.250),"basalt-cut",.033)
        box(group,"Parapet band",(x,-.667,1.802),(.35,.247,.043),"foundry-copper",.010)
    chimney(group,0,.37,.56,2.73)
    for x in (-.91,.91):
        chimney(group,x,-.57,.351,1.59)


def gate_masonry(group):
    original=set(group.objects)
    arch(group,-.823)
    finishes={"stone-light":"foundry-brass","stone":"basalt-cut","oak":"tempered-wood",
              "wood-dark":"charred-timber","iron":"foundry-iron"}
    for obj in set(group.objects)-original:
        for slot in obj.material_slots:
            if slot.material.name in finishes:
                slot.material=material(finishes[slot.material.name])


def chimney(group,x,y,r,top):
    tube(group,"Octagonal forged tower",[(x,y,.141),(x,y,top*.54),(x,y,top-.21)],
         [r*1.14,r,r*.88],"basalt-deep",sides=8,accent="basalt-cut")
    for z in (top*.36,top*.59,top-.228):
        tube(group,"Copper tower binding",[(x,y,z),(x,y,z+.063)],[r*1.03,r*.99],
             "foundry-copper",sides=8)
    box(group,"Recessed forge slot",(x,y-r*.92,top*.69),(r*.33,.055,r*.52),"soot",.011)
    box(group,"Narrow heat in the slot",(x,y-r*.96,top*.678),(r*.145,.021,r*.37),"ember-seam",.007)
    tube(group,"Forged chimney cap",[(x,y,top-.215),(x,y,top-.097),(x,y,top+.15)],
         [r*1.14,r*1.06,r*.62],"foundry-iron",sides=8,accent="foundry-copper")
    tube(group,"Copper chimney flue",[(x,y,top+.137),(x,y,top+.246)],
         [r*.53,r*.44],"foundry-copper",sides=8)
    tube(group,"Shadow recessed into chimney opening",[(x,y,top+.241),(x,y,top+.251)],
         [r*.344,r*.344],"soot",sides=8)


def standard(group,seat):
    heraldry(group,seat)


def basalt_pipes(group):
    for i,(x,y,height,radius) in enumerate(((-.40,-.07,.81,.174),(-.16,.12,1.26,.187),
        (.08,.02,1.49,.219),(.36,.13,1.14,.185),(.48,-.12,.69,.157),(-.20,-.25,.61,.151),
    )):
        lean=.044*sin(i*2)
        tube(group,"Individual exposed basalt pipe",[(x,y,-.05),(x+lean,y,.41*height),
             (x+lean*.7,y-.032,height)],[radius*1.10,radius,radius*.88],
             "basalt" if i%2 else "basalt-deep",sides=6,accent="basalt-cut")
        tube(group,"Broken top course on basalt pipe",[(x+lean*.7,y-.032,height-.018),
             (x+lean*.7,y-.032,height+.042)],[radius*.88,radius*.74],"basalt-cut",sides=6)


def slag(group):
    for i,(x,y,rx,ry,rz) in enumerate(((-.21,0,.37,.26,.45),(.21,.065,.35,.25,.29),
                                      (.05,-.20,.24,.16,.24))):
        pebble(group,"Layered cooled slag block",(x,y,rz*.45),(rx,ry,rz),
               "cooled-slag",sides=7,accent="basalt-cut",phase=i*1.08)


def fissure(group):
    coords=((-.62,-.025,.023),(-.34,.043,.026),(-.16,-.064,.025),(.10,.009,.025),
            (.32,-.041,.022),(.63,.042,.021))
    tube(group,"Low bed of molten rock",coords,[.039,.061,.056,.047,.049,.021],"magma-edge",sides=5)
    tube(group,"Heated seam in the middle of the fissure",
         [(x,y,z+.035) for x,y,z in coords],[.014,.021,.018,.017,.018,.005],"magma-bright",sides=5)


def scorched_branch(group):
    tube(group,"Felled heat-weathered trunk",[(0,0,-.026),(-.049,.012,.54),(-.093,.019,1.11)],
         [.094,.063,.026],"charred-timber",sides=7)
    for x,y,z,end in ((-.06,.026,.39,(.25,.045,.76)),(-.06,.016,.61,(-.34,.124,1.0)),
                      (-.075,.019,.82,(.16,.098,1.17))):
        tube(group,"Broken dry branch",[(x,y,z),((x+end[0])*.5,y+.017,z+.14),end],
             [.048,.031,.008],"charred-timber",sides=6,accent="tempered-wood")


def grass(group):
    rng=Random(48)
    for _ in range(6):
        x,y=rng.uniform(-.20,.20),rng.uniform(-.16,.14)
        h=rng.uniform(.10,.25)
        tube(group,"Dry wind-swept ash grass",[(x,y,0),(x+.058,y-.017,h*.67),(x+.085,y-.028,h)],
             [.009,.007,.002],"dry-grass",sides=5)


def rail(group):
    for x in (-.55,.55):
        box(group,"Riveted iron rail post",(x,0,.25),(.134,.147,.50),"foundry-iron",.021)
        box(group,"Copper post head",(x,0,.516),(.149,.160,.055),"foundry-copper",.013)
    for z in (.188,.365):
        box(group,"Straight forged rail",(0,0,z),(1.13,.059,.079),"foundry-iron",.014)
        box(group,"Copper narrow rail edge",(0,-.035,z+.027),(1.04,.026,.022),"foundry-brass",.006)


def brazier(group):
    box(group,"Basalt fire plinth",(0,0,.078),(.431,.438,.163),"basalt-deep",.046)
    tube(group,"Brass pedestal for the forge bowl",[(0,0,.133),(0,0,.338)],
         [.144,.101],"foundry-copper",sides=8)
    tube(group,"Thick forged fire bowl",[(0,0,.312),(0,0,.398),(0,0,.453)],
         [.137,.215,.196],"foundry-iron",sides=9,accent="foundry-brass")
    pebble(group,"Bed of incandescent coal",(0,0,.446),(.170,.159,.037),"ember-seam",accent="magma-bright")
    for x,y,h in ((-.067,.027,.212),(.075,.041,.167),(.025,-.050,.246)):
        tube(group,"Separate flame tongue",[(x,y,.463),(x+.027,y-.018,.497+h*.52),
             (x+.011,y+.018,.451+h)],[.049,.034,.005],"fire",sides=6,accent="glow")


def anvil(group):
    box(group,"Anvil stone footing",(0,0,.096),(.488,.386,.197),"basalt-cut",.033)
    box(group,"Pinned steel waist",(0,0,.221),(.292,.214,.113),"foundry-iron",.023)
    box(group,"Shaped anvil work surface",(0,0,.324),(.421,.241,.138),"foundry-iron",.022)
    tube(group,"Tapered anvil horn",[(.15,0,.329),(.259,0,.332),(.363,0,.364)],
         [.105,.078,.008],"foundry-iron",sides=6,accent="foundry-brass")
    box(group,"Separate hammered edge plate",(-.012,-.122,.354),(.392,.033,.057),"basalt-cut",.012)


def crust(group):
    pebble(group,"Floating irregular slag crust",(0,0,-.020),(.666,.427,.176),
           "cooled-slag",sides=7,accent="sintered-crust",phase=.24)
    pebble(group,"Darker recently cooled slag cap",(-.069,.027,.045),(.462,.293,.072),
           "basalt-deep",sides=7,accent="basalt",phase=.31)


def vent(group):
    for i in range(5):
        a=(i+.13)*tau/5
        x,y=.71*cos(a),.43*sin(a)
        tube(group,"Staggered ring of basalt vent stone",[(x,y,-.09),(x,y,.31+i*.063),
             (x*.92,y*.88,.46+i*.065)],[.25,.21,.16],"cooled-slag",sides=6,accent="basalt-cut")
    pebble(group,"Warm mineral pool in the vent",(0,0,.022),(.59,.381,.084),
           "magma-edge",sides=9,accent="magma-bright")


def crater(group):
    profile=((1,0),(1.03,.38),(.71,1.05),(.48,2.42),(.39,2.68),(.31,2.58),(.25,1.96))
    angles=[tau*i/13 for i in range(13)]
    rings=[[(3.10*scale*cos(a)+.17*z,2.04*scale*sin(a)+.11*z,
             z+.24*sin(a*3+j*.31)+.11*cos(a*5))
            for a in angles] for j,(scale,z) in enumerate(profile)]
    keys=("smoke-ridge","basalt-cut","basalt","basalt-deep","sintered-crust","cooled-slag")
    rows(group,"Broken volcano profile with open caldera",rings,
         lambda band,i:"smoke-ridge-light" if band<3 and i%5==0 else keys[band])
    for i,width in ((9,.29),(10,.18)):
        molten_runoff(group,rings,i,width)
    pebble(group,"Lava lake recessed in the caldera",(.32,.20,2.03),(.89,.57,.146),
           "magma-edge",sides=11,accent="magma-bright",phase=.13)
    for x,y,r,height in ((-1.72,.57,.30,.81),(1.49,.54,.43,1.11),(2.28,.31,.27,.72)):
        tube(group,"Stacked basalt rock around the caldera",[(x,y,-.05),(x+.11,y,.49),
             (x+.16,y,height)],[r*1.42,r,r*.64],"smoke-ridge",sides=7,accent="basalt-cut")



def molten_runoff(group,rings,index,width):
    points=[]
    for j,ring in enumerate(rings[:5]):
        a,b=ring[index],ring[index+1]
        center=.39+.13*sin(index+j*.91)
        for t in (center-width/2,center+width/2):
            x,y,z=(u+(v-u)*t for u,v in zip(a,b,strict=True))
            points.append((x,y-.027,z+.023))
    faces=[(i,i+1,i+3,i+2) for i in range(0,len(points)-2,2)]
    surface(group,"Molten runnel climbing the fractured caldera wall",points,faces,
            ["ember-seam","magma-edge","magma-bright","magma-edge"])
