"""Model sandstone Strongholds, the stepped sun temple and sculpted guardians."""

from math import cos, pi, sin, tau

from mesh import box, pebble, surface, tube
from structures import cloth


def pylon(group,at,width,depth,height):
    x,y,z = at
    outline = ((-1,-1),(1,-1),(1,1),(-1,1))
    verts = [(x+u*width*.5*s,y+v*depth*.5*s,z+h)
             for h,s in ((0,1),(height,.80)) for u,v in outline]
    surface(group,"Tapered dressed-sandstone pylon",verts,
            [(3,2,1,0),(4,5,6,7),(0,1,5,4),(1,2,6,5),(2,3,7,6),(3,0,4,7)],
            ["dr-stone-dark","dr-stone-light","dr-stone","dr-stone-light","dr-stone-dark","dr-stone"])
    box(group,"Wide sandstone pylon cornice",(x,y,z+height+.048),
        (width*.98,depth*.99,.126),"dr-stone-light",.031)
    box(group,"Inset turquoise pylon cornice",(x,y,z+height+.119),
        (width*.85,depth*.88,.044),"dr-turquoise-dark",.012)


def papyrus_column(group,at,height,radius):
    x,y,z = at
    tube(group,"Stepped sandstone column base",[(x,y,z+.025),(x,y,z+.110)],
         [radius*1.31,radius*1.27],"dr-stone-light",sides=8)
    tube(group,"Tapered papyrus column shaft",[(x,y,z+.103),(x,y,z+height-.165)],
         [radius,radius*.84],"dr-stone",sides=8,accent="dr-stone-light")
    tube(group,"Flared turquoise papyrus capital",[(x,y,z+height-.187),(x,y,z+height-.092),
         (x,y,z+height-.053)],[radius*.88,radius*1.48,radius*1.40],
         "dr-turquoise",sides=8,accent="dr-turquoise-light")
    box(group,"Square stone above the papyrus capital",(x,y,z+height-.024),
        (radius*2.95,radius*2.95,.063),"dr-stone-light",.013)
    for i in range(4):
        a = i*tau/4
        tube(group,"Narrow gold column binding",[(x+radius*cos(a),y+radius*sin(a),z+.22),
             (x+radius*.86*cos(a),y+radius*.86*sin(a),z+height-.21)],
             [.010,.008],"dr-gold-dark",sides=5)


def gate(group,at,width,height):
    x,y,z = at
    for s in (-1,1):
        box(group,"Carved stone temple jamb",(x+s*width*.47,y,z+height*.49),
            (width*.158,.197,height*.98),"dr-stone-light",.021)
    for i in range(6):
        u = (i-2.5)*width*.121
        box(group,"Dark carved temple door plank",(x+u,y+.041,z+height*.45),
            (width*.114,.084,height*.85),"dr-oak",.008)
    for h in (.27,.61):
        box(group,"Bronze temple door binding",(x,y-.016,z+height*h),
            (width*.75,.043,.048),"dr-bronze",.009)
    box(group,"Lintel of the temple entrance",(x,y,z+height+.019),
        (width*1.13,.242,.183),"dr-stone-light",.028)
    box(group,"Turquoise inset in the temple lintel",(x,y-.131,z+height+.043),
        (width*.86,.031,.053),"dr-turquoise",.008)


def sun_disk(group,at,radius):
    x,y,z = at
    tube(group,"Gold sun-disk backing",[(x,y+.018,z),(x,y-.025,z)],
         [radius,radius],"dr-gold",sides=16,accent="dr-gold-light")
    tube(group,"Turquoise center of the sun disk",[(x,y-.029,z),(x,y-.052,z)],
         [radius*.68,radius*.68],"dr-turquoise",sides=12,accent="dr-turquoise-light")
    for i in range(8):
        a = (i+.5)*tau/8
        verts = [(x+(radius+.028)*cos(a-.16),y-.016,z+(radius+.028)*sin(a-.16)),
                 (x+(radius+.151)*cos(a),y-.016,z+(radius+.151)*sin(a)),
                 (x+(radius+.028)*cos(a+.16),y-.016,z+(radius+.028)*sin(a+.16))]
        surface(group,"Separate gold ray of the sun disk",verts,[(0,1,2)],["dr-gold-light"])


