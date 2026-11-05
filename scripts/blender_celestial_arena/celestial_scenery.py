"""Create the gardens, carved terraces and offshore waterfall assemblies."""

from math import cos, sin, tau
from random import Random

from celestial_architecture import gem
from mesh import box, pebble, surface, tube
from terrain import rows


def rocky_islet(group, height, rx, ry):
    rings = []
    profiles = ((.96,-.38),(1.04,-.08),(1,.34*height),(.91,.68*height),(.81,height))
    for band,(scale,z) in enumerate(profiles):
        rings.append([(rx*scale*cos(a)*(1+.077*sin(i*2.03)),
                       ry*scale*sin(a)*(1+.083*cos(i*1.46)),
                       z+.047*sin(i*1.69+band*.52))
                      for i in range(24) for a in (i*tau/24,)])
    keys = ("cel-cliff","cel-ivory-shade","cel-stone","cel-ivory")
    rows(group,"Terraced ivory island rock",rings,
         lambda band,i:keys[band] if (i+band)%6 else "cel-cliff",floor=True)


def citadel_islet(group):
    rocky_islet(group,.59,3.82,2.27)


def waterfall_islet(group):
    rocky_islet(group,1.34,1.34,1.13)
    pebble(group,"Weathered shelves beside the waterfall",(1.01,-.15,.493),(.40,.42,.50),
           "cel-stone",sides=6,accent="cel-cliff",phase=.73)


def waterfall(group):
    spring_pool(group)
    rings = ((-.61,1.39,.38),(-.87,1.37,.37),(-1.03,1.30,.35),(-1.14,.84,.31),
             (-1.24,.34,.28),(-1.39,.06,.40),(-1.60,.034,.56))
    vertices = [(width*t,y+.017*sin(i*.9+j*1.58),z+.020*cos(i*.63+j*1.16))
                for i,(y,z,width) in enumerate(rings) for j,t in enumerate((-1,-.58,-.11,.44,1))]
    faces = [(i*5+j,(i+1)*5+j,(i+1)*5+j+1,i*5+j+1) for i in range(6) for j in range(4)]
    keys = ("cel-fall-shadow","cel-waterfall","cel-foam","cel-waterfall")
    surface(group,"Separate faceted waterfall sheet",vertices,faces,list(keys)*6)
    for x,y,w in ((-.49,-1.50,.29),(.12,-1.62,.43),(.40,-1.53,.34)):
        pebble(group,"Low white splash where the cascade hits the sea",(x,y,.055),(.8*w,.24,.09),
               "cel-foam",sides=7,accent="cel-waterfall",phase=x)
    for s in (-1,1):
        surface(group,"Splayed cascade ripple",[(s*.23,-1.61,.043),(s*.71,-1.81,.046),
                (s*.93,-1.83,.042),(s*.70,-1.86,.046),(s*.20,-1.65,.043)],
                [(0,1,2,3,4)],["cel-ripple"])


def spring_pool(group):
    outline = [(-.37,-.71),(.37,-.71),(.52,-.37),(.62,-.06),(.46,.35),
               (.19,.48),(-.26,.44),(-.55,.18),(-.63,-.16),(-.48,-.44)]
    vertices = [(0,-.09,1.408)]+[(x,y,1.405) for x,y in outline]
    faces = [(0,i+1,(i+1)%len(outline)+1) for i in range(len(outline))]
    keys = ["cel-waterfall" if i in (0,1,4,7,9) else "cel-fall-shadow" for i in range(len(faces))]
    surface(group,"Blue spring pool feeding the cascade",vertices,faces,keys)
    for i,(x,y) in enumerate(outline[1:],1):
        a,b = outline[(i+1)%len(outline)]
        tube(group,"Shallow pale rock lip around the spring",[(x,y,1.413),((a+x)*.5,(b+y)*.5,1.436),
             (a,b,1.413)],[.044,.057,.044],"cel-stone",sides=6,accent="cel-ivory")
    surface(group,"Quiet highlight at the spring mouth",[(-.20,-.57,1.417),(.13,-.55,1.417),
             (.24,-.51,1.417),(-.15,-.53,1.417)],[(0,1,2,3)],["cel-foam"])


def crystal_garden(group):
    box(group,"Shallow cut ivory shrine plinth",(0,0,.068),(.86,.69,.145),"cel-ivory-shade",.034)
    tube(group,"Octagonal gold-lined crystal socket",[(0,0,.130),(0,0,.211),(0,0,.287)],
         [.279,.261,.300],"cel-gold",sides=8,accent="cel-gold-light")
    for i,(x,y,z,r,h) in enumerate(((0,0,.226,.131,.62),(-.17,-.076,.235,.073,.324),
                                     (.134,.097,.217,.081,.387),(.095,-.145,.215,.060,.269))):
        gem(group,"Individual azure sanctuary crystal",(x,y,z),r,h)
        if i:
            pebble(group,"Pale stone around the crystal socket",(x,y,.209),(.105,.099,.051),
                   "cel-ivory",sides=6)


