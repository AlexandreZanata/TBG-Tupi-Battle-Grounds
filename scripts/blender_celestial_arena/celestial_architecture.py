"""Model ivory Strongholds, the stepped citadel and its ceremonial guardians."""

from math import cos, pi, sin

from mesh import box, pebble, surface, tube
from structures import cloth


def gem(group,name,at,width,height):
    x,y,z = at
    tube(group,name,[(x,y,z),(x,y,z+height*.24),(x,y,z+height*.71),(x,y,z+height)],
         [width*.51,width,width*.81,.004],"cel-crystal",sides=6,accent="cel-crystal-light")


def column(group,at,height,radius):
    x,y,z = at
    box(group,"Square carved column foot",(x,y,z+.052),(radius*3,radius*3,.116),"cel-stone",.024)
    tube(group,"Octagonal ivory column shaft",[(x,y,z+.10),(x,y,z+height-.15)],
         [radius,radius*.83],"cel-ivory",sides=8,accent="cel-stone")
    tube(group,"Flared gilt column capital",[(x,y,z+height-.18),(x,y,z+height-.072),
         (x,y,z+height-.037)],[radius*.91,radius*1.36,radius*1.36],
         "cel-gold",sides=8,accent="cel-gold-light")
    box(group,"Ivory abacus above the gold capital",(x,y,z+height-.024),
        (radius*2.96,radius*2.96,.061),"cel-ivory",.012)


def arch(group,at,radius,rise,depth):
    x,y,z = at
    outer = radius+.145
    for i in range(11):
        a,b = (i+.042)*pi/11,(i+.958)*pi/11
        profile = [(x+r*cos(t),z+rise*r*sin(t)/radius)
                   for r,t in ((radius,a),(outer,a),(outer,b),(radius,b))]
        verts = [(u,y+v,w) for v in (-depth*.5,depth*.5) for u,w in profile]
        faces = [(0,1,2,3),(7,6,5,4),(0,4,5,1),(1,5,6,2),(2,6,7,3),(3,7,4,0)]
        key = "cel-gold-light" if i==5 else "cel-ivory" if i%2 else "cel-stone"
        surface(group,"Individually cut ivory arch stone",verts,faces,[key]*6)


def gate(group,at,width,height):
    x,y,z = at
    radius = width*.37
    spring = height*.62
    for s in (-1,1):
        column(group,(x+s*(radius+.075),y,z),spring,.066+width*.017)
    arch(group,(x,y,z+spring),radius,height*.27,.17)
    for i in range(6):
        u = (i-2.5)*radius*.31
        h = spring+height*.245*(1-(u/radius)**2)**.5
        box(group,"Recessed blue gate leaf",(x+u,y+.065,z+h*.5),
            (radius*.296,.081,h),"cel-azure-deep" if i%2 else "cel-azure",.009)
    for h in (.30,.55):
        box(group,"Gilt gate crossbar",(x,y+.013,z+height*h),
            (radius*1.78,.047,.045),"cel-gold",.008)
    tube(group,"Gilt split seam on the gate",[(x,y-.012,z+.12),(x,y-.012,z+height*.78)],
         [.017,.017],"cel-gold-light",sides=6)


def roof(group,at,radius,height):
    x,y,z = at
    tube(group,"Gilt eave ring beneath the blue roof",[(x,y,z),(x,y,z+.083)],
         [radius,radius*.96],"cel-gold",sides=8,accent="cel-gold-light")
    tube(group,"Layered faceted azure roof",[(x,y,z+.074),(x,y,z+height*.35),
         (x,y,z+height*.69),(x,y,z+height)],
         [radius*.94,radius*.78,radius*.48,radius*.09],
         "cel-azure",sides=8,accent="cel-azure-light")
    tube(group,"Gold cap above the azure tiles",[(x,y,z+height-.055),(x,y,z+height+.112)],
         [radius*.125,radius*.063],"cel-gold-light",sides=8)


