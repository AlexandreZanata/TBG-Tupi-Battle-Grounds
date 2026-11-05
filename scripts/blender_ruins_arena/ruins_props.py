"""Model the carved sanctuary, ruins Strongholds and individual archaeological props."""

from math import cos, pi, sin, tau
from random import Random

from mesh import box, pebble, surface, tube
from structures import cloth
from terrain import rows


def ashlar_arch(group,name,center,span,rise,depth,*,broken=False):
    """Segment an arch into separate keyed voussoirs in the XZ plane."""
    x,y,z=center
    inner=span*.5
    outer=inner+.17
    for i in range(9):
        if broken and i in (6,7,8):
            continue
        a,b=(i+.045)*pi/9,(i+.955)*pi/9
        profile=[(x+r*cos(t),z+rise*r*sin(t)/inner) for r,t in ((inner,a),(outer,a),(outer,b),(inner,b))]
        vertices=[(u,y+dy,v) for dy in (-depth/2,depth/2) for u,v in profile]
        faces=[(0,1,2,3),(7,6,5,4),(0,4,5,1),(1,5,6,2),(2,6,7,3),(3,7,4,0)]
        key="ruin-jade" if i==4 else "limestone-light" if i%3==0 else "limestone"
        surface(group,name,vertices,faces,[key]*6)


def column(group,center,height,radius,*,broken=False):
    x,y,z=center
    box(group,"Square column stylobate",(x,y,z+.052),(radius*2.93,radius*2.93,.117),"limestone-dark",.022)
    tube(group,"Octagonal column foot",[(x,y,z+.105),(x,y,z+.182)],
         [radius*1.28,radius*1.05],"limestone-light",sides=8)
    shaft_top=z+height-(.135 if broken else .175)
    tube(group,"Tapered fluted shaft",[(x,y,z+.17),(x+.015,y,shaft_top)],
         [radius,radius*.82],"limestone",sides=8,accent="limestone-seam")
    for i in range(4):
        a=(i+.5)*tau/8
        u,v=cos(a)*radius,sin(a)*radius
        tube(group,"Raised worn flute",[(x+.92*u,y+.92*v,z+.197),(x+.77*u,y+.77*v,shaft_top-.032)],
             [.012,.009],"limestone-light",sides=5)
    if broken:
        pebble(group,"Sheared column crown",(x+.024,y,shaft_top+.007),
               (radius*.96,radius*.94,.113),"limestone-dark",sides=7,accent="limestone-light",phase=.38)
        return
    tube(group,"Worked octagonal capital",[(x+.014,y,shaft_top-.015),(x,y,shaft_top+.106)],
         [radius*.88,radius*1.37],"limestone-light",sides=8,accent="limestone-seam")
    box(group,"Square abacus above the capital",(x,y,z+height-.033),
        (radius*2.94,radius*2.94,.073),"limestone-dark",.015)


def stronghold(group):
    box(group,"Weathered Stronghold limestone foundation",(0,0,.046),(2.55,2.03,.166),"limestone-dark",.088)
    box(group,"Dressed secondary plinth",(0,0,.143),(2.39,1.86,.113),"limestone",.045)
    box(group,"Recessed Stronghold hall",(0,.109,.827),(1.78,1.37,1.42),"ruin-shadow",.049)
    for row in range(5):
        z=.24+row*.244
        for side in (-1,1):
            box(group,"Masonry beside the gate",(side*(.62+.014*(row%2)),-.647,z),
                (.40,.265,.235),"limestone" if row%3 else "limestone-seam",.023)
            for i in range(3):
                box(group,"Chipped side ashlar",(side*.866,-.37+i*.432,z),(.188,.409,.232),
                    "limestone" if (i+row)%3 else "limestone-dark",.020)
        if row>=3:
            box(group,"Lintel supporting masonry",(0,-.637,z),(.883,.249,.235),"limestone",.024)
    gate(group)
    box(group,"Plain stone parapet walk",(0,.096,1.557),(1.98,1.59,.165),"limestone-light",.041)
    for x in (-.56,0,.56):
        box(group,"Separate parapet stone",(x,-.647,1.695),(.34,.237,.251),"limestone",.033)
    tower(group,0,.372,.575,2.64)
    for x in (-.91,.91):
        tower(group,x,-.557,.350,1.58)
    pebble(group,"Moss clustered by entrance",(-.71,-.91,.124),(.246,.130,.075),
           "ruin-moss",sides=7,accent="ruin-leaf-dark")


