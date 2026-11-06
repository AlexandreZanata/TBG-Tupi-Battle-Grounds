"""Author the winter Stronghold, glacier, shelter and shoreline prop collections."""

from math import cos, sin, tau
from random import Random

from mesh import box, pebble, surface, tube
from structures import arch, heraldry


def stronghold(group):
    box(group,"Frost-cut plinth",(0,0,.043),(2.55,2.03,.170),"frost-basalt",.095)
    box(group,"Chipped foundation course",(0,0,.145),(2.39,1.87,.116),"frost-ashlar",.054)
    box(group,"Inset granite keep",(0,.118,.836),(1.78,1.39,1.40),"frost-basalt",.068)
    for row in range(5):
        z=.25+row*.247
        for side in (-1,1):
            box(group,"Dressed granite facade block",(side*(.622+.018*(row%2)),-.636,z),
                (.41,.284,.235),"frost-granite" if row%2 else "frost-ashlar",.028)
            for i in range(3):
                box(group,"Bonded frosted side course",(side*.867,-.39+i*.43,z),
                    (.223,.396,.234),"frost-ashlar" if (i+row)%3==0 else "frost-granite",.023)
        if row>=3:
            box(group,"Frosted lintel bed",(0,-.641,z),(.88,.241,.237),"frost-granite",.023)
    arch(group,-.822)
    for i in range(3):
        box(group,"Steps with worn icy edges",(0,-1.188+i*.12,.026+i*.029),
            (.78,.176,.079+i*.059),"road-edge",.017)
    box(group,"Stone parapet walk",(0,.079,1.515),(1.96,1.62,.154),"frost-ashlar",.045)
    for x in (-.57,0,.57):
        box(group,"Frosted battlement",(x,-.68,1.663),(.37,.245,.234),"frost-granite",.034)
        box(group,"Independent parapet snow cap",(x,-.68,1.788),(.38,.26,.075),"snow-light",.030)
    tower(group,0,.37,.59,2.69)
    for x in (-.92,.92):
        tower(group,x,-.54,.36,1.49)


def tower(group,x,y,r,top):
    tube(group,"Octagonal granite tower",[(x,y,.15),(x,y,top*.55),(x,y,top-.19)],
         [r*1.12,r,r*.93],"frost-granite",sides=8,accent="frost-ashlar")
    for z in (top*.40,top*.65,top-.20):
        tube(group,"Raised ashlar binding course",[(x,y,z),(x,y,z+.053)],
             [r*1.02,r],"frost-ashlar",sides=8)
    box(group,"Shadowed narrow window",(x,y-r*.91,top*.66),(r*.23,.069,r*.60),"soot",.012)
    tube(group,"Copper snow roof eave",[(x,y,top-.20),(x,y,top-.08),(x,y,top+.41)],
         [r*1.16,r*1.08,.037],"winter-copper",sides=8)
    tube(group,"Ragged seated roof snow",[(x,y,top+.015),(x,y,top+.15),(x,y,top+.428)],
         [r*.98,r*.62,.018],"snow-light",sides=8,accent="snow-shadow")
    tube(group,"Forged roof finial",[(x,y,top+.40),(x,y,top+.55)],[.038,.008],"iron",sides=7)


def standard(group,seat):
    heraldry(group,seat)


def fir(group,young=False):
    stature=.82 if young else 1.0
    tube(group,"Cold-weather fir trunk",[(0,0,0),(.032,.018,1.95*stature)],
         [.087,.031],"winter-timber",sides=7)
    for i,(z,r,height) in enumerate(((.41,.57,.70),(.83,.47,.67),(1.18,.37,.68),(1.53,.263,.62))):
        x=.047*i
        z*=stature
        tube(group,"Separate layered spruce bough",[(x,0,z),(x-.022,.018,z+.13),
             (x+.041,.038,z+height*stature)],[r*.86,r,.013],"frost-fir-dark" if i==0 else "frost-fir",
             sides=9,accent="frost-fir-light")
        tube(group,"Independent snow cover on spruce bough",[(x,.018,z+.21),(x+.013,.022,z+.36),
             (x+.041,.038,z+height*stature+.033)],[r*.95,r*.71,.009],
             "snow" if i%2 else "snow-shadow",sides=9,accent="snow-light")


def boulder(group):
    for i,(x,y,rx,ry,rz) in enumerate(((-.22,0,.32,.27,.44),(.23,.07,.30,.31,.32),(.20,-.20,.18,.15,.20))):
        pebble(group,"Frost-weathered granite",(x,y,rz*.49),(rx,ry,rz),
               "frost-basalt",sides=8,accent="frost-granite",phase=i*.9)
        pebble(group,"Independent snow cap",(x+.015,y-.015,rz*1.01),(rx*.79,ry*.81,rz*.17),
               "snow",sides=8,accent="snow-light",phase=i*.9)