def stronghold(group):
    box(group,"Lower sandstone Stronghold footing",(0,0,.057),(2.55,2.09,.166),"dr-stone-dark",.061)
    box(group,"Bright upper Stronghold terrace",(0,0,.165),(2.35,1.89,.093),"dr-stone-light",.035)
    box(group,"Recessed sandstone gate hall",(0,.066,.750),(1.65,1.35,1.11),"dr-stone-dark",.055)
    for side in (-1,1):
        stronghold_wing(group,side)
    gate(group,(0,-.768,.216),.81,1.05)
    for i in range(3):
        box(group,"Individual sandstone gate step",(0,-1.181+i*.113,.035+i*.032),
            (.847,.171,.083+i*.065),"dr-stone-light",.020)
    box(group,"First stepped Stronghold crown",(0,.110,1.415),(1.73,1.43,.194),"dr-stone-light",.041)
    box(group,"Turquoise parapet course",(0,.110,1.537),(1.61,1.31,.063),"dr-turquoise-dark",.015)
    box(group,"Upper block of the stepped keep",(0,.246,1.883),(1.06,.97,.665),"dr-stone",.046)
    box(group,"Upper carved cornice of the keep",(0,.246,2.256),(1.25,1.14,.145),"dr-stone-light",.036)
    box(group,"Turquoise coping on the keep",(0,.246,2.342),(1.14,1.02,.046),"dr-turquoise",.012)
    sun_disk(group,(0,-.270,1.920),.188)
    for x in (-.46,.46):
        tube(group,"Stone pyramid finial",[(x,.596,2.373),(x,.596,2.697)],
             [.107,.004],"dr-stone-light",sides=4,accent="dr-gold")
    box(group,"Central upper keep step",(0,.28,2.501),(.70,.58,.283),"dr-stone-light",.028)
    box(group,"Gold square below the sun finial",(0,.28,2.663),(.494,.406,.055),"dr-gold",.012)
    sun_disk(group,(0,.275,2.960),.177)


def stronghold_wing(group,side):
    x = side*.956
    pylon(group,(x,-.420,.216),.47,.64,1.30)
    for z in (.52,.98):
        box(group,"Carved turquoise slit on a corner pylon",(x,-.754,z),
            (.095,.035,.178),"dr-turquoise-dark",.009)
    box(group,"Gold border above the corner slit",(x,-.751,1.25),(.203,.033,.045),"dr-gold",.008)


def standard(group,key):
    tube(group,"Gold keep pennant mast",[(0,.30,2.86),(0,.30,3.76)],
         [.024,.017],"dr-gold-dark",sides=7)
    cloth(group,"Long keep pennant",key,(.271,.30,3.705),.54,.264)
    for x in (-.69,.69):
        tube(group,"Short bronze wall pennant bar",[(x-.124,-.801,1.376),(x+.124,-.801,1.376)],
             [.015,.015],"dr-bronze",sides=6)
        cloth(group,"Architectural wall pennant",key,(x,-.822,1.375),.229,.432)


def temple(group):
    for z,width,depth,height in ((.105,5.38,2.82,.227),(.275,5.04,2.50,.155),(.425,4.73,2.25,.151)):
        box(group,"Separate step of the temple stylobate",(0,.14,z),(width,depth,height),
            "dr-stone-light" if z>.2 else "dr-stone-dark",.046)
    box(group,"Recessed hall of the sun temple",(0,.378,1.327),(3.57,1.53,1.67),"dr-stone-dark",.064)
    for side in (-1,1):
        temple_wing(group,side)
    gate(group,(0,-.575,.516),1.13,1.48)
    box(group,"Broad sandstone temple cornice",(0,.21,2.247),(4.74,2.07,.230),"dr-stone-light",.044)
    box(group,"Turquoise line across the temple cornice",(0,-.855,2.279),(4.39,.047,.083),"dr-turquoise",.013)
    box(group,"Temple first roof step",(0,.24,2.480),(3.89,1.70,.241),"dr-stone",.043)
    box(group,"Temple upper roof step",(0,.25,2.707),(3.04,1.34,.217),"dr-stone-light",.040)
    box(group,"Low turquoise roof coping",(0,.25,2.844),(2.84,1.20,.070),"dr-turquoise-dark",.017)
    box(group,"High sun-disk backing panel",(0,.228,3.058),(1.23,.581,.430),"dr-stone",.037)
    sun_disk(group,(0,-.093,3.112),.312)
    for i in range(7):
        box(group,"Individual broad sandstone temple stair",(0,-1.819+i*.143,.031+i*.034),
            (1.64,.241,.070+i*.068),"dr-stone-light" if i%2 else "dr-stone",.022)