def gate(group):
    for i in range(3):
        box(group,"Limestone entrance step",(0,-1.19+i*.113,.029+i*.029),
            (.771,.171,.082+.058*i),"limestone-light",.016)
    for s in (-1,1):
        for j in range(3):
            box(group,"Segmented carved gate jamb",(s*.363,-.828,.18+j*.227),
                (.184,.162,.223),"limestone-light" if j%2 else "limestone",.019)
    for x in (-.204,-.102,0,.102,.204):
        h=.70+.154*(1-(x/.285)**2)**.5
        box(group,"Dark gate planks",(x,-.786,h*.5),(.097,.087,h),"ruin-oak",.012)
    for z in (.245,.547):
        box(group,"Bronze horizontal gate band",(0,-.846,z),(.517,.038,.035),"ruin-bronze",.007)
    ashlar_arch(group,"Carved Stronghold gate stone",(0,-.833,.692),.554,.241,.168)


def tower(group,x,y,r,top):
    tube(group,"Tapered octagonal ancient tower",[(x,y,.15),(x,y,top*.53),(x,y,top-.24)],
         [r*1.14,r,r*.92],"limestone-dark",sides=8,accent="limestone")
    for z in (top*.37,top*.64,top-.235):
        tube(group,"Dressed horizontal tower course",[(x,y,z),(x,y,z+.065)],
             [r*1.03,r*.99],"limestone-light",sides=8)
    box(group,"Recessed tower aperture",(x,y-r*.927,top*.68),(r*.29,.052,r*.58),"ruin-shadow",.011)
    box(group,"Patinated window inset",(x,y-r*.969,top*.657),(r*.125,.024,r*.34),"ruin-jade-dark",.008)
    tube(group,"Terraced patinated tower crown",[(x,y,top-.211),(x,y,top-.095),
         (x,y,top+.016),(x,y,top+.19)],[r*1.09,r*1.01,r*.73,r*.45],
         "ruin-copper",sides=8,accent="ruin-jade")
    tube(group,"Limestone cap below the finial",[(x,y,top+.17),(x,y,top+.257)],
         [r*.50,r*.41],"limestone-light",sides=8)


def standard(group,key):
    tube(group,"Ancient flagstaff",[(0,.37,2.877),(0,.37,3.76)],[.024,.017],"ruin-bronze",sides=7)
    tube(group,"Patinated flagstaff finial",[(0,.37,3.74),(0,.37,3.84)],
         [.040,.013],"ruin-jade",sides=7)
    cloth(group,"Stronghold pennant",key,(.263,.37,3.712),.526,.295)
    for x in (-.92,.92):
        tube(group,"Side banner crossbar",[(x-.149,-.96,1.419),(x+.149,-.96,1.419)],
             [.018,.018],"ruin-bronze",sides=7)
        cloth(group,"Architectural side banner",key,(x,-.977,1.418),.26,.49)


def colonnade(group):
    box(group,"Fragment of a limestone stylobate",(0,0,.061),(1.72,.729,.152),"limestone-dark",.045)
    column(group,(-.64,0,.132),1.47,.154)
    column(group,(.62,0,.132),1.47,.154)
    box(group,"Discontinuous carved colonnade beam",(0,0,1.633),(1.61,.447,.153),"limestone",.028)
    box(group,"Blue-green inlaid beam molding",(0,-.238,1.645),(1.36,.035,.050),"ruin-jade",.009)


def gate_fragment(group):
    box(group,"Resting arch foundation",(0,0,.069),(1.45,.603,.157),"limestone-dark",.052)
    for s,height in ((-1,1.10),(1,.73)):
        box(group,"Fragmented arch pier",(s*.54,0,height*.49+.117),(.329,.406,height),
            "limestone",.036)
        box(group,"Worn edge molding",(s*.54,-.217,height*.53+.109),
            (.199,.052,height*.76),"limestone-light",.019)
    ashlar_arch(group,"Individually broken arch segment",(0,0,1.152),.86,.424,.43,broken=True)


def collapsed_column(group):
    for i in range(3):
        a=-.53+i*.05
        x=-.47+i*.37
        tube(group,"Tumbled fluted limestone drum",[(x,0,.18),(x+.294*cos(a),.294*sin(a),.221)],
             [.177,.169],"limestone",sides=8,accent="limestone-light")
    box(group,"Detached stone capital",(.59,-.12,.152),(.43,.38,.274),"limestone-dark",.052,angle=-.27)
    pebble(group,"Ground moss across fallen stone",(.47,.02,.29),(.166,.106,.046),
           "ruin-moss",accent="ruin-leaf")