def frost_crystals(group):
    pebble(group,"Stone bed under the crystal vein",(0,0,.067),(.29,.235,.143),
           "frost-granite",accent="snow-shadow")
    for x,y,h,r in ((-.16,-.036,.31,.064),(-.04,.042,.57,.082),(.12,-.045,.39,.068),(.175,.065,.26,.053)):
        tube(group,"Faceted frost crystal",[(x,y,.07),(x,y,h*.74),(x+.038,y-.027,h)],
             [r*.85,r,.004],"winter-crystal-dark",sides=5,accent="winter-crystal")


def drift(group):
    pebble(group,"Wind-packed snowdrift",(0,0,.066),(.63,.31,.136),
           "snow",sides=9,accent="snow-light")


def reeds(group):
    rng=Random(38)
    for i in range(7):
        x,y=rng.uniform(-.19,.17),rng.uniform(-.16,.13)
        height=rng.uniform(.13,.27)
        tube(group,"Dry wind-bent frost reed",[(x,y,0),(x+.045,y-.014,height*.7),(x+.080,y-.035,height)],
             [.012,.010,.002],"winter-reed" if i%3 else "winter-moss",sides=5)


def cairn(group):
    for i,r in enumerate((.213,.174,.126)):
        pebble(group,"Stacked waystone",(i*.018,0,.062+i*.123),(r,r*.86,.114),
               "frost-granite",sides=8,accent="frost-ashlar",phase=i*.5)
    pebble(group,"Snow hood on trail cairn",(.034,0,.35),(.117,.106,.041),"snow-light",sides=7)


def rail(group):
    for x in (-.52,.52):
        box(group,"Short split oak upright",(x,0,.233),(.139,.148,.465),"winter-timber",.022)
        box(group,"Snow gathered on rail post",(x,0,.475),(.151,.160,.040),"snow-light",.014)
    for z in (.178,.315):
        box(group,"Weathered timber rail",(0,0,z),(1.07,.089,.074),"winter-wood",.017)
    box(group,"Snow strip along timber rail",(0,-.004,.362),(.98,.070,.027),"snow-shadow",.011)


def beacon(group):
    box(group,"Stone lantern footing",(0,0,.075),(.38,.39,.15),"frost-basalt",.043)
    box(group,"Carved beacon post",(0,0,.267),(.152,.154,.312),"winter-timber",.026)
    box(group,"Lantern tray",(0,0,.442),(.241,.243,.065),"winter-copper",.016)
    tube(group,"Honey-colored lamp glass",[(0,0,.470),(0,0,.682)],[.082,.069],"glow",sides=6)
    for x in (-.097,.097):
        for y in (-.097,.097):
            tube(group,"Lamp frame upright",[(x,y,.474),(x,y,.685)],[.012,.013],"iron",sides=6)
    tube(group,"Peaked copper cap",[(0,0,.675),(0,0,.715),(0,0,.802)],
         [.158,.141,.024],"winter-copper",sides=8)
    tube(group,"Thin snow on the lamp roof",[(0,0,.739),(0,0,.815)],
         [.104,.019],"snow",sides=8)


def icefall(group):
    for x,y,length,width in ((-.36,0,1.04,.10),(-.18,.035,1.29,.13),(0,.01,1.49,.16),
                             (.16,.046,1.10,.11),(.31,.012,.83,.08)):
        tube(group,"Frozen waterfall ribbon",[(x,y,.045),(x+.035,y-.19,-length*.40),
             (x+.065,y-.30,-length)],[width,width*.72,.012],"glacier",sides=5,accent="glacier-light")


def floe(group):
    pebble(group,"Uneven sea-ice fragment",(0,0,-.024),(.67,.44,.161),
           "ice-deep",sides=7,accent="ice",phase=.19)
    pebble(group,"Rime on the drift ice",(-.05,.043,.066),(.48,.32,.062),
           "ice-white",sides=7,accent="snow-light",phase=.2)


def ridge(group):
    for i,(x,y,rx,ry,h) in enumerate(((-1.26,.19,1.30,.66,1.08),(-.24,.34,1.29,.77,1.85),
                                     (.68,.05,1.33,.81,1.46),(1.53,.27,.91,.62,1.07))):
        angles=[tau*j/6+.12*i for j in range(6)]
        outline=[(x+rx*cos(a),y+ry*sin(a),-.12) for a in angles]
        foot=[(x+.45*(u-x)-.09,y+.43*(v-y),h*.58+.11*sin(j*4)) for j,(u,v,_) in enumerate(outline)]
        apex=(x-.29,y+.16,h)
        faces=[(j,(j+1)%6,6+(j+1)%6,6+j) for j in range(6)]
        faces.extend((6+j,6+(j+1)%6,12) for j in range(6))
        surface(group,"Glacial ridge with an uneven snowline",outline+foot+[apex],faces,
                ["ridge-dark","ridge-dark","ridge-light"]*2+["ridge-snow","snow-shadow"]*3)