def stronghold(group):
    box(group,"Ivory Stronghold lower plinth",(0,0,.072),(2.56,2.08,.174),"cel-ivory-shade",.062)
    box(group,"Bright coping above the lower plinth",(0,0,.188),(2.36,1.89,.104),"cel-ivory",.042)
    box(group,"Recessed central keep",(0,.12,.747),(1.59,1.27,1.11),"cel-stone",.066)
    for s in (-1,1):
        stronghold_wing(group,s)
    gate(group,(0,-.774,.218),.86,1.05)
    for i in range(3):
        box(group,"Separate ivory entry step",(0,-1.177+i*.109,.039+i*.038),
            (.868,.165,.084+i*.074),"cel-ivory",.020)
    box(group,"Upper lookout terrace",(0,.09,1.263),(1.91,1.53,.142),"cel-ivory",.045)
    tower(group,(0,.239,1.29),.52,1.02,.62)
    gem(group,"Azuri crown crystal",(0,.239,3.00),.133,.332)


def stronghold_wing(group,side):
    for i in range(4):
        box(group,"Individual pale ashlar on a Stronghold wing",(side*.70,-.631,.35+i*.246),
            (.57,.32,.239),"cel-ivory" if i%3 else "cel-stone",.023)
    tower(group,(side*.959,-.49,.20),.300,1.29,.42)
    for z in (.411,.826):
        box(group,"Blue recessed corner slit",(side*.959,-.792,z),(.079,.035,.156),"cel-azure-deep",.011)


def tower(group,at,radius,height,roof_height):
    x,y,z = at
    tube(group,"Pale octagonal tower body",[(x,y,z),(x,y,z+height*.24),(x,y,z+height)],
         [radius*1.05,radius,radius*.90],"cel-ivory",sides=8,accent="cel-stone")
    tube(group,"Coping around the upper tower",[(x,y,z+height-.109),(x,y,z+height-.027)],
         [radius*.98,radius*.96],"cel-gold",sides=8)
    box(group,"Deep azure arched tower niche",(x,y-radius*.918,z+height*.53),
        (radius*.48,.038,height*.372),"cel-azure-deep",.028)
    box(group,"Light set behind the tower niche",(x,y-radius*.961,z+height*.51),
        (radius*.19,.019,height*.223),"cel-lit",.007)
    roof(group,(x,y,z+height),radius*1.09,roof_height)


def standard(group,key):
    tube(group,"Gilt keep banner mast",[(0,.24,3.315),(0,.24,3.81)],[.022,.017],"cel-gold",sides=7)
    cloth(group,"Long keep pennant",key,(.266,.24,3.745),.53,.243)
    for x in (-.85,.85):
        tube(group,"Gilt corner flag bar",[(x-.137,-.821,1.256),(x+.137,-.821,1.256)],
             [.016,.016],"cel-gold-light",sides=6)
        cloth(group,"Keep wall pennant",key,(x,-.842,1.256),.236,.466)


def palace(group):
    box(group,"Citadel lower stylobate",(0,.10,.120),(4.87,2.14,.247),"cel-stone",.057)
    box(group,"Citadel upper stylobate",(0,.10,.281),(4.63,1.93,.153),"cel-ivory",.040)
    box(group,"Recessed blue palace hall",(0,.261,1.025),(3.35,1.50,1.43),"cel-azure-deep",.071)
    for side in (-1,1):
        palace_wing(group,side)
    gate(group,(0,-.585,.34),1.23,1.56)
    box(group,"Long gilt cornice across the citadel",(0,.165,1.739),(3.47,1.73,.171),"cel-gold",.043)
    box(group,"Ivory roof terrace",(0,.169,1.866),(3.56,1.79,.142),"cel-ivory",.037)
    crest(group,(0,-.744,1.82))
    tower(group,(0,.26,1.914),.76,1.04,.88)
    gem(group,"Crown of the celestial citadel",(0,.26,3.931),.182,.452)
    for i in range(6):
        box(group,"Individual broad citadel entry step",(0,-1.629+i*.133,.025+i*.026),
            (1.53,.238,.058+i*.054),"cel-ivory" if i%2 else "cel-stone",.022)