def balustrade(group):
    box(group,"Short limestone balustrade base",(0,0,.062),(1.12,.24,.135),"limestone-dark",.020)
    for x in (-.46,0,.46):
        tube(group,"Individual turned stone baluster",[(x,0,.115),(x,0,.273),(x,0,.421)],
             [.068,.051,.072],"limestone",sides=6,accent="limestone-light")
    box(group,"Weathered jade coping strip",(0,0,.456),(1.11,.183,.078),"ruin-jade-dark",.016)


def ritual_lamp(group):
    box(group,"Limestone lamp plinth",(0,0,.078),(.431,.423,.159),"limestone-dark",.036)
    tube(group,"Octagonal ritual lamp pedestal",[(0,0,.143),(0,0,.356)],
         [.134,.091],"limestone",sides=8,accent="limestone-light")
    tube(group,"Flared patinated lamp bowl",[(0,0,.321),(0,0,.406),(0,0,.456)],
         [.103,.195,.174],"ruin-copper",sides=8,accent="ruin-jade")
    pebble(group,"Lit mineral in the ritual bowl",(0,0,.453),(.151,.147,.029),
           "ritual-ember",accent="ritual-glow")
    for x,y,h in ((-.052,.028,.173),(.057,.034,.126),(.012,-.041,.209)):
        tube(group,"Separate sculpted ritual flame",[(x,y,.461),(x+.028,y,.461+h*.57),
             (x+.013,y+.016,.450+h)],[.043,.028,.004],"ritual-glow",sides=5,accent="ruin-jade-light")


def shrine_islet(group):
    rings=[]
    for sx,sy,z in ((3.56,1.96,-.43),(3.59,1.97,-.12),(3.38,1.78,.18),(3.11,1.52,.34)):
        rings.append([(sx*cos(i*tau/22)*(1+.08*sin(i*2.01)),
                       sy*sin(i*tau/22)*(1+.09*cos(i*1.39)),z+.05*sin(i*1.70))
                      for i in range(22)])
    keys=("ruin-sand","limestone-dark","limestone")
    rows(group,"Eroded rock beneath the sanctuary",rings,
         lambda band,i:keys[band] if i%5 else "limestone-seam",floor=True)
    for x,y in ((-2.43,-.69),(2.52,-.58),(-1.98,.74),(2.24,.74)):
        pebble(group,"Moss spread over the islet",(x,y,.345),(.45,.30,.071),
               "ruin-moss",sides=8,accent="ruin-leaf-dark")


def sanctuary(group):
    box(group,"Sanctuary lower limestone podium",(0,0,.142),(5.18,2.17,.284),"limestone-dark",.064)
    box(group,"Dressed upper sanctuary podium",(0,0,.323),(4.89,1.94,.164),"limestone-light",.048)
    for i in range(4):
        box(group,"Wide stair to the sanctuary",(0,-1.52+i*.189,.047+i*.043),
            (2.10,.285,.094+i*.086),"limestone" if i%2 else "limestone-seam",.027)
    for x in (-2.0,-.67,.67,2.0):
        column(group,(x,-.51,.401),1.19,.155)
        column(group,(x,.68,.401),1.19,.129,broken=abs(x)<1)
    for x in (-1.335,0,1.335):
        ashlar_arch(group,"Sanctuary arch stone",(x,-.50,1.56),1.009,.495,.31)
    box(group,"Continuous lintel across the sanctuary",(0,-.50,2.260),(4.83,.538,.183),
        "limestone-light",.036)
    box(group,"Jade inlay across the lintel",(0,-.790,2.275),(4.42,.055,.057),"ruin-jade",.011)
    section=[(-2.38,2.340),(-.60,2.862),(.60,2.862),(2.38,2.340)]
    verts=[(x,y,z) for y in (-.77,-.18) for x,z in section]
    surface(group,"Broken trapezoidal sanctuary pediment",verts,
            [(3,2,1,0),(4,5,6,7),(0,1,5,4),(1,2,6,5),(2,3,7,6),(0,4,7,3)],
            ["limestone","limestone-seam","limestone-light","limestone-dark",
             "limestone-light","limestone-dark"])
    for s in (-1,1):
        tube(group,"Raised stone pediment border",[(s*.59,-.804,2.839),(s*2.31,-.804,2.351)],
             [.038,.040],"limestone-light",sides=5)
    tube(group,"Diamond jade crest at the head of the sanctuary",[(0,-.838,2.456),
         (0,-.881,2.627),(0,-.854,2.790)],[.046,.239,.035],"ruin-jade",sides=4,accent="ruin-jade-dark")
    pebble(group,"Copper heart set in the jade crest",(0,-1.051,2.627),(.087,.035,.087),
           "ruin-bronze",sides=6)
    for x in (-1.93,1.93):
        box(group,"Small weathered votive plinth",(x,-.92,.449),(.31,.35,.134),
            "ruin-bronze",.023)