def temple_wing(group,side):
    pylon(group,(side*1.957,.298,.489),.684,1.78,1.62)
    for x in (side*.94,side*1.44):
        papyrus_column(group,(x,-.662,.505),1.64,.128)
    for j in range(3):
        box(group,"Temple pylon turquoise inset",(side*1.957,-.627,.91+j*.294),
            (.151,.033,.112),"dr-turquoise" if j%2 else "dr-turquoise-dark",.012)


def obelisk(group):
    box(group,"Square obelisk footing",(0,0,.067),(.62,.58,.149),"dr-stone-dark",.030)
    box(group,"Dressed obelisk base",(0,0,.167),(.50,.46,.110),"dr-stone-light",.021)
    tube(group,"Faceted sandstone obelisk shaft",[(0,0,.217),(0,0,1.535),(0,0,1.78)],
         [.195,.130,.004],"dr-stone",sides=4,accent="dr-stone-light")
    tube(group,"Gold cap on the pointed obelisk",[(0,0,1.566),(0,0,1.795)],
         [.116,.004],"dr-gold",sides=4,accent="dr-gold-light")
    for i in range(3):
        z = .49+i*.25
        box(group,"Turquoise glyph on the obelisk face",(0,-.169+i*.010,z),
            (.063,.025,.086),"dr-turquoise",.006,angle=pi/4)


def guardian(group):
    box(group,"Rectangular guardian sandstone plinth",(0,0,-.023),(.76,1.16,.377),"dr-stone-dark",.039)
    box(group,"Turquoise line around the guardian base",(0,0,.182),(.65,1.05,.051),"dr-turquoise-dark",.013)
    pebble(group,"Carved reclining feline body",(0,.162,.420),(.277,.419,.238),
           "dr-stone",sides=8,accent="dr-stone-light",phase=.23)
    for side in (-1,1):
        box(group,"Long carved forepaw",(side*.159,-.353,.274),(.203,.408,.139),"dr-stone-light",.032)
        pebble(group,"Rounded feline hindquarter",(side*.173,.356,.319),(.146,.20,.148),
               "dr-stone",sides=7,accent="dr-stone-light",phase=side)
    guardian_head(group)
    tube(group,"Carved tail resting on the plinth",[(.218,.38,.263),(.273,.47,.261),
         (.274,.24,.259),(.280,.02,.256)],[.036,.034,.030,.017],"dr-stone-dark",sides=6)


def guardian_head(group):
    tube(group,"Upright neck of the guardian",[(0,-.108,.421),(0,-.142,.656)],
         [.145,.152],"dr-stone",sides=8,accent="dr-stone-light")
    pebble(group,"Sculpted feline guardian head",(0,-.176,.770),(.181,.166,.176),
           "dr-stone-light",sides=8,accent="dr-stone",phase=.2)
    for side in (-1,1):
        tube(group,"Pointed guardian ear",[(side*.115,-.136,.855),(side*.139,-.110,1.019)],
             [.062,.004],"dr-stone",sides=4,accent="dr-stone-light")
        pebble(group,"Turquoise eye inset",(side*.079,-.312,.808),(.031,.016,.021),
               "dr-turquoise-dark",sides=6)
        box(group,"Turquoise temple cloth beside the head",(side*.173,-.103,.671),
            (.055,.157,.235),"dr-turquoise",.015)
    pebble(group,"Carved feline muzzle",(0,-.320,.721),(.099,.086,.073),
           "dr-stone-light",sides=7,accent="dr-stone")
    pebble(group,"Dark polished guardian nose",(0,-.381,.755),(.041,.024,.028),
           "dr-stone-dark",sides=6)
    tube(group,"Gold collar below the guardian head",[(0,-.145,.581),(0,-.145,.635)],
         [.155,.155],"dr-gold",sides=8)