def cypress(group):
    tube(group,"Exposed ornamental cypress trunk",[(0,0,-.02),(.028,.012,.47),(.079,.027,1.39)],
         [.091,.069,.027],"cel-oak",sides=7)
    for i in range(3):
        z = .25+i*.43
        x = .026+i*.047
        r = .354-i*.063
        tube(group,"Sculpted slender cypress crown",[(x,0,z),(x,.014,z+.16),(x+.060,.027,z+.63)],
             [r*.45,r,.023],"cel-leaf-dark",sides=8,accent="cel-leaf")


def flowering_shrub(group):
    rng = Random(106)
    for i,(x,y,z,r) in enumerate(((-.21,.05,.16,.19),(.19,.06,.18,.24),(0,-.09,.19,.25),(0,.11,.25,.21))):
        pebble(group,"Rounded layered garden shrub",(x,y,z),(r,r,.22),
               "cel-leaf" if i%2 else "cel-leaf-dark",sides=7,accent="cel-leaf-light",phase=i*.69)
    for i in range(7):
        a = i*tau/7
        r = rng.uniform(.13,.31)
        pebble(group,"Small stone-pink garden blossom",(r*cos(a),.08+r*sin(a)*.68,rng.uniform(.26,.36)),
               (.069,.057,.054),"cel-bloom",sides=5,accent="cel-bloom-light",phase=a)


def planter(group):
    box(group,"Chamfered garden planter base",(0,0,.064),(.73,.64,.139),"cel-ivory-shade",.036)
    box(group,"Ivory flared planter rim",(0,0,.177),(.80,.70,.114),"cel-ivory",.023)
    box(group,"Exposed soil inside the planter",(0,0,.233),(.673,.573,.017),"cel-soil",.021)
    for x,y in ((-.22,.087),(0,.01),(.21,.046)):
        pebble(group,"Individual clipped planter foliage",(x,y,.35),(.20,.20,.25),
               "cel-leaf",sides=7,accent="cel-leaf-light",phase=x)
    for x in (-.285,0,.285):
        box(group,"Small gilt planter inset",(x,-.354,.174),(.072,.017,.063),"cel-gold",.009)


def lantern(group):
    box(group,"Ivory lantern foot",(0,0,.065),(.355,.351,.132),"cel-stone",.021)
    tube(group,"Blue lantern column",[(0,0,.122),(0,0,.373),(0,0,.425)],
         [.080,.062,.108],"cel-azure",sides=7,accent="cel-azure-light")
    tube(group,"Lit glass inside the lantern",[(0,0,.414),(0,0,.587)],
         [.083,.074],"cel-flame",sides=8)
    for x,y in ((-.086,0),(.086,0),(0,-.086),(0,.086)):
        tube(group,"Separate gilt lantern frame",[(x,y,.401),(x*.91,y*.91,.598)],
             [.016,.014],"cel-gold",sides=6)
    tube(group,"Small azure lantern cover",[(0,0,.571),(0,0,.618),(0,0,.713)],
         [.128,.126,.022],"cel-azure",sides=8,accent="cel-azure-light")


def balustrade(group):
    box(group,"Carved garden balustrade base",(0,0,.055),(1.11,.263,.125),"cel-ivory-shade",.020)
    for x in (-.459,0,.459):
        tube(group,"Turned ivory garden baluster",[(x,0,.111),(x,0,.239),(x,0,.396)],
             [.066,.039,.069],"cel-ivory",sides=7,accent="cel-stone")
    box(group,"Shaped blue rail over the balusters",(0,0,.418),(1.115,.193,.073),"cel-azure",.014)
    for x in (-.457,.457):
        box(group,"Short gilt cap at the end of the rail",(x,0,.453),(.181,.199,.036),"cel-gold",.010)


def ripple(group):
    for i,(x,y,length) in enumerate(((-.48,-.04,.81),(.14,.14,.72),(-.35,.31,.96))):
        points = [(x,y,.012),(x+length*.40,y+.031,.016),(x+length,y+.062,.012),
                  (x+length*.78,y+.083,.016),(x+length*.33,y+.057,.020)]
        surface(group,"Long thin highlight on blue water",points,[(0,1,4),(1,2,3,4)],
                ["cel-ripple" if i%2==0 else "cel-shelf-light","cel-shelf-light"])


def beacon(group):
    box(group,"Stepped garden beacon footing",(0,0,.063),(.626,.565,.145),"cel-stone",.032)
    box(group,"Ivory secondary beacon step",(0,0,.164),(.537,.48,.096),"cel-ivory",.025)
    tube(group,"Faceted beacon column",[(0,0,.204),(0,0,.478),(0,0,.602)],
         [.123,.089,.169],"cel-stone",sides=8,accent="cel-ivory")
    tube(group,"Gilt rim around the beacon stone",[(0,0,.567),(0,0,.639)],
         [.174,.167],"cel-gold",sides=8,accent="cel-gold-light")
    gem(group,"Upright beacon crystal",(0,0,.625),.112,.335)