def submerged_drum(group):
    tube(group,"Flooded section of a fluted column",[(-.40,0,-.19),(-.28,0,-.10),(.34,.074,.031)],
         [.231,.254,.197],"limestone-dark",sides=8,accent="limestone-seam")
    tube(group,"Pale broken end of a flooded drum",[(.292,.069,.022),(.433,.077,.052)],
         [.201,.178],"limestone",sides=8,accent="ruin-moss")


def flooded_steps(group):
    for i in range(5):
        box(group,"Individual step disappearing into the lagoon",(0,i*.34-.74,i*.116-.13),
            (1.66+.11*(i%2),.435,.141+i*.027),"limestone-dark" if i<2 else "limestone",.037)
    for s in (-1,1):
        pebble(group,"Broken stair pier",(s*.79,.60,.205),(.265,.237,.415),
               "limestone-seam",sides=7,accent="limestone")


def lily(group):
    for i,(x,y,r) in enumerate(((-.24,.05,.27),(.17,-.17,.22),(.32,.23,.19),(-.49,-.14,.14))):
        points=[(x,y,.020)]+[(x+r*cos(i+theta),y+r*.84*sin(i+theta),.031)
                             for theta in (j*5.65/9 for j in range(10))]
        surface(group,"Notched lagoon lily pad",points,[(0,j+1,j+2) for j in range(9)],
                ["ruin-leaf","ruin-leaf","ruin-moss"]*3)
    for i in range(6):
        a=i*tau/6
        tube(group,"Pale lotus petal",[(.32,.23,.034),(.32+.065*cos(a),.23+.065*sin(a),.078)],
             [.035,.006],"ruin-petal",sides=5)
    pebble(group,"Lotus center",(.32,.23,.061),(.04,.04,.031),"ruin-flower",sides=6)


def fern(group):
    for i in range(8):
        a=i*tau/8
        length=.23+.05*sin(i*1.74)
        dx,dy=cos(a),sin(a)
        tip=(length*dx,length*dy,.12+.036*cos(i))
        tube(group,"Rib of a low ground fern",[(0,0,0),(.11*dx,.11*dy,.123),tip],
             [.016,.012,.003],"ruin-fern",sides=5)
        for j in range(1,4):
            t=j*.23
            x,y,z=tip[0]*t,tip[1]*t,tip[2]*t+.04*sin(pi*t)
            for s in (-1,1):
                surface(group,"Pointed fern leaflet",[(x,y,z),(x+.043*dx+s*.041*dy,
                        y+.043*dy-s*.041*dx,z+.015),(x+.060*dx,y+.060*dy,z+.018)],
                        [(0,1,2)],["ruin-leaf" if j%2 else "ruin-leaf-dark"])


def fir(group):
    tube(group,"Bent old cedar trunk",[(0,0,0),(.052,0,.69),(.105,.028,1.48)],
         [.115,.086,.034],"ruin-oak",sides=6)
    for i in range(4):
        x,y,z=.04+i*.035,.017*i,.44+i*.31
        r=.67-i*.114
        tube(group,"Open layered cedar crown",[(x,y,z),(x,y,z+.15),(x+.056,y,z+.58)],
             [r*.36,r,.034],"ruin-fir-dark",sides=7,accent="ruin-fir")


def reeds(group):
    rng=Random(204)
    for i in range(11):
        x,y=rng.uniform(-.26,.26),rng.uniform(-.15,.15)
        h=rng.uniform(.22,.53)
        dx,dy=rng.uniform(-.14,.14),rng.uniform(-.09,.09)
        tube(group,"Leaning lagoon reed",[(x,y,0),(x+.36*dx,y+.36*dy,h*.54),(x+dx,y+dy,h)],
             [.014,.010,.004],"ruin-fern",sides=5)
        if i%3==0:
            tube(group,"Ochre reed seed head",[(x+.86*dx,y+.86*dy,h-.057),(x+dx,y+dy,h+.025)],
                 [.020,.011],"ruin-reed",sides=5)