def crest(group,at):
    x,y,z = at
    points = [(x,y-.029,z)]+[(x+r*cos(a),y,z+r*sin(a))
                             for i in range(8) for a,r in ((i*pi/4,.181 if i%2==0 else .071),)]
    surface(group,"Four-point gold crest above the citadel gate",points,
            [(0,i+1,(i+1)%8+1) for i in range(8)],["cel-gold-light","cel-gold"]*4)
    pebble(group,"Blue gem inset in the citadel crest",(x,y-.040,z),(.046,.021,.049),
           "cel-crystal",sides=6,accent="cel-crystal-light")


def palace_wing(group,side):
    x = side*1.70
    tower(group,(x,.341,.321),.494,1.70,.68)
    column(group,(side*1.11,-.668,.34),1.45,.123)
    column(group,(side*1.58,-.651,.34),1.45,.116)
    arch(group,(side*1.34,-.668,1.414),.245,.252,.17)
    for i in range(3):
        box(group,"Individual golden step on the upper palace wall",(side*(.72+i*.27),-.776,1.653),
            (.216,.21,.104),"cel-stone" if i%2 else "cel-gold",.014)


def celestial_arch(group):
    box(group,"Low foundation for the garden arch",(0,0,.055),(1.44,.558,.126),"cel-stone",.030)
    for s in (-1,1):
        column(group,(s*.545,0,.107),1.03,.105)
    arch(group,(0,0,1.094),.414,.405,.248)
    gem(group,"Cut crystal above the garden keystone",(0,0,1.58),.092,.231)


def guardian(group):
    box(group,"Square guardian plinth",(0,0,.077),(.65,.58,.165),"cel-stone",.034)
    box(group,"Gilt line under the guardian statue",(0,0,.164),(.54,.48,.051),"cel-gold",.015)
    for s in (-1,1):
        box(group,"Boot carved into the base",(s*.094,-.042,.222),(.153,.235,.128),"cel-ivory",.023)
        tube(group,"Armored stone leg",[(s*.093,0,.274),(s*.087,0,.52),(s*.066,.016,.674)],
             [.075,.092,.079],"cel-stone",sides=7,accent="cel-ivory")
        pebble(group,"Separate ivory pauldron",(s*.210,.017,1.027),(.130,.130,.114),
               "cel-ivory",sides=7,accent="cel-gold-light")
    tube(group,"Carved cuirass of the guardian",[(0,.018,.623),(0,.016,.855),(0,.015,1.061)],
         [.129,.207,.182],"cel-ivory",sides=8,accent="cel-stone")
    guardian_equipment(group)
    tube(group,"Cut stone helmet and brow",[(0,.010,1.075),(0,.010,1.166),(0,.028,1.334)],
         [.084,.119,.061],"cel-ivory",sides=7,accent="cel-stone")
    box(group,"Dark inset helmet visor",(0,-.091,1.179),(.150,.033,.038),"cel-shadow",.008)
    tube(group,"Gilt crest on the statue helmet",[(0,.03,1.315),(0,.052,1.412)],
         [.071,.023],"cel-gold",sides=6)


def guardian_equipment(group):
    for s in (-1,1):
        tube(group,"Carved guardian arm",[(s*.204,0,1.015),(s*.254,-.027,.850),(s*.245,-.132,.808)],
             [.077,.072,.055],"cel-stone",sides=7,accent="cel-ivory")
    shield = [(-.37,-.195,1.002),(-.146,-.195,1.002),(-.153,-.195,.765),
              (-.258,-.195,.665),(-.359,-.195,.767),(-.258,-.248,.877)]
    surface(group,"Carved azure guardian shield",shield,
            [(0,1,5),(1,2,5),(2,3,5),(3,4,5),(4,0,5)],["cel-azure","cel-azure-light"]*2+["cel-azure"])
    tube(group,"Guardian spear shaft",[(.263,-.133,.21),(.257,-.133,1.474)],
         [.025,.018],"cel-gold-dark",sides=6)
    tube(group,"Gilt guardian spear tip",[(.257,-.133,1.438),(.254,-.133,1.530),(.251,-.133,1.684)],
         [.021,.057,.005],"cel-gold-light",sides=6)
